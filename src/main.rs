#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

mod assets;
#[cfg(test)]
mod test;
mod tw;

//
// Reports
//

#[derive(serde::Serialize)]
struct TemplateContext {
    title: String,
    report: tw::Report,
}

#[get("/")]
fn report_default() -> Result<rocket_contrib::templates::Template, rocket::http::Status> {
    report(rocket::http::RawStr::from_str("next")) // TODO get default report dynamically?
}

#[get("/<report_name>")]
fn report(
    report_name: &rocket::http::RawStr,
) -> Result<rocket_contrib::templates::Template, rocket::http::Status> {
    let report = tw::report(report_name).or_else(|_| Err(rocket::http::Status::NotFound))?;
    let context = TemplateContext {
        title: format!("{} report", report_name),
        report,
    };
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
// Tera custom filters
//

lazy_static! {
    static ref COLUMN_TO_CLASS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert(tw::ColumnType::_DateTime.to_string(), "dt".to_string());
        m.insert(tw::ColumnType::String.to_string(), "str".to_string());
        m.insert(tw::ColumnType::ReadOnly.to_string(), "ro".to_string());
        m
    };
}

fn column_class(
    column: rocket_contrib::templates::tera::Value,
    _args: std::collections::HashMap<String, rocket_contrib::templates::tera::Value>,
) -> rocket_contrib::templates::tera::Result<rocket_contrib::templates::tera::Value> {
    let s =
        rocket_contrib::templates::tera::try_get_value!("column_class", "value", String, column);
    let r = COLUMN_TO_CLASS
        .get(&s)
        .ok_or_else(|| format!("Unknown column {}", s))?;
    Ok(rocket_contrib::templates::tera::to_value(r).unwrap())
}

//
// Main
//

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(rocket_contrib::templates::Template::custom(
            |engines: &mut rocket_contrib::templates::Engines| {
                engines.tera.register_filter("column_class", column_class);
            },
        ))
        .mount("/", routes![report_default, report, asset])
        .register(catchers![not_modified])
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_module_level(
            "hyper",
            std::cmp::max(log::LevelFilter::Info, log::max_level()),
        )
        .with_module_level(
            "mio::poll",
            std::cmp::max(log::LevelFilter::Info, log::max_level()),
        )
        .with_module_level("_", std::cmp::max(log::LevelFilter::Info, log::max_level()))
        .init()
        .unwrap();

    rocket().launch();
}
