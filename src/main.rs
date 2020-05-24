#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod tw;

#[get("/")]
fn report_default() -> Option<String> {
    report(rocket::http::RawStr::from_str(""))
}

#[get("/<report>")]
fn report(report: &rocket::http::RawStr) -> Option<String> {
    tw::report(report).ok()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![report_default, report])
        .launch();
}
