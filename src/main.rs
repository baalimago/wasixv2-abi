use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use wasixv2_codegen::{GeneratedFile, Target};

/// Generates wasixv2 bindings from the annotated WIT definitions.
///
/// Run from the consumer's repository root: the `--wit` path is recorded (relative) in the output
/// header so its up-to-date tests can regenerate from the same source.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

/// Where a Rust backend reads from and writes to.
#[derive(clap::Args)]
struct RustArgs {
    /// WIT file or directory (relative to the current directory).
    #[arg(long)]
    wit: PathBuf,
    /// Output `.rs` file.
    #[arg(long)]
    out: PathBuf,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate the `wasixv2-abi` crate (wasmer: `lib/wasixv2/abi/src/generated.rs`).
    Abi(RustArgs),
    /// Generate the `Syscalls` trait (wasmer: `lib/wasixv2/syscalls/src/generated.rs`).
    Syscalls(RustArgs),
    /// Generate the import shims (wasmer: `lib/wasixv2/exec/src/generated.rs`).
    Exec(RustArgs),
    /// Generate the libc2 (musl) syscall glue.
    #[command(group(clap::ArgGroup::new("dest").required(true)))]
    C {
        /// WIT file or directory.
        #[arg(long)]
        wit: PathBuf,
        /// libc2 checkout: overwrites the generated files in place.
        #[arg(long, group = "dest")]
        libc: Option<PathBuf>,
        /// Directory to write the files to, mirroring the libc2 tree layout.
        #[arg(long, group = "dest")]
        out_dir: Option<PathBuf>,
    },
    /// Generate the Go `wasix/wasm` syscall glue.
    #[command(group(clap::ArgGroup::new("dest").required(true)))]
    Go {
        /// WIT file or directory.
        #[arg(long)]
        wit: PathBuf,
        /// GOROOT of a Go checkout with the wasix port: overwrites the generated files in place.
        #[arg(long, group = "dest")]
        goroot: Option<PathBuf>,
        /// Directory to write the files to, mirroring the GOROOT layout.
        #[arg(long, group = "dest")]
        out_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Cmd::Abi(args) => write_rust(args, wasixv2_codegen::generate_abi),
        Cmd::Syscalls(args) => write_rust(args, wasixv2_codegen::generate_syscalls),
        Cmd::Exec(args) => write_rust(args, wasixv2_codegen::generate_exec),
        Cmd::C { wit, libc, out_dir } => {
            let files = wasixv2_codegen::generate_c(&wit, Target::WASM32)?;
            let dest = Dest::new(libc, out_dir, "libc2", "arch/wasm32/syscall_arch.h")?;
            write_files(dest, files)
        }
        Cmd::Go {
            wit,
            goroot,
            out_dir,
        } => {
            let files = wasixv2_codegen::generate_go(&wit)?;
            let marker = "src/internal/runtime/syscall/linux/syscall_wasix_wasm.go";
            write_files(Dest::new(goroot, out_dir, "Go wasix", marker)?, files)
        }
    }
}

/// Where a consumer's files go: its checkout, or a directory mirroring its layout.
enum Dest {
    Checkout(PathBuf),
    OutDir(PathBuf),
}

impl Dest {
    fn new(
        checkout: Option<PathBuf>,
        out_dir: Option<PathBuf>,
        what: &str,
        marker: &str,
    ) -> Result<Self> {
        match (checkout, out_dir) {
            (Some(root), _) => {
                // Catch a wrong path before scattering files into it.
                if !root.join(marker).is_file() {
                    bail!("{} does not look like a {what} checkout", root.display());
                }
                Ok(Dest::Checkout(root))
            }
            (None, Some(dir)) => Ok(Dest::OutDir(dir)),
            (None, None) => unreachable!("clap requires one destination"),
        }
    }
}

fn write_files(dest: Dest, files: Vec<GeneratedFile>) -> Result<()> {
    let (root, create) = match dest {
        Dest::Checkout(root) => (root, false),
        Dest::OutDir(dir) => (dir, true),
    };
    for f in files {
        let path = root.join(f.path);
        if create && let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&path, f.contents).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn write_rust(args: RustArgs, generate: fn(&Path, &str, Target) -> Result<String>) -> Result<()> {
    let source = source_label(&args.wit)?;
    let code = generate(&args.wit, &source, Target::WASM32)?;
    std::fs::write(&args.out, code).with_context(|| format!("writing {}", args.out.display()))
}

fn source_label(wit: &Path) -> Result<String> {
    if wit.is_absolute() {
        bail!("--wit must be relative to the repository root");
    }
    let parts: Vec<_> = wit.iter().map(|c| c.to_string_lossy()).collect();
    Ok(parts.join("/"))
}
