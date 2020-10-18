#[test]
fn test_report_default() {
    let client = rocket::local::Client::new(super::rocket()).unwrap();
    let mut response = client.get("/").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(response.content_type(), Some(rocket::http::ContentType::HTML));
    assert!(response.body_string().unwrap().contains(&"<table>"));
}

#[test]
fn test_report() {
    let client = rocket::local::Client::new(super::rocket()).unwrap();
    let mut response = client.get("/waiting").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(response.content_type(), Some(rocket::http::ContentType::HTML));
    assert!(response.body_string().unwrap().contains(&"<table>"));
}

#[test]
fn test_asset() {
    let client = rocket::local::Client::new(super::rocket()).unwrap();
    let mut response = client.get("/static/favicon.ico").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert!(response.headers().get_one("Last-Modified").is_some());
    assert_eq!(response.content_type(), Some(rocket::http::ContentType::Icon));
    assert!(response.body_bytes().unwrap().len() > 0);
}
