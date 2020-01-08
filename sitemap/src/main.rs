use std::path::PathBuf;

use std::fs::{read_dir, read_to_string, write};

use serde_json::from_str;
use std::io::Result as IOResult;

use treexml::{Document, ElementBuilder};

use structopt::StructOpt;

mod builder;

#[derive(StructOpt)]
struct Cli {
    #[structopt(name = "directory", parse(from_os_str))]
    directory: PathBuf,

    #[structopt(long, default_value = "https://sm.rumblefrog.me/#")]
    /// Base URL w/o trailing slash to use in sitemap
    pub base_url: String,
}

fn main() -> IOResult<()> {
    let cli = Cli::from_args();

    if cli.directory.is_dir() {
        let mut entries: Vec<ElementBuilder> = read_dir(&cli.directory)?
            .into_iter()
            .filter(|r| r.is_ok())
            .map(|p| p.unwrap().path())
            .filter(|f| f.file_stem().unwrap() != "include")
            .flat_map(|path| {
                builder::build_include_tree(
                    path.file_stem().unwrap().to_string_lossy().to_string(),
                    from_str(&read_to_string(path).unwrap()).unwrap(),
                    &cli.base_url,
                )
            })
            .collect::<Vec<ElementBuilder>>();

        let mut interior_mut = Vec::with_capacity(entries.len());

        for entry in &mut entries {
            interior_mut.push(entry);
        }

        let doc = Document::build(
            &mut ElementBuilder::new("urlset")
                .attr("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
                .children(interior_mut),
        );

        write("sitemap.xml", doc.to_string())?;
    }

    Ok(())
}
