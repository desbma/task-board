#[derive(rust_embed::RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

lazy_static! {
    static ref HTTP_LAST_MODIFIED_STRING: String =
        env!("BUILD_DATETIME_HTTP_LAST_MODIFIED").to_string();
    static ref HTTP_LAST_MODIFIED_TIME: std::time::SystemTime =
        httpdate::parse_http_date(&HTTP_LAST_MODIFIED_STRING).unwrap();
}

pub struct IfModified<'a> {
    pub time: &'a str,
}

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for IfModified<'a> {
    type Error = ();

    fn from_request(
        request: &'a rocket::request::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let if_mod = request.headers().get_one("If-Modified-Since");
        match if_mod {
            Some(if_mod_str) => {
                let if_mod_time = match httpdate::parse_http_date(if_mod_str) {
                    Ok(if_mod_time) => if_mod_time,
                    Err(_) => {
                        return rocket::request::Outcome::Success(IfModified {
                            time: &HTTP_LAST_MODIFIED_STRING,
                        });
                    }
                };
                if if_mod_time == *HTTP_LAST_MODIFIED_TIME {
                    rocket::request::Outcome::Failure((rocket::http::Status::NotModified, ()))
                } else {
                    rocket::request::Outcome::Success(IfModified {
                        time: &HTTP_LAST_MODIFIED_STRING,
                    })
                }
            }
            None => rocket::request::Outcome::Success(IfModified {
                time: &HTTP_LAST_MODIFIED_STRING,
            }),
        }
    }
}
