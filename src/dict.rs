use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use ::serde;
use ::serde_derive;
use ::serde_json;

pub trait Translator {
    fn translate(&self, &str) -> Option<&String>;
    fn language_name(&self) -> &String;
}

pub trait HasMutableMap {
    fn insert(&mut self, &str, &str);
    fn remove(&mut self, &str);
}

impl Dictionary {
    pub fn new(lang: &str, delimiter: &str) -> Dictionary {
        Dictionary {
            lang: lang.to_string(),
            delimiter: delimiter.to_string(),
            map: HashMap::new(),
        }
    }
}

impl Translator for Dictionary {
    fn translate(&self, text: &str) -> Option<&String> {
        let mut m = &self.map;
        for k in text.split(&self.delimiter) {
            match m.get(k) {
                Some(&PreDictValue::String(ref string)) => {
                    return Some(string);
                },
                Some(&PreDictValue::SubDictionary(ref hm)) => {
                    m = hm
                },
                _ => return None
            }
        }
        return None;
    }
    fn language_name(&self) -> &String {
        return &self.lang;
    }
}

enum PreDictValue {
    String(String),
    SubDictionary(HashMap<String,PreDictValue>),
}

#[derive(Serialize, Deserialize, Debug)]
struct PreDictionary<T> {
    lang: String,
    delimiter: String,
    map: T,
}

pub struct Dictionary {
    lang: String,
    delimiter: String,
    map: HashMap<String, PreDictValue>,
}

fn pd2d(pd: &PreDictionary<serde_json::map::Map<String, serde_json::Value>>) -> Dictionary {
    let delimiter = &pd.delimiter;

    fn recpd(hm: &serde_json::map::Map<String, serde_json::Value>) -> HashMap<String,PreDictValue>{
        let mut generic_hm: HashMap<String,PreDictValue> = HashMap::new();
        for (k, v) in hm {
            match v {
                &serde_json::Value::Object(ref sub_map) => {
                    generic_hm.insert(k.clone(), PreDictValue::SubDictionary(recpd(sub_map)));
                },
                &serde_json::Value::String(ref string) => {
                    generic_hm.insert(k.clone(), PreDictValue::String(string.clone()));
                },
                _ => continue
            }
        }
        return generic_hm;
    }

    let mut d: Dictionary = Dictionary{lang: pd.lang.clone(), delimiter: delimiter.clone(), map: recpd(&pd.map)};
    return d;
}

pub fn from_json_str(json: &str) -> Result<Dictionary, &str> {
    if let Ok(pd) = serde_json::from_str(json) {
        return Ok(pd2d(&pd));
    }
    return Err("Parse error");
}

pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Dictionary, &'static str> {
    if let Ok(file) = File::open(path) {
        if let Ok(pd) = serde_json::from_reader(file) {
            return Ok(pd2d(&pd));
        }else{
            return Err("Parse error");
        }
    }
    return Err("File error");
}


