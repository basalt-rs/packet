use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    #[cfg(feature = "dev")]
    let config = std::fs::read_to_string("./examples/render-test.toml").unwrap();
    #[cfg(not(feature = "dev"))]
    let config = include_str!("./render-test.toml");

    let x = bedrock::Config::from_str(config, Some("one.toml")).unwrap();

    let mut out = std::fs::File::create("test.pdf").unwrap();

    x.write_pdf(&mut out, None)?;

    Ok(())
}
