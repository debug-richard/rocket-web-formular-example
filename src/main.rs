extern crate proc_macro;
#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use fluent_templates::FluentLoader;
use fluent_templates::Loader;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rocket::config::LogLevel;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect, Responder};
use rocket::State;
use rocket_dyn_templates::Template;
use serde::Serialize;
use tera::Context;

use crate::captchas::CaptchaList;
use crate::lang::{lookup_lang, UserAgent};
use crate::lang::LOCALES;
use crate::save::save_to_file;
use crate::simple_captcha::SimpleCaptcha;

mod captchas;
mod save;
mod simple_captcha;
mod lang;

#[derive(FromForm, Debug, Serialize)]
struct KitRequest {
    given_name: String,
    surname: String,
    email_address: String,
    request_description: String,
    captcha_hash: String,
    captcha_code: String,
}

#[derive(Debug)]
pub enum ApiError {
    Missing,
    Invalid,
}

#[allow(dead_code)]
#[derive(Responder)]
enum TemplateRedirect {
    Template(Template),
    Redirect(Redirect),
    Flash(Flash<Redirect>),
    NotFound(Status),
}

#[get("/")]
async fn root(para: UserAgent) -> Redirect {
    Redirect::to(format!("/{}/formular", para.language))
}

#[get("/<lang>/formular")]
fn index(
    captchas: &State<CaptchaList>,
    flash: Option<FlashMessage>,
    lang: &str,
) -> TemplateRedirect {
    let new_captcha = SimpleCaptcha::new();
    let captcha_base64 = new_captcha.image_base64.clone();
    let captcha_hash = new_captcha.image_hash.clone();
    captchas.insert(new_captcha);

    let mut redirect = false;
    let lang = match lookup_lang(lang) {
        Ok(_) => lang,
        Err(_) => {
            redirect = true;
            "en"
        }
    };
    let mut context = Context::new();
    context.insert("lang", lang);
    let version: &'static str = env!("CARGO_PKG_VERSION");
    context.insert("version", version);
    context.insert("captcha", captcha_base64.as_str());
    context.insert("captcha_hash", captcha_hash.as_str());
    let flash = flash
        .map(|msg| (msg.kind().to_string(), msg.message().to_string()))
        .unwrap_or_else(|| ("None".to_string(), "".to_string()));
    context.insert("flash_type", flash.0.as_str());
    context.insert("flash_msg", flash.1.as_str());
    let context = context.into_json();
    if redirect {
        return TemplateRedirect::Flash(Flash::error(
            Redirect::to(format!("/{}/formular", lang)),
            LOCALES.lookup(&lookup_lang(lang).expect("Not found"), "alert_error_msg_lang_not_found"),
        ));
    }
    TemplateRedirect::Template(Template::render("home", &context))
}

