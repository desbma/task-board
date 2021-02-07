use std::collections::HashMap;
use std::str::FromStr;

use crate::run_opts::RunOpts;

#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    strum_macros::EnumString,
)]
pub enum AttributeType {
    #[strum(serialize = "date")]
    DateTime,
    #[strum(serialize = "numeric")]
    Numeric,
    #[strum(serialize = "string")]
    String,
    #[strum(serialize = "<type>")]
    Uda,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ColumnType {
    pub type_: AttributeType,
    pub read_only: bool,
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
    // TODO invalidate this when taskrc is changed (possible new UDA attributes?)
    static ref COLUMNS_NAME_TO_TYPE: HashMap<String, ColumnType> =
        build_column_name_to_type_map().unwrap();
}

fn build_column_name_to_type_map() -> anyhow::Result<HashMap<String, ColumnType>> {
    let mut r = HashMap::new();

    let output = invoke_internal(&["columns"], None, true)?;
    let mut output_lines = output.lines();

    // Compute offset for each column from first line (labels)
    let label_lines = [
        output_lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to get column labels"))?,
        output_lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to get column labels"))?,
    ];
    let column_char_offsets = parse_label_lines(label_lines)?.1;

    // Split lines at column offsets
    let mut prev_parsed_line: Option<(String, ColumnType)> = None;
    for line in output_lines {
        if line.is_empty() {
            break;
        }

        let mut column_attributes = Vec::new();
        column_attributes.reserve(column_char_offsets.len());

        for cur_column_char_offsets in column_char_offsets.windows(2) {
            let chunk = &line[std::cmp::min(line.len(), cur_column_char_offsets[0])
                ..std::cmp::min(line.len(), cur_column_char_offsets[1])];
            column_attributes.push(chunk.trim().to_string());
        }

        let last_chunk_start: usize = *column_char_offsets.last().unwrap();
        if last_chunk_start < line.len() {
            let last_chunk = &line[last_chunk_start..];
            column_attributes.push(last_chunk.trim().to_string());
        }

        // elements are in this order: Columns, Type, Modifiable, Supported Formats, Example
        assert!(column_attributes.len() >= 4);

        if column_attributes[0].is_empty() && column_attributes[3].is_empty() {
            continue;
        }

        let (base_column_name, column_type) = if !column_attributes[0].is_empty() {
            let type_ = AttributeType::from_str(&column_attributes[1])?;
            let read_only = column_attributes[2] == "Read Only";
            let column_type = ColumnType { type_, read_only };
            (column_attributes[0].clone(), column_type)
        } else {
            prev_parsed_line.unwrap()
        };
        if column_attributes[3].ends_with('*') {
            // Default format for this column name, we add both explicit and implicit format
            let mut fmt = column_attributes[3].clone();
            fmt.pop();
            let explicit_name = format!("{}.{}", base_column_name, fmt);
            r.insert(explicit_name, column_type.clone());
            r.insert(base_column_name.clone(), column_type.clone());
        } else {
            let name = format!("{}.{}", base_column_name.clone(), column_attributes[3]);
            r.insert(name, column_type.clone());
        }
        prev_parsed_line = Some((base_column_name, column_type));
    }

    Ok(r)
}

static CL_ARGS_READ_ONLY: [&str; 2] = ["rc.recurrence:0", "rc.gc:0"];
static CL_ARGS_OUTPUT: [&str; 2] = ["rc.verbose=label", "limit:4294967296"]; // 2^32

fn column_label_to_type(
    label: &str,
    label2column: &HashMap<String, String>,
) -> anyhow::Result<ColumnType> {
    match label2column.get(label) {
        None => Err(anyhow::anyhow!("Unknown column label {}", label)),
        Some(c) => COLUMNS_NAME_TO_TYPE
            .get(c)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Unknown column {}", c)),
    }
}

fn task_output(
    cmd_args: &[&str],
    options: Option<&RunOpts>,
) -> anyhow::Result<std::process::Output> {
    log::debug!("Running command: task {}", cmd_args.join(" "));

    let ts_before = std::time::Instant::now();

    let mut cmd = std::process::Command::new("task");
    cmd.args(cmd_args);
    if let Some(opts) = options {
        if let Some(task_data_dir) = &opts.task_data_dir {
            cmd.env("TASKDATA", task_data_dir);
        }
    }

    let output = cmd.output()?;

    //println!("task {}\n{:?}", cmd_args.join(" "), output);

    let ts_after = std::time::Instant::now();
    log::debug!(
        "Command took {}ms to run",
        ts_after.duration_since(ts_before).as_millis()
    );

    Ok(output)
}

fn invoke_internal(
    args: &[&str],
    options: Option<&RunOpts>,
    static_report: bool,
) -> anyhow::Result<String> {
    let mut cmd_args: Vec<&str> = Vec::new();
    if !static_report {
        cmd_args.extend(&CL_ARGS_OUTPUT);
    }
    cmd_args.extend(&CL_ARGS_READ_ONLY);
    let width = if let Some(some_options) = options {
        some_options.report_width // TODO remove uuid length if needed
    } else {
        0
    };
    let width_args = &format!("rc.defaultwidth:{}", width);
    cmd_args.push(width_args);
    cmd_args.extend(args);

    let output = task_output(&cmd_args, options)?;

    if !output.status.success() {
        let code = output.status.code().unwrap();
        if (code != 1) || (!output.stdout.is_empty()) {
            // task returns 1 with no output when having no results
            return Err(anyhow::anyhow!(
                "Task invocation with args {:?} failed with code {}",
                cmd_args,
                code
            ));
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout); // taskwarrior incorrectly splits utf-8 chars, fixed in 2.5.2?
    Ok(stdout.to_string())
}

pub fn invoke_external(args: &[&str], options: &RunOpts) -> anyhow::Result<(i32, String)> {
    if options.dry_run {
        Ok((0, "".to_string()))
    } else {
        let output = task_output(args, Some(options))?;

        let stdout = String::from_utf8_lossy(&output.stdout); // taskwarrior incorrectly splits utf-8 chars, fixed in 2.5.2?
        Ok((output.status.code().unwrap(), stdout.to_string()))
    }
}

#[allow(dead_code)]
fn show(what: &str, options: &RunOpts) -> anyhow::Result<Vec<String>> {
    let args = vec!["show", what];
    let output = invoke_internal(&args, Some(options), false)?;

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

fn dom_get(what: &str, options: &RunOpts) -> anyhow::Result<Vec<String>> {
    let args = vec!["_get", what];
    let output = invoke_internal(&args, Some(options), false)?;

    Ok(output
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Unexpected output for {:?}", args))?
        .split(',')
        .map(str::to_string)
        .collect())
}

#[allow(clippy::unnecessary_wraps)]
fn parse_label_lines(label_lines: [&str; 2]) -> anyhow::Result<(Vec<String>, Vec<usize>)> {
    let mut column_char_offsets = vec![0];
    let mut column_iter = label_lines[1].chars();
    let mut prev_pos = 0;
    while let Some(pos) = column_iter.position(char::is_whitespace) {
        let new_pos = pos + prev_pos + 1;
        column_char_offsets.push(new_pos);
        prev_pos = new_pos;
    }
    column_char_offsets.push(label_lines[1].len());

    let mut labels = Vec::new();
    for cur_column_char_offsets in column_char_offsets.windows(2) {
        let chunk = &label_lines[0][cur_column_char_offsets[0]
            ..std::cmp::min(label_lines[0].len(), cur_column_char_offsets[1])];
        labels.push(chunk.trim().to_string());
    }

    log::trace!(
        "labels = {:?}, column_char_offsets = {:?}",
        labels,
        column_char_offsets
    );

    Ok((labels, column_char_offsets))
}

pub fn report(report: &str, options: &RunOpts) -> anyhow::Result<Report> {
    // Get report columns & labels
    // TODO cache this until taskrc is changed
    // with task show data.location + inotify or keep mtime
    let column_arg = format!("rc.report.{}.columns", report);
    let label_arg = format!("rc.report.{}.labels", report);
    let report_columns = dom_get(&column_arg, options)?;
    log::trace!("report_columns = {:?}", report_columns);
    let report_labels = dom_get(&label_arg, options)?;
    log::trace!("report_labels = {:?}", report_labels);
    assert!(report_labels.len() == report_columns.len());

    // Prepend UUID to report columns & labels
    let mut args = vec![report];
    let custom_columns_arg = format!("{}:uuid,{}", column_arg, report_columns.join(","));
    args.push(&custom_columns_arg);
    let custom_labels_arg = format!("{}:UUID,{}", label_arg, report_labels.join(","));
    args.push(&custom_labels_arg);
    let output = invoke_internal(&args, Some(options), false)?;

    // Empty report case
    if output.is_empty() {
        return Ok(Report {
            column_types: vec![],
            labels: vec![],
            tasks: vec![],
        });
    }
    let mut report_output_lines = output.lines();

    // Build a hashmap of label -> column
    let mut label2column: HashMap<String, String> = HashMap::new();
    for (label, column) in report_labels.iter().zip(report_columns.iter()) {
        label2column.insert(label.to_string(), column.to_string());
    }

    // Compute offset for each column from first report line (labels), and also get used labels
    let label_lines = [
        report_output_lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to get column labels"))?,
        report_output_lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to get column labels"))?,
    ];
    let (mut present_labels, column_char_offsets) = parse_label_lines(label_lines)?;

    // Split lines at column offsets
    let mut report_tasks: Vec<Task> = Vec::new();
    for report_output_line in report_output_lines {
        let mut task_attributes = Vec::new();
        task_attributes.reserve(column_char_offsets.len());

        for cur_column_char_offsets in column_char_offsets.windows(2) {
            let chunk = &report_output_line[cur_column_char_offsets[0]
                ..std::cmp::min(report_output_line.len(), cur_column_char_offsets[1])];
            task_attributes.push(chunk.trim().to_string());
        }

        assert!(task_attributes.len() == column_char_offsets.len() - 1);

        // Separate UUID
        // TODO use a VecDeque to avoid expensive copy
        let uuid = task_attributes[0].to_string();
        task_attributes.remove(0);

        log::trace!(
            "task_attributes ({}) = {:?}",
            task_attributes.len(),
            task_attributes
        );

        report_tasks.push(Task {
            attributes: task_attributes,
            uuid,
        });
    }

    // Ignore added UUID
    // TODO use a VecDeque to avoid expensive copy
    present_labels.remove(0);
    log::trace!(
        "present_labels ({}) = {:?}",
        present_labels.len(),
        present_labels
    );

    // Get column types from present labels
    let column_types: Vec<ColumnType> = present_labels
        .iter()
        .map(|c| column_label_to_type(c, &label2column).unwrap()) // TODO remove unwrap
        .collect();
    log::trace!("column_types ({}) = {:?}", column_types.len(), column_types);

    Ok(Report {
        column_types,
        labels: present_labels,
        tasks: report_tasks,
    })
}
