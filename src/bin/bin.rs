use anyhow::Result;
use cargo_bin::manifest::Manifest;

fn main() -> Result<()> {
    let mut manifest = Manifest::new()?;
    // println!("manifest path: {:?}", manifest);

    manifest.add_bin("bin1", "src/b1.rs").unwrap();
    manifest.add_bin("bin2", "src/b2.rs").unwrap();
    manifest.add_bin("bin3", "src/b3.rs").unwrap();
    manifest.add_bin("bin1", "src/2/b1.rs").unwrap();

    println!("{}", "-".repeat(20));
    println!("{}", manifest.to_string());

    Ok(())
}
