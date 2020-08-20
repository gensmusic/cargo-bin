use anyhow::{bail, ensure, Context, Result};
use cargo_bin::manifest::Manifest;
use cargo_bin::project;
use std::fs;
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
enum Command {
    /// Create a new binary in current folder.
    New {
        /// Binary path or name.
        #[structopt()]
        path: String,

        /// assume all answers are yes
        #[structopt(short = "y")]
        assume_yes: bool,

        /// force create to override existing binary file
        #[structopt(short = "f", long)]
        force: bool,
    },
    /// Add missing and remove unused
    Tidy {},
    /// Remove binary
    Remove {},
    /// Add binaries
    Add {},
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-bin")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,

    /// verbose
    #[structopt(short, long, global = true)]
    verbose: bool,

    /// dry run
    #[structopt(long, global = true)]
    dry_run: bool,
}

fn main() -> Result<()> {
    // in case we are invoked by cargo-bin
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "bin" {
        args.remove(1);
    }
    let opt = Opt::from_clap(&Opt::clap().get_matches_from(args));
    if opt.verbose {
        println!("{:?}", opt);
    }

    match opt.cmd {
        Command::Add {} => {
            add_binaries(opt.dry_run)?;
        }
        Command::New {
            path,
            assume_yes: _,
            force,
        } => {
            new_binary(path, force, opt.dry_run)?;
        }
        Command::Remove {} => {}
        Command::Tidy {} => {}
    }

    Ok(())
}

fn new_binary(path: String, force: bool, dry_run: bool) -> Result<()> {
    ensure!(!path.is_empty(), "path cannot be empty");

    let mut path = path;
    if !path.ends_with(".rs") {
        path.push_str(".rs");
    }

    let path = Path::new(&path);
    if path.exists() {
        ensure!(path.is_file(), "{:?} already exits and is not a file", path);
        if !force {
            bail!("{:?} already exists, use --force to override it", path);
        }
    }

    println!("create {:?}", path);
    if !dry_run {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("open file {:?} err", path))?;
        let content = r#"
fn main() {
  println!("hello world");
}"#;
        file.write_all(content.as_bytes())
            .with_context(|| format!("write to {:?} err", path))?;
    }

    // TODO only add the new one
    add_binaries(dry_run)?;

    Ok(())
}

fn add_binaries(dry_run: bool) -> Result<()> {
    let mut manifest = Manifest::new()?;

    let root_path = project::root_path()?;
    let src_path = root_path.join("src");
    // TODO verbose print search path
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
        if !dry_run {
            manifest.add_bin(&name, path.to_str().unwrap())?;
        }
    }

    manifest.write()?;

    Ok(())
}
