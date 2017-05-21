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
    delimiter: String,
    map: HashMap<String, String>,
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


#[derive(Serialize, Deserialize, Debug)]
struct PreDictionary<T> {
    lang: String,
    delimiter: String,
    map: T,
}

fn pd2d(pd: &PreDictionary<HashMap<String, serde_json::Value>>) -> Dictionary {
    let delimiter = &pd.delimiter;
    let mut hm: HashMap<String, serde_json::Value> = pd.map.clone();
    let mut d: Dictionary = Dictionary::new(&pd.lang, delimiter);
    let mut end_flag = false;
    
    // Recursively concatenate keys to generate a flat dictionary.
    while !end_flag {
        end_flag = true;
        let hmcopy = hm.clone();
        for k in hmcopy.keys() {
            match hmcopy.get(k) {
                Some(&serde_json::Value::Object(ref x)) => {
                    hm.remove(k);
                    for xk in x.keys() {
                        if let Some(xv) = x.get(xk) {
                            hm.insert(String::new() + k + delimiter + xk, xv.clone());
                            if let &serde_json::Value::Object(_) = xv {
                                end_flag = false;
                            }
                        }
                    }
                },
                _ => continue,
            }
        }
    }
    let mut resmap: HashMap<String, String> = HashMap::new();
    for (k, v) in &hm {
        match v {
            &serde_json::Value::String(ref sv) => {
                resmap.insert(k.clone(), sv.clone());
            },
            _ => continue,
        }
    }
    d.map = resmap;
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
        let mut d = Dictionary::new("test", ".");
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
        let d = from_json_str("{\"lang\":\"en\", \"delimiter\": \".\", \"map\":{\"a\":\"b\",\"c\":\"d\", \"e\": {\"f\": \"g\", \"h\": \"i\"}}}").unwrap();
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
        assert_eq!(d.translate("e.f").unwrap(), "g");
        assert_eq!(d.translate("e.h").unwrap(), "i");
    }

}
