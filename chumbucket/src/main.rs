use clap::{crate_authors, crate_description, crate_version, App, Arg};

fn main() {
    let _matches = App::new("Chum Bucket")
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
                .required(false),
        )
        .arg(
            Arg::with_name("file")
                .about("Input file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
}
