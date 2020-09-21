use clap::{crate_authors, crate_description, crate_version, App, Arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Chum Bucket")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("include")
                .about("Target file is an include file")
                .short('i'),
        )
        .arg(
            Arg::with_name("rebuild-history")
                .about("Rebuild versioning history from the start. Will not read existing bundle for versioning.")
                .long("rebuild-history")
                .required(false),
        )
        .arg(
            // By default, it will output to a path relative to the chumbucket
            Arg::with_name("output")
                .about("Location to output bundle to")
                .short('o')
                .required(true),
        )
        .arg(
            Arg::with_name("file")
                .about("Input file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let fs_content = std::fs::read(matches.value_of("file").unwrap())?;

    let fs_out = matches.value_of("output").unwrap();

    let input = std::str::from_utf8(&fs_content)?;
    
    // Supercede and process singular include only
    if matches.is_present("include") {
        let res = alternator::consume("chumbucket", input).await?;

        write_to_disk(fs_out, res)?;

        return Ok(());
    }

    // Remaining manifest

    Ok(())
}

fn write_to_disk<T>(loc: &str, t: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let s = serde_json::to_string(&t)?;

    std::fs::write(loc, &s)?;

    Ok(())
}
