use crate::lang::LOCALES;
use fluent_templates::{LanguageIdentifier, Loader};
use rocket::response::{Flash, Redirect};
use std::collections::HashMap;
use std::{env, fs};

pub fn save_to_file(
    lang: LanguageIdentifier,
    identifier: String,
    result: HashMap<String, String>,
) -> Result<bool, Flash<Redirect>> {
    let path = env::var("DATA_STORAGE_DIR");
    let path = match path {
        Ok(value) => value,
        Err(_) => {
            error!("DATA_STORAGE_DIR not set!");
            return Err(Flash::error(
                Redirect::to(format!("/{}/formular", lang)),
                LOCALES.lookup(&lang, "alert_error_msg_general"),
            ));
        }
    };
    let path = format!("{}/request_{}.json", &path, identifier);
    match fs::write(
        path.clone(),
        serde_json::to_string_pretty(&result).expect("Failed serializing request"),
    ) {
        Ok(_) => {
            return Ok(true);
        }
        Err(_) => {
            error!("Could not write file to {:?}", path);
            return Err(Flash::error(
                Redirect::to(format!("/{}/formular", lang)),
                LOCALES.lookup(&lang, "alert_error_msg_general"),
            ));
        }
    }
}
