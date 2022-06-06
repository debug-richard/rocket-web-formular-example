use fluent_templates::{LanguageIdentifier, static_loader};
use fluent_templates::loader::langid;
use rocket::{Request, request};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;

use crate::{ApiError};

pub const US_ENGLISH: LanguageIdentifier = langid!("en");
pub const GERMAN: LanguageIdentifier = langid!("de");

pub fn lookup_lang(lang: &str) -> Result<LanguageIdentifier, ()> {
    return match lang {
        "de" => Ok(GERMAN),
        "en" => Ok(US_ENGLISH),
        _ => Err(()),
    };
}

static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        //customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[derive(Debug)]
pub struct UserAgent {
    pub language: String,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for UserAgent {
    type Error = ApiError;
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language
    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut language = request.headers().get_one("accept-language").unwrap_or("en");
        if language != "en" {
            let mut languages = language.split(",");
            let first_language = languages.next().unwrap_or("en");

            // Use German when available in the first tag
            // Note: q-factor weighting is ignored!
            let first_language = first_language.contains("de");

            if first_language {
                language = "de";
            } else {
                language = "en";
            }
        }

        return Outcome::Success(UserAgent {
            language: language.to_string(),
        });
    }
}