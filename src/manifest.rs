use anyhow::{bail, ensure, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, Document, Item, Table};

const CARGO_TOML: &str = "Cargo.toml";

#[derive(Debug)]
pub struct Manifest {
    root: Document,
}

impl Manifest {
    /// create Manifest with searching Cargo.toml from current path.
    pub fn new() -> Result<Self> {
        let path = search_cargo_toml(
            &env::current_dir().context("get current dir err")?,
            CARGO_TOML,
        )?;
        Self::open(path.as_path())
    }

    // TODO pub ?
    fn open(path: &Path) -> Result<Self> {
        let v = fs::read_to_string(path)
            .with_context(|| format!("read toml file err, path: {:?}", path))?;
        let doc = v
            .parse::<Document>()
            .with_context(|| format!("parse toml file err, path: {:?}", path))?;
        Ok(Self { root: doc })
    }

    /// add bin, only support name and path for now
    /// see cargo book: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#configuring-a-target
    pub fn add_bin(&mut self, name: &str, path: &str) -> Result<()> {
        ensure!(!name.is_empty(), "bin.name cannot be empty");
        ensure!(!path.is_empty(), "bin.path cannot be empty");

        const KEY_BIN: &str = "bin";
        const KEY_NAME: &str = "name";
        const KEY_PATH: &str = "path";

        if let Item::None = &self.root[KEY_BIN] {
            self.root[KEY_BIN] = Item::ArrayOfTables(ArrayOfTables::default());
        }

        let item = &mut self.root[KEY_BIN];
        let bins = match item {
            Item::ArrayOfTables(v) => v,
            _ => bail!("bin should be type ArrayOfTables instead of {:?}", item),
        };

        // remove the same name or path
        let keys = vec![(KEY_NAME, name), (KEY_PATH, path)];
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
        table[KEY_NAME] = value(name);
        table[KEY_PATH] = value(path);
        bins.append(table);

        Ok(())
    }
}

impl ToString for Manifest {
    fn to_string(&self) -> String {
        self.root.to_string()
    }
}

fn search_cargo_toml(start_dir: &PathBuf, file_name: &str) -> Result<PathBuf> {
    let mut path = start_dir.as_path();
    loop {
        let toml = path.join(file_name);
        if toml.exists() {
            return Ok(toml);
        }
        path = path
            .parent()
            .with_context(|| format!("Cargo.toml not found search from: {:?}", start_dir))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search() {
        let dir = env::current_dir().unwrap().join("misc");
        let file_path = search_cargo_toml(&dir, "test-cargo.toml").expect("search should be ok");
        println!("file_path: {:?}", file_path);
    }

    #[test]
    fn open_manifest() {
        let dir = env::current_dir().unwrap().join("misc");
        let file_path = search_cargo_toml(&dir, "test-cargo.toml").unwrap();
        let _manifest = Manifest::open(&file_path).unwrap();
    }

    #[test]
    fn add_bin() {
        let mut manifest = Manifest {
            root: Document::new(),
        };
        manifest.add_bin("bin1", "src/b1.rs").unwrap();
        manifest.add_bin("bin2", "src/b2.rs").unwrap();
        manifest.add_bin("bin3", "src/b3.rs").unwrap();
        manifest.add_bin("bin1", "src/2/b1.rs").unwrap();

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
    }
}
