#[derive(serde::Serialize)]
pub struct Task {
    columns: Vec<String>,
}

static READ_ONLY_OPTS: [&str; 2] = ["rc.recurrence:no", "rc.gc:off"];

fn invoke(args: &[&str]) -> anyhow::Result<String> {
    let mut cmd_args: Vec<&str> = READ_ONLY_OPTS.to_vec();
    cmd_args.extend(args);
    let output = std::process::Command::new("task")
        .args(&cmd_args)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "task invocation with args {:?} failed with code {}",
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

pub fn report(report: &str) -> anyhow::Result<Vec<Task>> {
    let args = vec![report];
    let output = invoke(&args)?;

    let columns = show(&format!("report.{}.columns", report))?;
    println!("{:?}", columns);
    let labels = show(&format!("report.{}.labels", report))?;
    println!("{:?}", labels);

    // TODO
    // * get uuids
    // * use export as json, or _get to fetch columns values

    let mut tasks = Vec::new();
    for line in output.lines() {
        tasks.push(Task {
            columns: line.split_whitespace().map(String::from).collect(),
        })
    }
    Ok(tasks)
}
