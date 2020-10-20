#[derive(Debug, serde::Serialize, strum_macros::ToString)]
pub enum ColumnType {
    _DateTime,
    String,
    ReadOnly,
}

#[derive(serde::Serialize)]
struct Task {
    attributes: Vec<String>,
    uuid: String,
}

#[derive(serde::Serialize)]
pub struct Report {
    column_types: Vec<ColumnType>,
    labels: Vec<String>,
    tasks: Vec<Task>,
}

lazy_static! {
    static ref READ_ONLY_COLUMNS: std::collections::HashSet<String> = {
        let mut s = std::collections::HashSet::new();
        s.insert("id".to_string());
        s.insert("urgency".to_string());
        s
    };
}

static READ_ONLY_OPTS: [&str; 2] = ["rc.recurrence:no", "rc.gc:off"];
static OUTPUT_OPTS: [&str; 3] = ["rc.defaultwidth:", "limit:", "rc.verbose=label"]; // TODO allow setting width from command line

fn column_label_to_type(
    label: &str,
    label2column: &std::collections::HashMap<String, String>,
) -> anyhow::Result<ColumnType> {
    match label2column.get(label) {
        None => Err(anyhow::anyhow!("Unknown column label {}", label)),
        Some(c) => {
            if READ_ONLY_COLUMNS.contains(c) {
                Ok(ColumnType::ReadOnly)
            } else {
                Ok(ColumnType::String)
            }
        }
    }
}

fn invoke(args: &[&str]) -> anyhow::Result<String> {
    let mut cmd_args: Vec<&str> = READ_ONLY_OPTS.to_vec();
    cmd_args.extend(&OUTPUT_OPTS);
    cmd_args.extend(args);
    log::debug!("Running command: task {}", cmd_args.join(" "));

    let ts_before = std::time::Instant::now();

    let output = std::process::Command::new("task")
        .args(&cmd_args)
        .output()?;

    let ts_after = std::time::Instant::now();
    log::debug!("Command took {}ms to run", ts_after.duration_since(ts_before).as_millis());

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Task invocation with args {:?} failed with code {}",
            cmd_args,
            output.status.code().unwrap()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout); // taskwarrior incorrectly splits utf-8 chars
    Ok(stdout.to_string())
}

#[allow(dead_code)]
fn show(what: &str) -> anyhow::Result<Vec<String>> {
    let args = vec!["show", what];
    let output = invoke(&args)?;

    for line in output.lines() {
        if !line.starts_with(what) {
            continue;
        }
        let data: Vec<String> = line
            .split_at(what.len() + 1)
            .1
            .split(',')
            .map(str::to_string)
            .collect();
        return Ok(data);
    }

    Err(anyhow::anyhow!("Unexpected output for {:?}", args))
}

fn dom_get(what: &str) -> anyhow::Result<Vec<String>> {
    let args = vec!["_get", what];
    let output = invoke(&args)?;

    Ok(output
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Unexpected output for {:?}", args))?
        .split(',')
        .map(str::to_string)
        .collect())
}

pub fn report(report: &str) -> anyhow::Result<Report> {
    // Get report columns & labels
    // TODO cache this until taskrc is changed
    // with task show data.location + inotify or keep mtime
    let column_arg = format!("rc.report.{}.columns", report);
    let label_arg = format!("rc.report.{}.labels", report);
    let report_columns = dom_get(&column_arg)?;
    log::trace!("report_columns = {:?}", report_columns);
    let report_labels = dom_get(&label_arg)?;
    log::trace!("report_labels = {:?}", report_labels);
    assert!(report_labels.len() == report_columns.len());

    // Prepend UUID to report columns & labels
    let mut args = vec![report];
    let custom_columns_arg = format!("{}:uuid,{}", column_arg, report_columns.join(","));
    args.push(&custom_columns_arg);
    let custom_labels_arg = format!("{}:UUID,{}", label_arg, report_labels.join(","));
    args.push(&custom_labels_arg);
    let output = invoke(&args)?;
    let mut report_output_lines = output.lines();

    // Build a hashmap of label -> column
    let mut label2column: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (label, column) in report_labels.iter().zip(report_columns.iter()) {
        label2column.insert(label.to_string(), column.to_string());
    }

    // Compute offset for each column from first report line (labels), and also get used labels
    let label_line = report_output_lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to get report column labels"))?;
    report_output_lines.next(); // Drop line after label made of '-' chars
    let mut char_is_in_label = true;
    let mut column_char_offsets = Vec::new();
    column_char_offsets.push(0);
    let mut present_labels = Vec::new();
    let mut cur_label = String::new();
    for (line_offset, c) in label_line.chars().enumerate() {
        if c.is_ascii_whitespace() {
            if char_is_in_label {
                present_labels.push(cur_label.clone());
                cur_label.clear();
                char_is_in_label = false;
            }
        } else {
            if !char_is_in_label {
                char_is_in_label = true;
                column_char_offsets.push(line_offset);
            }
            cur_label.push(c);
        }
    }
    if !cur_label.is_empty() {
        present_labels.push(cur_label);
    }

    // Split lines at column offsets
    let mut report_tasks: Vec<Task> = Vec::new();
    for report_output_line in report_output_lines {
        let mut task_attributes = Vec::new();
        task_attributes.reserve(column_char_offsets.len());

        for cur_column_char_offsets in column_char_offsets.windows(2) {
            let chunk = &report_output_line[cur_column_char_offsets[0]..cur_column_char_offsets[1]];
            task_attributes.push(chunk.trim().to_string());
        }

        let last_chunk_start: usize = *column_char_offsets.last().unwrap();
        let last_chunk = &report_output_line[last_chunk_start..];
        task_attributes.push(last_chunk.trim().to_string());

        assert!(task_attributes.len() == column_char_offsets.len());

        // Separate UUID
        // TODO use a VecDeque to avoid expensive copy
        let uuid = task_attributes[0].to_string();
        task_attributes.remove(0);

        report_tasks.push(Task {
            attributes: task_attributes,
            uuid,
        });
    }

    // Ignore added UUID
    // TODO use a VecDeque to avoid expensive copy
    present_labels.remove(0);
    log::trace!("present_labels = {:?}", present_labels);

    // Get column types from present labels
    let column_types: Vec<ColumnType> = present_labels
        .iter()
        .map(|c| column_label_to_type(c, &label2column).unwrap()) // TODO remove unwrap
        .collect();
    log::trace!("column_types = {:?}", column_types);

    Ok(Report {
        column_types,
        labels: present_labels,
        tasks: report_tasks,
    })
}
