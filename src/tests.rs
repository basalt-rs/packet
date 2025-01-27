use miette::Result;

use super::*;
use std::io::Cursor;

const EXAMPLE_ONE_CONTENT: &str = include_str!("../examples/one.toml");

#[test]
fn packets_parse_correctly() -> Result<()> {
    // parse example one
    Config::from_str(EXAMPLE_ONE_CONTENT, Some("Cargo.toml"))?;
    Ok(())
}

#[test]
fn packet_files_parse_correctly() -> Result<()> {
    let mut file = Cursor::new(EXAMPLE_ONE_CONTENT);
    let _ = Config::read(&mut file, Some("Cargo.toml"))?;
    Ok(())
}

#[tokio::test]
async fn packet_files_parse_correctly_async() -> Result<()> {
    let mut file = Cursor::new(EXAMPLE_ONE_CONTENT);
    let _ = Config::read_async(&mut file, Some("cargo.toml")).await?;
    Ok(())
}
