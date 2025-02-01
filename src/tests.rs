use language::{BuiltInLanguage, Language, Version};
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
    let config = Config::read(&mut file, Some("Cargo.toml"))?;

    assert_eq!(
        Some(&Language::BuiltIn {
            language: BuiltInLanguage::Python3,
            version: Version::Latest
        }),
        config.languages.get_by_str(&"python3")
    );

    assert_eq!(
        Some(&Language::BuiltIn {
            language: BuiltInLanguage::Java,
            version: Version::Specific("23".into())
        }),
        config.languages.get_by_str(&"java")
    );

    assert_eq!(
        Some(&Language::Custom {
            raw_name: "ocaml".into(),
            name: "ocaml".into(),
            build: Some("ocamlc -o out solution.ml".into()),
            run: "./out".into(),
            source_file: "solution.ml".into()
        }),
        config.languages.get_by_str(&"ocaml")
    );
    Ok(())
}

#[tokio::test]
async fn packet_files_parse_correctly_async() -> Result<()> {
    let mut file = Cursor::new(EXAMPLE_ONE_CONTENT);
    let _ = Config::read_async(&mut file, Some("cargo.toml")).await?;
    Ok(())
}

#[test]
fn default_config() {
    let _ = Config::default();
}
