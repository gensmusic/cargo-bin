use anyhow::{Context, Result};
use cargo_bin::manifest::Manifest;
use cargo_bin::project;

fn main() -> Result<()> {
    let mut manifest = Manifest::new()?;

    let root_path = project::root_path()?;
    let src_path = root_path.join("src");
    let main_files = project::find_main_file(&src_path)?;

    for entry in main_files.iter() {
        let path = entry
            .as_path()
            .strip_prefix(&root_path)
            .with_context(|| format!("path: {:?} strip prefix err", entry))?;
        let name = path
            .to_str()
            .unwrap()
            .strip_suffix(".rs")
            .unwrap()
            .strip_prefix("src/")
            .unwrap()
            .replace("/", "-");
        println!("add bin: name: {:?}, path: {:?},", name, path);
        manifest.add_bin(&name, path.to_str().unwrap())?;
    }

    manifest.write()?;

    Ok(())
}
