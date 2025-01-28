use bedrock::Config;

const FILE: &'static str = include_str!("./imports.toml");
const SETUP_FILE: &'static str = include_str!("./setup.toml");

#[test]
fn parse() -> miette::Result<()> {
    Config::from_str(FILE, Some("imports.toml"))?;
    Ok(())
}

#[test]
fn parse_get() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("imports.toml"))?;
    let setup = config.setup.map(|roi| roi.get()).unwrap().unwrap();
    let setup_toml = toml_edit::de::from_str(SETUP_FILE).unwrap();
    assert_eq!(setup, setup_toml);
    Ok(())
}