#[post("/<lang>/formular", data = "<task>")]
async fn index_post(
    captchas: &State<CaptchaList>,
    task: Form<KitRequest>,
    lang: &str,
) -> TemplateRedirect {
    let lang = match lookup_lang(lang) {
        Ok(val) => val,
        Err(_) => {
            return TemplateRedirect::Flash(Flash::error(
                Redirect::to(format!("/{}/formular", lang)),
                LOCALES.lookup(&lookup_lang(lang).expect("Not found"), "alert_error_msg_lang_not_found"),
            ));
        }
    };
    let result = captchas.validate(task.captcha_hash.clone(), task.captcha_code.clone());
    match result {
        Ok(_) => {
            info!("Passed captcha");
            if task.given_name.is_empty() || task.given_name.len() > 200 {
                return TemplateRedirect::Flash(Flash::error(
                    Redirect::to(format!("/{}/formular", lang)),
                    LOCALES.lookup(&lang, "alert_error_msg_given_name_not_valid"),
                ));
            }
            if task.surname.is_empty() || task.surname.len() > 200 {
                return TemplateRedirect::Flash(Flash::error(
                    Redirect::to(format!("/{}/formular", lang)),
                    LOCALES.lookup(&lang, "alert_error_msg_surname_not_valid"),
                ));
            }
            if task.email_address.is_empty() || task.email_address.len() > 200 {
                return TemplateRedirect::Flash(Flash::error(
                    Redirect::to(format!("/{}/formular", lang)),
                    LOCALES.lookup(&lang, "alert_error_msg_email_not_valid"),
                ));
            }
            if task.request_description.is_empty() || task.request_description.len() > 10000 {
                return TemplateRedirect::Flash(Flash::error(
                    Redirect::to(format!("/{}/formular", lang)),
                    LOCALES.lookup(&lang, "alert_error_msg_motivation_not_valid"),
                ));
            }
            let result = serde_json::to_string(&task.into_inner()).expect("Could not serialize");

            let mut modified_result: HashMap<String, String> =
                serde_json::from_str(&result).expect("Failed deserializing");
            modified_result.remove_entry("captcha_hash");
            modified_result.remove_entry("captcha_code");

            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();
            modified_result.insert("timestamp".to_string(), now);

            let identifier: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            modified_result.insert("id".to_string(), identifier.clone());

            //info!("{:?}", result);
            //info!("{:?}", modified_result);

            match save_to_file(lang.clone(), identifier.clone(), modified_result) {
                Ok(_) => {}
                Err(err) => return TemplateRedirect::Flash(err),
            }

            let args = {
                let mut map = HashMap::new();
                map.insert(String::from("reqid"), identifier.into());
                map
            };
            return TemplateRedirect::Flash(Flash::success(
                Redirect::to(format!("/{}/formular", lang)),
                LOCALES.lookup_with_args(&lang, "alert_success_msg_ok", &args),
            ));
        }
        Err(_) => {
            info!("Failed captcha");
            return TemplateRedirect::Flash(Flash::error(
                Redirect::to(format!("/{}/formular", lang)),
                LOCALES.lookup(&lang, "alert_error_msg_captcha_failed"),
            ));
        }
    }
}

#[rocket::main]
async fn main() {
    if !env::var("ROCKET_ENV").is_ok() {
        // DEBUGGING ONLY
        env::set_var("PORT", "2000");
        env::set_var("RUST_APP_LOG", "info");
        env::set_var("ROCKET_ENV", "development");
        env::set_var("DATA_STORAGE_DIR", "/tmp/");
        // DEBUGGING ONLY END
    }

    pretty_env_logger::init_custom_env("RUST_APP_LOG");
    let version = env!("CARGO_PKG_VERSION");
    info!("Version: {:?}", version);

    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("log_level", parse_level()))
        .merge(("port", parse_port()));

    let captcha_list = Arc::new(Mutex::new(HashMap::new()));
    let captchas = CaptchaList {
        captchas: captcha_list,
    };

    let mut result = rocket::custom(figment)
        .mount("/", routes![root, index, index_post])
        .manage(captchas)
        .attach(Template::custom(|engines| {
            engines
                .tera
                .register_function("fluent", FluentLoader::new(&*LOCALES));
        }));

    if parse_env() == "development" {
        let static_files = FileServer::from("./static/");
        result = result.mount("/static", static_files);
    }

    let result = result.launch();

    if let Err(e) = result.await {
        println!("This rocket did not launch:");
        drop(e);
    };
}

fn parse_env() -> String {
    let rocket_environment = match env::var("ROCKET_ENV") {
        Ok(val) => val,
        Err(_e) => "dev".to_string(),
    };
    return rocket_environment;
}

fn parse_port() -> u16 {
    let port = match env::var("PORT") {
        Ok(val) => val,
        Err(_e) => "none".to_string(),
    };

    let port = match port.parse::<u16>() {
        Ok(val) => val,
        Err(_e) => panic!("Invalid port number!"),
    };

    return port;
}

fn parse_level() -> LogLevel {
    let log_level = match env::var("RUST_APP_LOG") {
        Ok(val) => val,
        Err(_e) => "none".to_string(),
    };

    let log_level = match log_level.as_str() {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Normal,
        "critical" => LogLevel::Critical,
        _ => LogLevel::Off,
    };

    return log_level;
}
