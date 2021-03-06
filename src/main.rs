#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

mod assets;
mod run_opts;
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
fn report_default(
    options: rocket::State<run_opts::RunOpts>,
) -> anyhow::Result<rocket_contrib::templates::Template> {
    report(rocket::http::RawStr::from_str("next"), options) // TODO get default report dynamically?
}

#[get("/<report_name>")]
#[allow(clippy::unnecessary_wraps)]
fn report(
    report_name: &rocket::http::RawStr,
    options: rocket::State<run_opts::RunOpts>,
) -> anyhow::Result<rocket_contrib::templates::Template> {
    let report = tw::report(report_name, &*options).unwrap(); //or_else(|_| Err(rocket::http::Status::NotFound))?;
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
// XHR
//

#[derive(serde::Serialize)]
struct CmdResult {
    output: String,
    code: i32,
}

#[post("/shell", format = "json", data = "<cmd>")]
fn cmd(
    cmd: rocket_contrib::json::Json<String>,
    options: rocket::State<run_opts::RunOpts>,
) -> anyhow::Result<rocket_contrib::json::Json<CmdResult>> {
    let cmd_str = &cmd.to_string();
    let cmd_split = shell_words::split(cmd_str)?;
    let args: Vec<&str> = cmd_split.iter().map(AsRef::as_ref).collect();
    let res = tw::invoke_external(&args[..], &options)?;

    Ok(rocket_contrib::json::Json(CmdResult {
        output: res.1,
        code: res.0,
    }))
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
        .map_err(|_| rocket::http::Status::NotFound)?;
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
    static ref COLUMN_ATTRIBUTE_TYPE_TO_CLASS: std::collections::HashMap<tw::AttributeType, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert(tw::AttributeType::DateTime, "dt".to_string());
        m.insert(tw::AttributeType::String, "str".to_string());
        m.insert(tw::AttributeType::Numeric, "num".to_string());
        m.insert(tw::AttributeType::Uda, "str".to_string());
        m
    };
}

fn column_html_classes(
    column: rocket_contrib::templates::tera::Value,
    _args: std::collections::HashMap<String, rocket_contrib::templates::tera::Value>,
) -> rocket_contrib::templates::tera::Result<rocket_contrib::templates::tera::Value> {
    let s = rocket_contrib::templates::tera::try_get_value!(
        "column_classes",
        "value",
        tw::ColumnType,
        column
    );
    let mut r = COLUMN_ATTRIBUTE_TYPE_TO_CLASS
        .get(&s.type_)
        .ok_or_else(|| format!("Unknown column type {:?}", s.type_))?
        .clone();
    if s.read_only {
        r.push_str(" ro");
    }
    Ok(rocket_contrib::templates::tera::to_value(r).unwrap())
}

//
// Main
//

fn rocket(options: run_opts::RunOpts) -> rocket::Rocket {
    rocket::ignite()
        .attach(rocket_contrib::templates::Template::custom(
            |engines: &mut rocket_contrib::templates::Engines| {
                engines
                    .tera
                    .register_filter("column_classes", column_html_classes);
            },
        ))
        .mount("/", routes![report_default, report, cmd, asset])
        .register(catchers![not_modified])
        .manage(options)
}

fn main() {
    let run_opts = run_opts::get_cl_opts();

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

    rocket(run_opts).launch();
}
