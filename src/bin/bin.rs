use anyhow::{bail, ensure, Context, Result};
use cargo_bin::manifest::Manifest;
use cargo_bin::project;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
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

        /// search main from root path, default is src
        #[structopt(long = "from_root")]
        from_root: bool,
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
            add_binaries(AddArgs {
                from_root: false, // TODO
                dry_run: opt.dry_run,
                verbose: opt.verbose,
            })?;
        }
        Command::New {
            path,
            assume_yes: _,
            force,
            from_root,
        } => {
            new_binary(NewBinaryArgs {
                path,
                force,
                from_root,
                dry_run: opt.dry_run,
                verbose: opt.verbose,
            })?;
        }
        Command::Remove {} => {}
        Command::Tidy {} => {
            tide_binaries(opt.dry_run)?;
        }
    }

    Ok(())
}

struct NewBinaryArgs {
    path: String,
    force: bool,
    from_root: bool,
    dry_run: bool,
    verbose: bool,
}

fn new_binary(args: NewBinaryArgs) -> Result<()> {
    let mut path = args.path.clone();
    ensure!(!path.is_empty(), "path cannot be empty");
    if !path.ends_with(".rs") {
        path.push_str(".rs");
    }

    let path = Path::new(&path);
    if path.exists() {
        ensure!(path.is_file(), "{:?} already exits and is not a file", path);
        if !args.force {
            bail!("{:?} already exists, use --force to override it", path);
        }
    }

    println!("create {:?}", path);
    if !args.dry_run {
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

    add_binaries(AddArgs {
        from_root: args.from_root,
        dry_run: args.dry_run,
        verbose: args.verbose,
    })?;

    Ok(())
}

pub struct AddArgs {
    from_root: bool,
    dry_run: bool,
    verbose: bool,
}

fn add_binaries(args: AddArgs) -> Result<()> {
    let mut manifest = Manifest::new()?;

    let root_path = project::root_path()?;
    let src_path = root_path.join("src");
    let search_path = if args.from_root {
        root_path.clone()
    } else {
        src_path.clone()
    };
    if args.verbose {
        println!("search_path: {:?}", search_path);
    }
    // TODO search path must in root path
    let main_files = project::find_main_file(&search_path)?;

    for entry in main_files.iter() {
        let path = entry
            .as_path()
            .strip_prefix(&root_path)
            .with_context(|| format!("path: {:?} strip prefix err", entry))?;
        let mut name = path.to_str().unwrap().strip_suffix(".rs").unwrap();
        if !args.from_root {
            name = name.strip_prefix("src/").unwrap();
        }
        let name = name.replace("/", "-");

        println!("add bin: name: {:?}, path: {:?},", name, path);
        if !args.dry_run {
            manifest.add_bin(&name, path.to_str().unwrap())?;
        }
    }

    manifest.write()?;

    Ok(())
}

fn tide_binaries(dry_run: bool) -> Result<()> {
    Ok(())
}
