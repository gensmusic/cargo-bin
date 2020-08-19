use anyhow::Result;
use cargo_bin::manifest::Manifest;
use cargo_bin::project;

fn main() -> Result<()> {
    let mut manifest = Manifest::new()?;
    // println!("manifest path: {:?}", manifest);

    manifest.add_bin("bin1", "src/b1.rs").unwrap();
    manifest.add_bin("bin2", "src/b2.rs").unwrap();
    manifest.add_bin("bin3", "src/b3.rs").unwrap();
    manifest.add_bin("bin1", "src/2/b1.rs").unwrap();

    println!("{}", "-".repeat(20));
    println!("{}", manifest.to_string());

    println!("---find main------");
    let src_path = project::root_path()?.join("src");
    let main_files = project::find_main_file(&src_path)?;
    println!("{:?}", main_files);

    Ok(())
}
