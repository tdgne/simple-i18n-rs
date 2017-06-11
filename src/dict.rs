use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::error;

use serde_json;
use toml;

pub enum DictValue {
    String(String),
    SubDictionary(HashMap<String,DictValue>),
}

#[derive(Serialize, Deserialize, Debug)]
struct PreDictionary<T> {
    name: String,
    map: T,
}

fn pd2d(pd: &PreDictionary<serde_json::map::Map<String, serde_json::Value>>, delimiter: &str) -> Dictionary {
    fn recpd(hm: &serde_json::map::Map<String, serde_json::Value>) -> HashMap<String,DictValue>{
        let mut generic_hm: HashMap<String,DictValue> = HashMap::new();
        for (k, v) in hm {
            match v {
                &serde_json::Value::Object(ref sub_map) => {
                    generic_hm.insert(k.clone(), DictValue::SubDictionary(recpd(sub_map)));
                },
                &serde_json::Value::String(ref string) => {
                    generic_hm.insert(k.clone(), DictValue::String(string.clone()));
                },
                _ => continue
            }
        }
        return generic_hm;
    }

    return Dictionary{name: pd.name.clone(), delimiter: delimiter.to_string(), map: recpd(&pd.map)};
}

pub struct Dictionary {
    pub name: String,
    pub delimiter: String,
    pub map: HashMap<String, DictValue>,
}

impl Dictionary {
    // Not supported yet.
    /*
    pub fn new(name: &str, delimiter: &str) -> Dictionary {
        Dictionary {
            name: name.to_string(),
            delimiter: delimiter.to_string(),
            map: HashMap::new(),
        }
    }
    */

    pub fn translate(&self, key: &str) -> Option<&String> {
        let mut m = &self.map;
        for k in key.split(&self.delimiter) {
            match m.get(k) {
                Some(&DictValue::String(ref string)) => {
                    return Some(string);
                },
                Some(&DictValue::SubDictionary(ref hm)) => {
                    m = hm
                },
                _ => return None
            }
        }
        return None;
    }

    pub fn with_json_str(json: &str, delimiter: &str) -> Result<Dictionary, Box<error::Error>> {
        let pd = try!(serde_json::from_str(json));
        return Ok(pd2d(&pd, delimiter));
    }

    pub fn with_json_filepath<P: AsRef<Path>>(path: P, delimiter: &str) -> Result<Dictionary, Box<error::Error>> {
        let file = try!(File::open(path));
        let mut br = BufReader::new(file);
        let mut json = String::new();
        try!(br.read_to_string(&mut json));
        return Self::with_json_str(&json, delimiter);
    }

    pub fn with_toml_str(toml: &str, delimiter: &str) -> Result<Dictionary, Box<error::Error>> {
        let pd = try!(toml::from_str(toml));
        return Ok(pd2d(&pd, delimiter));
    }

    pub fn with_toml_filepath<P: AsRef<Path>>(path: P, delimiter: &str) -> Result<Dictionary, Box<error::Error>> {
        let file = try!(File::open(path));
        let mut br = BufReader::new(file);
        let mut toml = String::new();
        try!(br.read_to_string(&mut toml));
        return Self::with_toml_str(&toml, delimiter);
    }
}

#[cfg(test)]
mod tests {
    use ::dict::*;

    #[test]
    fn test_loading_from_json_str() {
        let d = Dictionary::with_json_str("{\"name\":\"en\", \"map\":{\"a\":\"b\",\"c\":\"d\", \"e\": {\"f\": \"g\", \"h\": \"i\"}}}", ".").unwrap();
        assert_eq!(d.name, "en");
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
        assert_eq!(d.translate("e.f").unwrap(), "g");
        assert_eq!(d.translate("e.h").unwrap(), "i");
    }

    #[test]
    fn test_loading_from_toml_str() {
        let d = Dictionary::with_toml_str(r#"
            name = "ja"

            [map]
            a = "b"
            c = "d"

            [map.e]
            f = "g"
            h = "i"
        "#, ".").unwrap();
        assert_eq!(d.name, "ja");
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
        assert_eq!(d.translate("e.f").unwrap(), "g");
        assert_eq!(d.translate("e.h").unwrap(), "i");
    }
}
