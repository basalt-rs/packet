use bedrock::Config;

const FILE: &str = include_str!("./imports.toml");
const SETUP_FILE: &str = include_str!("./setup.toml");

#[test]
fn parse() -> miette::Result<()> {
    Config::from_str(FILE, Some("imports.toml"))?;
    Ok(())
}

#[test]
fn parse_get() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("imports.toml"))?;
    dbg!(config.hash());
    let setup = config.setup.unwrap();
    let setup_toml = toml_edit::de::from_str(SETUP_FILE).unwrap();
    assert_eq!(setup, setup_toml);
    Ok(())
}
