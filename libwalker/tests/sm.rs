extern crate walker;

use walker::Walker;

#[test]
fn test_walk() -> Result<(), Box<dyn std::error::Error>> {
    let mut walker = Walker::new(
        "/home/fishy/Documents/Github/sp-docgen/libwalker/tests/sourcemod",
        vec!["plugins/include/geoip.inc"],
    )?;

    let spec_diffs = walker.walk(None)?;

    for t in spec_diffs {
        for c in t {
            std::fs::write(format!("tests/contents/{}", c.commit.to_string()), c.content)?;
        }
    }

    Ok(())
}
