extern crate walker;

use walker::Walker;

#[test]
fn test_walk() -> Result<(), Box<dyn std::error::Error>> {
    let mut walker = Walker::new(
        "/home/fishy/Documents/Github/sp-docgen/libwalker/tests/sourcemod",
        vec!["plugins/include/geoip.inc"],
    )?;

    walker.walk(None)?;

    println!("{:?}", walker.commits());

    Ok(())
}
