use clap::{crate_version, crate_description, crate_authors, App, Arg};

fn main() {
    let _matches = App::new("Chum Bucket")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("include")
                .about("Target file is an include file")
                .short('i')
        )
        .arg(
            Arg::new("output")
                .about("Location to output bundle to")
                .short('o')
                .required(false)
        )
        .get_matches();
}
