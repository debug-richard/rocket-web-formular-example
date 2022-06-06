use crate::simple_captcha::SimpleCaptcha;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const SRV_TIMEOUT: f32 = 300.0;

#[derive(Debug)]
pub struct CaptchaList {
    pub(crate) captchas: Arc<Mutex<HashMap<String, SimpleCaptcha>>>,
}

impl CaptchaList {
    pub fn insert(&self, captcha: SimpleCaptcha) {
        clear_outdated(self.captchas.clone());
        let mut captchas = self.captchas.lock().unwrap();
        captchas.insert(captcha.image_hash.clone(), captcha);
    }

    pub fn validate(&self, hash: String, code: String) -> Result<String, String> {
        clear_outdated(self.captchas.clone());
        let mut captchas = self.captchas.lock().unwrap();

        let find_hash = captchas.get_key_value(hash.as_str());
        if find_hash.is_some() {
            let entry = find_hash.expect("Failed finding hash");
            let time_el = entry.1.timestamp.elapsed().unwrap().as_secs_f32();
            if time_el > SRV_TIMEOUT {
                captchas.remove(&*hash);
                return Err("Captcha timed out".to_string());
            }

            let mut ok = false;
            if entry.0 == &hash {
                if entry.1.clone().check(code) {
                    ok = true;
                } else {
                    info!("Captcha code check failed")
                }
            } else {
                ok = false;
            }
            captchas.remove(hash.as_str());

            if ok {
                return Ok("Captcha valid".to_string());
            }
            return Err("Captcha not valid".to_string());
        } else {
            return Err("Captcha not found".to_string());
        }
    }
}

fn clear_outdated(captchas: Arc<Mutex<HashMap<String, SimpleCaptcha>>>) {
    let mut captcha_list = captchas.lock().unwrap();

    let mut remove_list = Vec::new();
    for captcha in captcha_list.iter() {
        let time_el = captcha.1.timestamp.elapsed().unwrap().as_secs_f32();
        if time_el > SRV_TIMEOUT {
            remove_list.push(captcha.0.to_string());
        }
    }

    for element in remove_list.iter() {
        captcha_list.remove(element.as_str());
    }
}
