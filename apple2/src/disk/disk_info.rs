use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum WozVersion {
    #[default]
    Unknown,
    Dsk, Woz1, Woz2,
}

#[derive(Clone, Debug, Default)]
pub struct DiskInfo {
    pub(crate) name: Option<String>,
    pub(crate) woz_version: WozVersion,
    pub(crate) path: String,
    pub(crate) map: HashMap<String, String>,
    pub(crate) is_write_protected: bool,
}

impl Display for DiskInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} {}", if self.name.is_none() { "(no name)" } else { "name:" },
            self.name())).unwrap();
        Ok(())
    }
}

impl DiskInfo {
    pub fn new(name: &str, path: &str) -> Self {
        Self {
            name: Some(name.to_string()), path: path.to_string(), map: HashMap::new(),
            woz_version: WozVersion::Unknown, is_write_protected: true,
        }
    }

    pub fn new2(name: Option<String>, path: &str, map: HashMap<String, String>, disk_type: WozVersion,
            is_write_protected: bool)
            -> Self {
        Self {
            name, path: path.to_string(), map,
            woz_version: disk_type, is_write_protected,
        }
    }

    pub fn n(path: &str) -> Self {
        Self { name: None, path: path.to_string(), map: HashMap::default(),
            woz_version: WozVersion::Unknown, is_write_protected: true, }
    }

    pub fn path(&self) -> String { self.path.to_string() }

    pub fn name(&self) -> &str {
        if let Some(n) = &self.name { n }
        else if let Some(n) = self.map.get("title") { n }
        else {
            match Path::new(&self.path).iter().last() {
                None => { "" }
                Some(n) => { n.to_str().unwrap() }
            }
        }
    }

    pub fn side(&self) -> Option<String> {
        self.map.get("side").cloned()
    }
}
