#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

mod assets;
mod tw;

//
// Reports
//

#[get("/")]
fn report_default() -> Result<rocket_contrib::templates::Template, rocket::http::Status> {
    report(rocket::http::RawStr::from_str(""))
}

#[get("/<report>")]
fn report(
    report: &rocket::http::RawStr,
) -> Result<rocket_contrib::templates::Template, rocket::http::Status> {
    let tasks = tw::report(report).or_else(|_| Err(rocket::http::Status::NotFound))?;
    let mut context = std::collections::HashMap::new();
    let report_name = if report.is_empty() { "Default" } else { report };
    context.insert("title", format!("{} report", report_name));
    context.insert("txt", tasks);
    // TODO bundle templates, see https://github.com/SergioBenitez/Rocket/issues/943
    Ok(rocket_contrib::templates::Template::render(
        "layout", &context,
    ))
}

//
// Assets
//

#[get("/static/<path..>")]
fn asset(path: std::path::PathBuf, last_mod: assets::IfModified) -> rocket::response::Result {
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
                .raw_header("Last-Modified", last_mod.time)
                .sized_body(std::io::Cursor::new(d))
                .ok()
        },
    )
}

#[catch(304)]
fn not_modified(_req: &rocket::request::Request) {}

//
// Main
//

fn main() {
    rocket::ignite()
        .attach(rocket_contrib::templates::Template::fairing())
        .mount("/", routes![report_default, report, asset])
        .register(catchers![not_modified])
        .launch();
}
