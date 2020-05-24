pub fn report(report: &str) -> anyhow::Result<String> {
    let args = if report.is_empty() {
        vec![]
    } else {
        vec![report]
    };
    let output = std::process::Command::new("task") // TODO read only opts
        .args(&args)
        .output()?;
    if !output.status.success() {
        Err(anyhow::anyhow!(
            "task invocation failed with code {}",
            output.status.code().unwrap()
        ))
    } else {
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    }
}
