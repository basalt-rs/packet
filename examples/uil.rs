use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    // using the dev feature so we don't have have to recompile for every change to the toml
    #[cfg(feature = "dev")]
    let config = std::fs::read_to_string("./examples/uil.toml").unwrap();
    #[cfg(not(feature = "dev"))]
    let config = include_str!("./uil.toml");

    let x = bedrock::Config::from_str(config, Some("one.toml")).unwrap();

    let mut out = std::fs::File::create("uil.pdf").unwrap();

    x.write_pdf(&mut out)?;

    Ok(())
}
