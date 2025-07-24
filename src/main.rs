use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    println!("Logs from myGit app...");

    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
            println!("Git directory initialised");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "-p argument is required");
            let f = std::fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("open in .git/objects")?;

            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();
            z.read_until(0, &mut buf)
                .context("read header from .git/objects")?;

            let header =
                CStr::from_bytes_with_nul(&buf).expect("there is only 1 null and that too at end");
            let header = header
                .to_str()
                .context(".git/objects file header is not a valid UTF-8")?;

            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(
                    ".git/objects file header did not start with a known type : '{header}'"
                );
            };

            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("cannot print '{kind}'"),
            };

            let size = size
                .parse::<usize>()
                .context(".git/objects file header has invalid size: {size}")?;

            buf.clear();
            buf.resize(size, 0);
            z.read_exact(&mut buf[..])
                .context("read true contents of .git/objects file")?;
            let n = z
                .read(&mut [0])
                .context("validate EOF in .git/objects file")?;
            anyhow::ensure!(n == 0, ".git/objects file has {n} trailing bytes");

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            match kind {
                Kind::Blob => stdout
                    .write_all(&buf)
                    .context("write object contents to stdout")?,
            }
        }
    }

    Ok(())
}
