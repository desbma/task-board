#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod assets;
mod tw;

//
// Reports
//

#[get("/")]
fn report_default() -> rocket_contrib::templates::Template {
    report(rocket::http::RawStr::from_str(""))
}

#[get("/<report>")]
fn report(report: &rocket::http::RawStr) -> rocket_contrib::templates::Template {
    let tasks = tw::report(report).unwrap();  // TODO propagate error
    let mut context = std::collections::HashMap::new();
    let report_name = if report.is_empty() {
        "Default"
    }
    else {
        report
    };
    context.insert("title", format!("{} report", report_name));
    context.insert("txt", tasks);
    rocket_contrib::templates::Template::render("layout", &context)
}

//
// Assets
//

#[get("/favicon.ico")]
fn asset_favicon<'r>() -> rocket::response::Result<'r> {
    asset(std::path::PathBuf::from("favicon.ico"))
}

#[get("/static/<path..>")]
fn asset<'r>(path: std::path::PathBuf) -> rocket::response::Result<'r> {
    let filepath = path
        .clone()
        .into_os_string()
        .into_string()
        .or_else(|_| Err(rocket::http::Status::NotFound))?;
    assets::Assets::get(&filepath).map_or_else(
        || Err(rocket::http::Status::NotFound),
        |d| {
            let ext = path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_else(|| panic!("Could not get extension from path {}", filepath));
            let content_type =
                rocket::http::ContentType::from_extension(ext).unwrap_or_else(|| {
                    panic!("Could not guess file content type from extension {}", ext)
                });
            rocket::response::Response::build()
                .header(content_type)
                .sized_body(std::io::Cursor::new(d))
                .ok()
        },
    )
}

//
// Main
//

fn main() {
    rocket::ignite()
        .attach(rocket_contrib::templates::Template::fairing())
        .mount("/", routes![report_default, report, asset_favicon, asset])
        .launch();
}
