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
    /// create Manifest with searching Cargo.toml from current path.
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
            Item::ArrayOfTables(v) => {
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

    /// check if same binary exists.
    /// exists means name or path is equal to some existed ones.
    pub fn exists(&self, name: &str, path: &str) -> bool {
        for bin_table in self.bins().iter() {
            if let Some(v) = bin_table[KEY_BIN_NAME].as_str() {
                if v == name {
                    return true;
                }
            }
            if let Some(v) = bin_table[KEY_BIN_PATH].as_str() {
                if v == path {
                    return true;
                }
            }
        }
        false
    }

    /// add bin, only support name and path for now
    /// see cargo book: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#configuring-a-target
    pub fn add_bin(&mut self, name: &str, path: &str) -> Result<()> {
        ensure!(!name.is_empty(), "bin.name cannot be empty");
        ensure!(!path.is_empty(), "bin.path cannot be empty");

        let bins = self.bins_mut();

        // remove the same name or path
        let keys = vec![(KEY_BIN_NAME, name), (KEY_BIN_PATH, path)];
        let mut to_removed = vec![];
        for i in 0..bins.len() {
            let table = bins
                .get_mut(i)
                .with_context(|| format!("array of tables should exists at index {:?}", i))?;
            for (key, val) in keys.iter() {
                let field = &table[*key];
                ensure!(
                    field.is_str(),
                    "{} should be type str instead of {:?}",
                    *key,
                    field
                );
                if field.as_str().unwrap() == *val {
                    to_removed.push(i);
                    break;
                }
            }
        }
        to_removed.iter().for_each(|&i| bins.remove(i));

        // append new bin
        let mut table = Table::default();
        table[KEY_BIN_NAME] = value(name);
        table[KEY_BIN_PATH] = value(path);
        bins.append(table);

        Ok(())
    }

    /// write changes to manifest file
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
        manifest.add_bin("bin1", "src/b1.rs");
        assert_eq!(1, manifest.bins().len());
        manifest.add_bin("bin2", "src/b2.rs");
        assert_eq!(2, manifest.bins().len());

        Ok(())
    }

    #[test]
    fn bin_exists() -> Result<()> {
        let mut manifest = new_empty_manifest();
        assert!(!manifest.exists("bin1", "src/b1.rs"));
        manifest.add_bin("bin1", "src/b1.rs");
        assert!(manifest.exists("bin1", ""));
        assert!(manifest.exists("", "src/b1.rs"));
        Ok(())
    }
}
