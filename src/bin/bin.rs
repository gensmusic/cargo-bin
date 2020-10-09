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
    /// Create a new binary in current folder. Use --force to override if binary already exists.
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
    /// Add existing binaries to Cargo.toml, abort if binary doesn't exists.
    Add {
        /// Binary path or name
        #[structopt()]
        path: String,

        /// force create to override existing binary file
        #[structopt(short = "f", long)]
        force: bool,
    },
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

    let root_path = project::root_path()?;
    if opt.verbose {
        println!("root_path: {:?}", root_path);
    }

    match opt.cmd {
        Command::Add { path, force } => {
            let bin_path = get_bin_path(path)?;
            add_binaries(AddArgs {
                bin_path,
                force,
                root_path,
                dry_run: opt.dry_run,
                verbose: opt.verbose,
            })?;
        }
        Command::New {
            path,
            assume_yes: _,
            force,
        } => {
            new_binary(NewBinaryArgs {
                root_path,
                path,
                force,
                dry_run: opt.dry_run,
                verbose: opt.verbose,
            })?;
        }
        Command::Remove {} => {}
        Command::Tidy {} => {
            tide_binaries(TideArgs {
                root_path,
                dry_run: opt.dry_run,
                verbose: opt.verbose,
            })?;
        }
    }

    Ok(())
}

struct NewBinaryArgs {
    path: String, // binary path
    force: bool,
    root_path: PathBuf,
    dry_run: bool,
    verbose: bool,
}

fn new_binary(args: NewBinaryArgs) -> Result<()> {
    let bin_path = get_bin_path(args.path.clone())?;
    if bin_path.exists() {
        ensure!(
            bin_path.is_file(),
            "{:?} already exits and is not a file",
            args.path
        );
        if !args.force {
            bail!("{:?} already exists, use --force to override it", args.path);
        }
    }

    println!("create {:?}", bin_path);
    if !args.dry_run {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(bin_path.clone())
            .with_context(|| format!("open file {:?} err", bin_path))?;
        let content = r#"
fn main() {
  println!("hello world");
}"#;
        file.write_all(content.as_bytes())
            .with_context(|| format!("write to {:?} err", bin_path))?;
    }

    // TODO only add the new one
    add_binaries(AddArgs {
        bin_path,
        force: args.force,
        root_path: args.root_path,
        dry_run: args.dry_run,
        verbose: args.verbose,
    })?;

    Ok(())
}

pub struct AddArgs {
    bin_path: PathBuf,
    root_path: PathBuf,
    force: bool,
    dry_run: bool,
    verbose: bool,
}

/// won't check if path is valid
fn add_binaries(args: AddArgs) -> Result<()> {
    let mut manifest = Manifest::new()?;

    let BinInfo { name, path } = get_bin_info(&args.bin_path, args.root_path.to_str().unwrap())?;

    if manifest.exists(&name, &path) {
        if !args.force {
            println!(
                "same name {:?} or path {:?} already exists, use --force to override",
                name, path
            );
            return Ok(());
        }

        if args.verbose {
            println!(
                "same name {:?} or path {:?} already exists, override!",
                name, path
            );
        }
    }

    println!("add bin: {:?} -> {:?}", name, path);
    manifest.add_bin(&name, &path)?;

    if !args.dry_run {
        manifest.write()?;
    }

    Ok(())
}

struct TideArgs {
    root_path: PathBuf,
    dry_run: bool,
    verbose: bool,
}

fn tide_binaries(args: TideArgs) -> Result<()> {
    let mut manifest = Manifest::new()?;

    // add the new main files
    let main_files = project::find_main_file(&args.root_path)?;
    for entry in main_files.iter() {
        let BinInfo { name, path } = get_bin_info(entry, args.root_path.to_str().unwrap())?;

        if manifest.exists(&name, &path) {
            if args.verbose {
                println!("bin {}: {} already exists, skip", name, path)
            }
            continue;
        }

        println!("add new bin: name: {:?}, path: {:?},", name, path);
        manifest.add_bin(&name, &path)?;
    }

    // check existing bins

    // write the changes
    if !args.dry_run {
        manifest.write()?;
    }

    Ok(())
}

// utils

fn get_bin_path(path: String) -> Result<PathBuf> {
    let mut path = path;
    ensure!(!path.is_empty(), "path cannot be empty");
    if !path.ends_with(".rs") {
        path.push_str(".rs");
    }

    let path = Path::new(&path);
    Ok(path.to_path_buf())
}

struct BinInfo {
    name: String,
    path: String,
}

fn get_bin_info(bin_path: &Path, root_path: &str) -> Result<BinInfo> {
    let mut root_path = root_path.to_string();
    if !root_path.ends_with('/') {
        root_path.push('/');
    }

    // convert to absolute path first
    let bin_path = fs::canonicalize(bin_path).with_context(|| format!("{:?}", bin_path))?;

    // path remove root path
    let path = bin_path
        .to_str()
        .and_then(|v| v.strip_prefix(&root_path))
        .with_context(|| format!("get bin path err from: {:?}", bin_path))?;

    // name, remove src if it's under src folder
    let name = bin_path
        .to_str()
        .and_then(|v| v.strip_prefix(&root_path))
        .and_then(|v| {
            for prefix in ["src/", "src"].iter() {
                if v.starts_with(*prefix) {
                    return v.strip_prefix(*prefix);
                }
            }
            Some(v)
        })
        .and_then(|v| v.strip_suffix(".rs"))
        .map(|v| v.replace("/", "-"))
        .with_context(|| format!("get bin name err, from: {:?}", bin_path))?;

    Ok(BinInfo {
        name,
        path: path.to_string(),
    })
}
