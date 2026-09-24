//! Prints the strand of one or more SourcePawn files as JSON.
//!
//! Usage: `cargo run -p alternator --example dump -- <file> [out.json]`

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: dump <file> [out.json]");
        std::process::exit(2);
    }

    let content = std::fs::read(&args[0]).expect("unable to read file");

    match alternator::parse_bytes(&content) {
        Ok(strand) => {
            // Round trip through Value for stable key order
            let value = serde_json::to_value(&strand).unwrap();
            let json = serde_json::to_string_pretty(&value).unwrap();
            match args.get(1) {
                Some(out) => std::fs::write(out, json + "\n").expect("unable to write output"),
                None => println!("{}", json),
            }
        }
        Err(e) => {
            eprintln!("{}: {}", args[0], e);
            std::process::exit(1);
        }
    }
}
