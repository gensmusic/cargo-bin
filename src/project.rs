use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use syn::Item;

const CARGO_TOML: &str = "Cargo.toml";

/// return cargo project root path (absolute path)
pub fn root_path() -> Result<PathBuf> {
    let manifest = search_manifest()?;
    let root = manifest
        .parent()
        .with_context(|| format!("{:?} has no parent", manifest))?;
    // to absolute path
    fs::canonicalize(root).context("root path convert to absolute err")
}

/// search from current_dir() with name Cargo.toml
pub fn search_manifest() -> Result<PathBuf> {
    search_manifest_from(
        &env::current_dir().context("get current dir err")?,
        CARGO_TOML,
    )
}

/// search manifest from dir with specified filename (absolute path)
pub fn search_manifest_from(start_dir: &PathBuf, file_name: &str) -> Result<PathBuf> {
    let mut path = start_dir.as_path();
    loop {
        let toml = path.join(file_name);
        if toml.exists() {
            return fs::canonicalize(toml).context("manifest path convert to absolute path err");
        }

        path = path
            .parent()
            .with_context(|| format!("Cargo.toml not found search from: {:?}", start_dir))?
    }
}

/// find rust source file with main() from the specified dir
/// TODO for now ignore some folders like target, .git, .github
pub fn find_main_file(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut ignored_folders = HashSet::new();
    for folder in ["target", "src/bin", ".git", ".github"].iter() {
        ignored_folders.insert(dir.join(*folder));
    }
    let mut files = vec![];

    fn find(dir: &Path, files: &mut Vec<PathBuf>, ignored: &HashSet<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        if ignored.contains(&dir.to_path_buf()) {
            return Ok(());
        }
        for entry in fs::read_dir(dir).with_context(|| format!("read_dir err, dir: {:?}", dir))? {
            let entry = entry.with_context(|| "dir entry err")?;
            let path = entry.path();
            if path.is_dir() {
                find(&path, files, ignored)?;
                continue;
            }

            let ext = path
                .as_path()
                .extension()
                .map_or("", |v| v.to_str().unwrap_or(""));
            if ext != "rs" {
                continue;
            }

            if contains_main(&path)? {
                files.push(path)
            }
        }
        Ok(())
    }

    find(dir, &mut files, &ignored_folders)?;

    Ok(files)
}

// parse file and see if the file contains fn main()
fn contains_main(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path).with_context(|| format!("open file {:?} err", path))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .with_context(|| format!("read file {:?} err", path))?;

    let ast = syn::parse_file(&content)?;

    let is_main = ast.items.iter().any(|v| match v {
        Item::Fn(item) => item.sig.ident == "main",
        _ => false,
    });

    Ok(is_main)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search() {
        let dir = env::current_dir().unwrap().join("misc");
        let file_path = search_manifest_from(&dir, "test-cargo.toml").expect("search should be ok");
        println!("file_path: {:?}", file_path);
    }
}
