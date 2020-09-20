extern crate walker;

use walker::Walker;

#[test]
fn test_walk() -> Result<(), Box<dyn std::error::Error>> {
    let mut walker = Walker::new(
        "/home/fishy/Documents/Github/sp-docgen/libwalker/tests/sourcemod",
        vec!["plugins/include/*.inc"],
    )?;

    let spec_diffs = walker.walk(None)?;

    println!("{:?}", spec_diffs);

    Ok(())
}
