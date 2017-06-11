#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate toml;

#[cfg(test)]
extern crate tempdir;

use std::path;
use std::error;
use std::fs;

pub mod dict;
pub use dict::Dictionary;

pub struct Cascade {
    pub dicts: Vec<Dictionary>,
}

impl Cascade {
    pub fn with_filepath_strs(files: &Vec<&str>, delimiter: &str) -> Result<Cascade, Box<error::Error>> {
        let mut ds: Vec<Dictionary> = Vec::new();
        for f in files {
            if let Some(ext) = path::Path::new(f).extension() {
                if ext == "json" {
                    ds.push(try!(Dictionary::with_json_filepath(f, delimiter)));
                    continue;
                }else if ext == "toml" {
                    ds.push(try!(Dictionary::with_toml_filepath(f, delimiter)));
                    continue;
                }
            }

            // Try to read it as json if the file type couldn't be guessed from its extension.
            ds.push(try!(Dictionary::with_json_filepath(f, delimiter)));
        }
        Ok(Self{dicts: ds})
    }

    pub fn with_dirpath<P: AsRef<path::Path>>(dir: P, delimiter: &str) -> Result<Cascade, Box<error::Error>> {
        let rd = try!(fs::read_dir(dir));
        let mut v:Vec<Box<String>> = Vec::new();
        for f in rd {
            if let Some(pathstr) = try!(f).path().to_str() {
                v.push(Box::new(pathstr.to_string()));
            }
        }
        return Self::with_filepath_strs((&v.iter().map(|s| &*s as &str).collect::<Vec<&str>>()), delimiter);
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
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_cascade() {
        let d1 = Dictionary::with_json_str("{\"name\":\"ja\", \"map\":{\"a\":\"b\"}}", ".").unwrap();
        let d2 = Dictionary::with_json_str("{\"name\":\"en\", \"map\":{\"a\":\"c\", \"x\":\"y\"}}", ".").unwrap();
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
        let s1 = "{\"name\":\"ja\", \"map\":{\"a\":\"b\"}}";
        let s2 = "{\"name\":\"en\", \"map\":{\"a\":\"c\", \"x\":\"y\"}}";
        let s3 = r#"
            name = "t"

            [map]
            a = "d"
            z = "Z"
        "#;

        let mut f1 = File::create(tmpdir.path().join("1.json")).unwrap();
        let mut f2 = File::create(tmpdir.path().join("2.json")).unwrap();
        let mut f3 = File::create(tmpdir.path().join("3.toml")).unwrap();
        let _ = f1.write_all(s1.as_bytes());
        let _ = f2.write_all(s2.as_bytes());
        let _ = f3.write_all(s3.as_bytes());

        let c = Cascade::with_dirpath(&tmpdir.path().to_str().unwrap(), ".").unwrap();
        let t = |key| c.translate(key).unwrap();
        assert_eq!(c.translate("a").unwrap(), "b");
        assert_eq!(c.translate("x").unwrap(), "y");
        assert_eq!(c.translate("z").unwrap(), "Z");
        assert_eq!(t("a"), "b");
        assert_eq!(t("x"), "y");
        assert_eq!(t("z"), "Z");

        tmpdir.close().unwrap();
    }
}
