use anyhow::Result;

pub fn write_to_disk<T>(loc: &str, t: T) -> Result<()>
where
    T: serde::Serialize,
{
    let s = serde_json::to_string(&t)?;

    std::fs::write(loc, &s)?;

    Ok(())
}
