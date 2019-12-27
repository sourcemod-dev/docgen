use std::path::PathBuf;

use structopt::StructOpt;

use futures::executor::block_on;

mod errors;
mod parser;
mod include;
mod manifest;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    /// Optional output path of generated include documentation.
    /// 
    /// For include mode, this should be the desired path to the output file
    /// For manifest mode, this should be the desired path to the output directory
    pub output: Option<PathBuf>,

    #[structopt(short = "i", long = "include")]
    /// Parses an include (.inc) file
    pub is_include: bool,

    #[structopt(short = "m", long = "manifest")]
    /// Parses a manifest (json) file
    pub is_manifest: bool,

    #[structopt(long, default_value = "https://cdn.jsdelivr.net/gh/rumblefrog/sp-gid@1")]
    /// Base URL w/o trailing slash to use in generated manifest file
    pub base_url: String,

    #[structopt(name = "file", parse(from_os_str))]
    /// Input path of an include or a manifest file to parse.
    pub input: PathBuf,
}

fn main() {
    let mut cli = Cli::from_args();

    if cli.is_include && cli.is_manifest {
        println!("Cannot be in both include and manifest mode!");
        return;
    }

    if !cli.is_include && !cli.is_manifest {
        println!("Defaulting to include mode");
        cli.is_include = true;
    }

    if cli.is_include {
        block_on(include::process_include(cli.input, cli.output));
    } else {
        if let Some(output_some) = &cli.output {
            if !output_some.is_dir() {
                println!("Output provided but is not a directory");
                return;
            }
        }

        block_on(manifest::process_manifest(cli.input, cli.output, cli.base_url));
    }
}
