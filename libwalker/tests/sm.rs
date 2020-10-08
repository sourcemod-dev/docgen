extern crate walker;

use walker::Walker;

#[test]
fn test_walk() -> Result<(), Box<dyn std::error::Error>> {
    let mut walker = Walker::from_remote(
        "https://github.com/alliedmodders/sourcemod.git",
        "sourcemod",
        vec!["plugins/include/geoip.inc"],
    )?;

    let spec_diffs = walker.walk(None)?;

    for t in spec_diffs {
        for c in t {
            println!("{} {}", c.commit, c.count);

            std::fs::write(
                format!("tests/contents/{}", c.commit.to_string()),
                c.content,
            )?;
        }
    }

    Ok(())
}
