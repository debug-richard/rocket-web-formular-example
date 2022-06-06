use captcha::filters::{Dots, Noise, Wave};
use captcha::Captcha;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha512};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct SimpleCaptcha {
    pub image_base64: String,
    pub image_hash: String,
    pub code_salt: String,
    pub code: String,
    pub timestamp: SystemTime,
}

impl SimpleCaptcha {
    pub fn new() -> SimpleCaptcha {
        let chars: Vec<_> = "ABCDEFGHKLMNPQRTUVW2346789".chars().collect();
        let mut captcha = Captcha::new();
        captcha.set_chars(&chars);
        captcha.add_chars(6);
        captcha.apply_filter(Noise::new(0.4));
        captcha.apply_filter(Dots::new(15));
        captcha.apply_filter(Wave::new(2.0, 20.0).horizontal());
        captcha.view(220, 120);

        let captcha_string = &captcha.chars_as_string();
        let captcha_base64 = &captcha.as_base64().expect("Error.");

        let mut hasher = Sha512::new();
        hasher.update(captcha_base64);
        let hash: String = format!("{:X}", hasher.finalize());

        let code_salt: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let mut code_hash = Sha512::new();
        code_hash.update(code_salt.clone() + captcha_string);
        let code_hash: String = format!("{:X}", code_hash.finalize());

        return SimpleCaptcha {
            image_base64: captcha_base64.to_string(),
            image_hash: hash,
            code_salt: code_salt,
            code: code_hash.to_string(),
            timestamp: SystemTime::now(),
        };
    }

    pub fn check(self, code: String) -> bool {
        let mut salted_code = Sha512::new();
        salted_code.update(self.code_salt.clone() + code.to_uppercase().as_str());
        let code_hash: String = format!("{:X}", salted_code.finalize());
        return if self.code == code_hash { true } else { false };
    }
}
