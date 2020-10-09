use crate::project::search_manifest;
use anyhow::{bail, ensure, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, Document, Item, Table};

const KEY_BIN: &str = "bin";
const KEY_BIN_NAME: &str = "name";
const KEY_BIN_PATH: &str = "path";

#[derive(Debug)]
pub struct Manifest {
    root: Document,
    path: PathBuf,
}

impl Manifest {
    /// Create a Manifest with searching Cargo.toml from current path.
    pub fn new() -> Result<Self> {
        let path = search_manifest()?;
        Self::open(&path)
    }

    // TODO pub ?
    fn open(path: &Path) -> Result<Self> {
        let file_content = fs::read_to_string(path)
            .with_context(|| format!("read toml file err, path: {:?}", path))?;
        let mut doc = file_content
            .parse::<Document>()
            .with_context(|| format!("parse toml file err, path: {:?}", path))?;

        // make sure bin is initialized
        let item = &doc[KEY_BIN];
        match item {
            Item::ArrayOfTables(_) => {
                // already exists bin
            }
            Item::None => {
                doc[KEY_BIN] = Item::ArrayOfTables(ArrayOfTables::default());
            }
            _ => bail!("bin should be type ArrayOfTables instead of {:?}", item),
        }

        Ok(Self {
            root: doc,
            path: path.to_path_buf(),
        })
    }

    fn bins(&self) -> &ArrayOfTables {
        let item = &self.root[KEY_BIN];
        match item {
            Item::ArrayOfTables(v) => v,
            _ => panic!("bin should be type ArrayOfTables instead of {:?}", item),
        }
    }
    fn bins_mut(&mut self) -> &mut ArrayOfTables {
        let item = &mut self.root[KEY_BIN];
        match item {
            Item::ArrayOfTables(v) => v,
            _ => panic!("bin should be type ArrayOfTables instead of {:?}", item),
        }
    }

    /// Check if same binary exists.
    /// exists means name or path is equal to some existed ones.
    pub fn exists(&self, name: &str, path: &str) -> bool {
        self.find_bin(name, path).is_some()
    }

    /// Find a bin's index within ArrayTable, cannot use ArrayTable's iter()
    /// because there is filter in it.
    fn find_bin(&self, name: &str, path: &str) -> Option<usize> {
        let bins = self.bins();
        for i in 0..bins.len() {
            if let Some(item) = bins.get(i) {
                if let Some(v) = item[KEY_BIN_NAME].as_str() {
                    if v == name {
                        return Some(i);
                    }
                }
                if let Some(v) = item[KEY_BIN_PATH].as_str() {
                    if v == path {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Add a bin, only support name and path for now.
    /// If a bin with same name or path already exists, will remove it first
    /// then add the new one.
    ///  About Cargo.toml bin, see cargo book: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#configuring-a-target
    pub fn add_bin(&mut self, name: &str, path: &str) -> Result<()> {
        ensure!(!name.is_empty(), "bin.name cannot be empty");
        ensure!(!path.is_empty(), "bin.path cannot be empty");

        // remove first
        self.remove_bin(name, path);

        // append new bin
        let mut table = Table::default();
        table[KEY_BIN_NAME] = value(name);
        table[KEY_BIN_PATH] = value(path);
        self.bins_mut().append(table);

        Ok(())
    }

    /// Remove a bin from manifest. Return true if found and delete.
    pub fn remove_bin(&mut self, name: &str, path: &str) -> bool {
        match self.find_bin(name, path) {
            Some(index) => {
                self.bins_mut().remove(index);
                true
            }
            None => false,
        }
    }

    /// Write changes to manifest file
    pub fn write(&self) -> Result<()> {
        fs::write(&self.path, self.root.to_string_in_original_order())?;
        Ok(())
    }
}

impl ToString for Manifest {
    fn to_string(&self) -> String {
        self.root.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::*;
    use anyhow::{Context, Result};

    fn new_empty_manifest() -> Manifest {
        let mut root = Document::new();
        root[KEY_BIN] = Item::ArrayOfTables(ArrayOfTables::default());
        Manifest {
            root,
            path: PathBuf::new(),
        }
    }

    #[test]
    fn open_manifest() -> Result<()> {
        let dir = std::env::current_dir()
            .context("get_current_dir error")?
            .join("misc");
        let file_path = search_manifest_from(&dir, "test-cargo.toml")?;
        let manifest = Manifest::open(&file_path)?;

        assert!(matches!(manifest.root[KEY_BIN], Item::ArrayOfTables(_)));

        Ok(())
    }

    #[test]
    fn add_bin() -> Result<()> {
        let mut manifest = new_empty_manifest();
        manifest.add_bin("bin1", "src/b1.rs")?;
        manifest.add_bin("bin2", "src/b2.rs")?;
        manifest.add_bin("bin3", "src/b3.rs")?;
        manifest.add_bin("bin1", "src/2/b1.rs")?;

        let expected = r#"[[bin]]
name = "bin2"
path = "src/b2.rs"
[[bin]]
name = "bin3"
path = "src/b3.rs"
[[bin]]
name = "bin1"
path = "src/2/b1.rs"
"#;
        assert_eq!(expected, manifest.to_string());

        Ok(())
    }

    #[test]
    fn get_bins() -> Result<()> {
        let mut manifest = new_empty_manifest();
        assert_eq!(0, manifest.bins().len());
        manifest.add_bin("bin1", "src/b1.rs")?;
        assert_eq!(1, manifest.bins().len());
        manifest.add_bin("bin2", "src/b2.rs")?;
        assert_eq!(2, manifest.bins().len());

        Ok(())
    }

    #[test]
    fn bin_exists() -> Result<()> {
        let mut manifest = new_empty_manifest();
        assert!(!manifest.exists("bin1", "src/b1.rs"));

        manifest.add_bin("bin1", "src/b1.rs")?;
        assert!(manifest.exists("bin1", ""));
        assert!(manifest.exists("", "src/b1.rs"));
        Ok(())
    }

    #[test]
    fn find_bin() {
        let mut manifest = new_empty_manifest();
        let index = manifest.find_bin("bin1", "src/b1.rs");
        assert!(index.is_none());

        manifest.add_bin("bin1", "src/b1.rs").unwrap();
        assert_eq!(manifest.find_bin("bin1", "").unwrap(), 0);
        assert_eq!(manifest.find_bin("", "src/b1.rs").unwrap(), 0);
    }
}
