use std::error::Error;

use bedrock::render::markdown::MarkdownRenderable;

fn main() -> Result<(), Box<dyn Error>> {
    let r = MarkdownRenderable::from_raw(
        r#"
# hello world

This is Euler's identity! blah $e ^(pi i) + 1 = 0$ blah

$$
e ^(pi i) + 1 &= 0
$$

This is factorial:

$$
"fact"(n) := cases(
  1 &"if" n <= 0,
   n "fact"(n - 1) &"otherwise",
)
$$
"#,
    );

    let html = r.html()?;

    std::fs::write("out.html", html)?;
    Ok(())
}
