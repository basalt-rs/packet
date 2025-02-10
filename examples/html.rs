use std::error::Error;

use bedrock::render::markdown::MarkdownRenderable;

fn main() -> Result<(), Box<dyn Error>> {
    let r = MarkdownRenderable::from_raw(
        r#"
# hello world

This is Euler's identity! blah $e ^(pi i) + 1 = 0$ blah

$$
e ^(pi i) + 1 &= 0 \
a x^2 + b x + c &= 0
$$
"#,
    );

    let html = r.html()?;

    std::fs::write("out.html", html)?;
    Ok(())
}
