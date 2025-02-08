const CONFIG: &str = include_str!("./one.toml");

#[tokio::main]
async fn main() {
    let x = bedrock::Config::from_str(CONFIG, Some("one.toml")).unwrap();

    let mut out = std::fs::File::create("urmom.pdf").unwrap();

    x.write_pdf(&mut out);
}
