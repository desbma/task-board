#[rstest::fixture]
fn test_data_dir() -> tempfile::TempDir {
    let data_dir = tempfile::Builder::new()
        .prefix("test_task_data")
        .tempdir()
        .unwrap();
    let task_cmds = vec![
        vec!["add", "test"],
        vec!["add", "test2", "due:eom"],
        vec![
            "add",
            "01234567890123456789012345678901234567890123456789012345678901234567890123456789",
        ],
    ];
    for task_cmd in task_cmds {
        std::process::Command::new("task")
            .args(&task_cmd)
            .env("TASKDATA", data_dir.path())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
    }
    data_dir
}

#[rstest::fixture]
fn run_opts(test_data_dir: tempfile::TempDir) -> crate::run_opts::RunOpts {
    let mut opts = crate::run_opts::get_default_opts();
    opts.task_data_dir = Some(test_data_dir.path().as_os_str().to_os_string());
    opts.tmp_dir = Some(test_data_dir);
    opts
}

#[rstest::fixture]
fn run_opts_low_report_width(run_opts: crate::run_opts::RunOpts) -> crate::run_opts::RunOpts {
    let mut run_opts = run_opts;
    run_opts.report_width = 60;
    run_opts
}

#[rstest::fixture]
fn rocket_client(run_opts: crate::run_opts::RunOpts) -> rocket::local::Client {
    rocket::local::Client::new(super::rocket(run_opts)).unwrap()
}

#[rstest::fixture]
fn rocket_client_low_report_width(
    run_opts_low_report_width: crate::run_opts::RunOpts,
) -> rocket::local::Client {
    rocket_client(run_opts_low_report_width)
}

#[rstest::rstest]
fn test_report_default(rocket_client: rocket::local::Client) {
    let mut response = rocket_client.get("/").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(
        response.content_type(),
        Some(rocket::http::ContentType::HTML)
    );
    assert!(response.body_string().unwrap().contains(&"<table>"));
}

#[rstest::rstest]
fn test_report(rocket_client: rocket::local::Client) {
    let mut response = rocket_client.get("/waiting").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(
        response.content_type(),
        Some(rocket::http::ContentType::HTML)
    );
    assert!(response.body_string().unwrap().contains(&"<table>"));
}

#[rstest::rstest]
fn test_report_low_width(rocket_client_low_report_width: rocket::local::Client) {
    let mut response = rocket_client_low_report_width.get("/waiting").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(
        response.content_type(),
        Some(rocket::http::ContentType::HTML)
    );
    assert!(response.body_string().unwrap().contains(&"<table>"));
}

#[rstest::rstest]
fn test_cmd(rocket_client: rocket::local::Client) {
    let mut response = rocket_client
        .post("/shell")
        .body("\"vbfqzsedmbcsdlkzf\"")
        .header(rocket::http::ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(
        response.content_type(),
        Some(rocket::http::ContentType::JSON)
    );
    assert_eq!(
        response.body_string().unwrap(),
        "{\"output\":\"\",\"code\":1}"
    );

    let mut response = rocket_client
        .post("/shell")
        .body("\"all\"")
        .header(rocket::http::ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(
        response.content_type(),
        Some(rocket::http::ContentType::JSON)
    );
    let body = response.body_string().unwrap();
    assert!(body.ends_with(",\"code\":0}"));
}

#[rstest::rstest]
fn test_asset(rocket_client: rocket::local::Client) {
    let mut response = rocket_client.get("/static/favicon.ico").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert!(response.headers().get_one("Last-Modified").is_some());
    assert_eq!(
        response.content_type(),
        Some(rocket::http::ContentType::Icon)
    );
    assert!(response.body_bytes().unwrap().len() > 0);
}

// TODO test for empty reports
