use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::error;

use serde_json;
use toml;

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

impl Dictionary {
    pub fn translate(&self, key: &str) -> Option<&String> {
        let mut m = &self.map;
        for k in key.split(&self.delimiter) {
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
    pub fn language_name(&self) -> &String {
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

    let d: Dictionary = Dictionary{lang: pd.lang.clone(), delimiter: delimiter.clone(), map: recpd(&pd.map)};
    return d;
}

pub fn from_json_str(json: &str) -> Result<Dictionary, Box<error::Error>> {
    let pd = try!(serde_json::from_str(json));
    return Ok(pd2d(&pd));
}

pub fn from_json_filepath<P: AsRef<Path>>(path: P) -> Result<Dictionary, Box<error::Error>> {
    let file = try!(File::open(path));
    let mut br = BufReader::new(file);
    let mut json = String::new();
    try!(br.read_to_string(&mut json));
    return from_json_str(&json);
}

pub fn from_toml_str(toml: &str) -> Result<Dictionary, Box<error::Error>> {
    let pd = try!(toml::from_str(toml));
    return Ok(pd2d(&pd));
}

pub fn from_toml_filepath<P: AsRef<Path>>(path: P) -> Result<Dictionary, Box<error::Error>> {
    let file = try!(File::open(path));
    let mut br = BufReader::new(file);
    let mut toml = String::new();
    try!(br.read_to_string(&mut toml));
    return from_toml_str(&toml);
}

#[cfg(test)]
mod tests {
    use ::dict::*;

    #[test]
    fn test_loading_from_json_str() {
        let d = from_json_str("{\"lang\":\"en\", \"delimiter\": \".\", \"map\":{\"a\":\"b\",\"c\":\"d\", \"e\": {\"f\": \"g\", \"h\": \"i\"}}}").unwrap();
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
        assert_eq!(d.translate("e.f").unwrap(), "g");
        assert_eq!(d.translate("e.h").unwrap(), "i");
    }

    #[test]
    fn test_loading_from_toml_str() {
        let d = from_toml_str(r#"
            lang = "ja"
            delimiter = "."

            [map]
            a = "b"
            c = "d"

            [map.e]
            f = "g"
            h = "i"
        "#).unwrap();
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
        assert_eq!(d.translate("e.f").unwrap(), "g");
        assert_eq!(d.translate("e.h").unwrap(), "i");
    }
}
