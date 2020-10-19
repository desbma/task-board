#[derive(Debug, serde::Serialize, strum_macros::ToString)]
pub enum ColumnType {
    _DateTime,
    String,
    ReadOnly,
}

type TaskLine = Vec<String>;

#[derive(serde::Serialize)]
pub struct Report {
    column_types: Vec<ColumnType>,
    labels: Vec<String>,
    tasks: Vec<TaskLine>,
}

lazy_static! {
    static ref READ_ONLY_COLUMNS: std::collections::HashSet<String> = {
        let mut s = std::collections::HashSet::new();
        s.insert("id".to_string());
        s.insert("urgency".to_string());
        s
    };
}

static READ_ONLY_OPTS: [&str; 3] = ["rc.recurrence:no", "rc.gc:off", "rc.verbose=label"];

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
    cmd_args.extend(args);
    let output = std::process::Command::new("task")
        .args(&cmd_args)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Task invocation with args {:?} failed with code {}",
            cmd_args,
            output.status.code().unwrap()
        ));
    }

    let stdout = std::str::from_utf8(&output.stdout)?;
    Ok(stdout.to_string())
}

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

pub fn report(report: &str) -> anyhow::Result<Report> {
    let args = vec![report];
    let output = invoke(&args)?;

    // TODO cache this until taskrc is changed
    // with task show data.location + inotify or keep mtime

    let report_columns = show(&format!("report.{}.columns", report))?;
    // TODO debug logging macro
    log::debug!("report_columns = {:?}", report_columns);

    let report_labels = show(&format!("report.{}.labels", report))?;
    log::debug!("report_labels = {:?}", report_labels);
    assert!(report_labels.len() == report_columns.len());

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
    log::debug!("present_labels = {:?}", present_labels);

    // Split lines at column offsets
    let mut report_tasks: Vec<TaskLine> = Vec::new();
    for report_output_line in report_output_lines {
        let mut task_line = TaskLine::new();
        task_line.reserve(column_char_offsets.len());

        for cur_column_char_offsets in column_char_offsets.windows(2) {
            let chunk = &report_output_line[cur_column_char_offsets[0]..cur_column_char_offsets[1]];
            task_line.push(chunk.trim().to_string());
        }

        let last_chunk_start: usize = *column_char_offsets.last().unwrap();
        let last_chunk = &report_output_line[last_chunk_start..];
        task_line.push(last_chunk.trim().to_string());

        assert!(task_line.len() == column_char_offsets.len());
        report_tasks.push(task_line);
    }

    // Get column types from present labels
    let column_types: Vec<ColumnType> = present_labels
        .iter()
        .map(|c| column_label_to_type(c, &label2column).unwrap()) // TODO remove unwrap
        .collect();
    log::debug!("column_types = {:?}", column_types);

    Ok(Report {
        column_types,
        labels: present_labels,
        tasks: report_tasks,
    })
}
