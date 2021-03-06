fn download(url: &str, target_filepath: &str) -> anyhow::Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    let mut target = std::fs::File::create(target_filepath)?;
    std::io::copy(&mut response, &mut target)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("assets")?;
    download(
        "https://taskwarrior.org/images/favicon.ico",
        "assets/favicon.ico",
    )?;
    download("https://andybrewer.github.io/mvp/mvp.css", "assets/mvp.css")?;
    let now = std::time::SystemTime::now();
    println!(
        "cargo:rustc-env=BUILD_DATETIME_HTTP_LAST_MODIFIED={}",
        httpdate::fmt_http_date(now)
    );
    Ok(())
}
