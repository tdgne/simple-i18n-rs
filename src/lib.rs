#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[cfg(test)]
extern crate tempdir;

use std::error;
use std::fs;

pub mod dict;

pub struct Cascade {
    dicts: Vec<dict::Dictionary>,
}

impl Cascade {
    pub fn from_filepaths(files: &Vec<&str>) -> Result<Cascade, Box<error::Error>> {
        let mut ds: Vec<dict::Dictionary> = Vec::new();
        for f in files {
            ds.push(try!(dict::from_json_filepath(f)));
        }
        Ok(Cascade{dicts: ds})
    }
    pub fn from_dirpath(dir: &str) -> Result<Cascade, Box<error::Error>> {
        let rd = try!(fs::read_dir(dir));
        let mut v:Vec<String> = Vec::new();
        for f in rd {
            if let Some(pathstr) = try!(f).path().to_str() {
                v.push(pathstr.to_string());
            }
        }
        return Cascade::from_filepaths((&v.iter().map(|s| &*s as &str).collect::<Vec<&str>>()));
    }
    pub fn translate<'a, 'b>(&'a self, key: &'b str) -> Option<&'a str> {
        for dict in &self.dicts {
            if let Some(x) = dict.translate(key) {
                return Some(&x);
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use ::*;
    use ::dict::*;
    use std::fs::File;
    use std::io::Write;

    /*
    #[test]
    fn test_insert_remove_translate() {
        let mut d = Dictionary::new("test", ".");
        d.insert("key1", "value1");
        d.insert("key2", "value2");
        assert_eq!(d.translate("key1").unwrap(), "value1");
        assert_eq!(d.translate("key2").unwrap(), "value2");
        d.remove("key2");
        assert_eq!(d.translate("key1").unwrap(), "value1");
    }
    */

    #[test]
    fn test_loading() {
        let d = from_json_str("{\"lang\":\"en\", \"delimiter\": \".\", \"map\":{\"a\":\"b\",\"c\":\"d\", \"e\": {\"f\": \"g\", \"h\": \"i\"}}}").unwrap();
        assert_eq!(d.translate("a").unwrap(), "b");
        assert_eq!(d.translate("c").unwrap(), "d");
        assert_eq!(d.translate("e.f").unwrap(), "g");
        assert_eq!(d.translate("e.h").unwrap(), "i");
    }

    #[test]
    fn test_cascade() {
        let d1 = from_json_str("{\"lang\":\"ja\", \"delimiter\": \".\", \"map\":{\"a\":\"b\"}}").unwrap();
        let d2 = from_json_str("{\"lang\":\"en\", \"delimiter\": \".\", \"map\":{\"a\":\"c\", \"x\":\"y\"}}").unwrap();
        let ds = vec![d1, d2];
        let c = Cascade{dicts: ds};
        let t = |key| c.translate(key).unwrap();
        assert_eq!(c.translate("a").unwrap(), "b");
        assert_eq!(c.translate("x").unwrap(), "y");
        assert_eq!(t("a"), "b");
        assert_eq!(t("x"), "y");
    }

    #[test]
    fn test_cascade_from_dir() {
        let tmpdir = tempdir::TempDir::new("test").unwrap();
        let s1 = "{\"lang\":\"ja\", \"delimiter\": \".\", \"map\":{\"a\":\"b\"}}";
        let s2 = "{\"lang\":\"en\", \"delimiter\": \".\", \"map\":{\"a\":\"c\", \"x\":\"y\"}}";
        let mut f1 = File::create(tmpdir.path().join("1.json")).unwrap();
        let mut f2 = File::create(tmpdir.path().join("2.json")).unwrap();
        let _ = f1.write_all(s1.as_bytes());
        let _ = f2.write_all(s2.as_bytes());

        let c = Cascade::from_dirpath(&tmpdir.path().to_str().unwrap()).unwrap();
        let t = |key| c.translate(key).unwrap();
        assert_eq!(c.translate("a").unwrap(), "b");
        assert_eq!(c.translate("x").unwrap(), "y");
        assert_eq!(t("a"), "b");
        assert_eq!(t("x"), "y");

        tmpdir.close().unwrap();
    }
}
