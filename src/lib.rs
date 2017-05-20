#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub trait Translator {
    fn translate(&self, &str) -> Option<&String>;
    fn language_name(&self) -> &String;
    fn map(&self) -> &HashMap<String, String>;
}

pub trait HasMutableMap {
    fn map_mut(&mut self) -> &mut HashMap<String, String>;
    fn insert(&mut self, &str, &str);
    fn remove(&mut self, &str);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dictionary {
    lang: String,
    map: HashMap<String, String>,
}

impl Dictionary {
    pub fn new(lang: &str) -> Dictionary {
        Dictionary {
            lang: lang.to_string(),
            map: HashMap::new(),
        }
    }
}

impl Translator for Dictionary {
    fn translate(&self, text: &str) -> Option<&String> {
        return self.map.get(&text.to_string());
    }
    fn language_name(&self) -> &String {
        return &self.lang;
    }
    fn map(&self) -> &HashMap<String, String> {
        return &self.map;
    }
}

impl HasMutableMap for Dictionary {
    fn map_mut(&mut self) -> &mut HashMap<String, String> {
        return &mut self.map;
    }
    fn insert(&mut self, k: &str, v: &str) {
        self.map.insert(k.to_string(), v.to_string());
    }
    fn remove(&mut self, k: &str) {
        self.map.remove(&k.to_string());
    }
}

pub fn from_json_str(json: &str) -> Result<Dictionary, &str> {
    if let Ok(d) = serde_json::from_str(json) {
        return Ok(d);
    }
    return Err("Parse error");
}

pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Dictionary, &'static str> {
    if let Ok(file) = File::open(path) {
        if let Ok(d) = serde_json::from_reader(file) {
            return Ok(d);
        }else{
            return Err("Parse error");
        }
    }
    return Err("File error");
}

#[cfg(test)]
mod tests {
    use ::*;
    use std::thread;

    #[test]
    fn test_insert_remove_translate() {
        let mut d = Dictionary::new("test");
        d.insert("key1", "value1");
        d.insert("key2", "value2");
        assert_eq!(d.translate("key1").unwrap(), "value1");
        assert_eq!(d.translate("key2").unwrap(), "value2");
        {
            let map = d.map();
            assert_eq!(map.keys().count(), 2);
        }
        d.remove("key2");
        assert_eq!(d.translate("key1").unwrap(), "value1");
        {
            let map = d.map();
            assert_eq!(map.keys().count(), 1);
        }
    }

    #[test]
    fn test_loading() {
        let d = from_json_str("{\"lang\":\"en\", \"map\":{\"a\":\"b\",\"c\":\"d\"}}").unwrap();
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
    }

}
