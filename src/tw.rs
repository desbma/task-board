#[derive(serde::Serialize)]
pub struct Task {
    columns: Vec<String>,
}

static READ_ONLY_OPTS: [&str; 2] = ["rc.recurrence:no", "rc.gc:off"];

pub fn report(report: &str) -> anyhow::Result<Vec<Task>> {
    let mut args: Vec<&str> = READ_ONLY_OPTS.to_vec();
    if !report.is_empty() {
        args.push(report);
    };
    let output = std::process::Command::new("task").args(&args).output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "task invocation failed with code {}",
            output.status.code().unwrap()
        ));
    }

    // TODO rewrite with this logic:
    // * get report column names & types
    // * get uuids
    // * use export as json, or _get to fetch columns values

    let mut tasks = Vec::new();
    for line in std::str::from_utf8(&output.stdout)?.lines() {
        tasks.push(Task {
            columns: line.split_whitespace().map(String::from).collect(),
        })
    }
    Ok(tasks)
}
