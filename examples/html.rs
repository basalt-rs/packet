use std::error::Error;

use bedrock::render::markdown::MarkdownRenderable;
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

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

```rs
use std::io::{self, BufRead, BufReader};
fn main() {
    println!("{}", BufRead::lines(BufReader::new(io::stdin()))
            .next()
            .unwrap()
            .unwrap()
            .chars()
            .rev()
            .collect::<String>());
}
```
"#,
    );

    let html = r.html()?;

    let ts = syntect::highlighting::ThemeSet::load_defaults();
    // One of:
    //
    // InspiredGitHub
    // Solarized (dark)
    // Solarized (light)
    // base16-eighties.dark
    // base16-mocha.dark
    // base16-ocean.dark
    // base16-ocean.light
    let theme = &ts.themes["base16-mocha.dark"];
    let css = css_for_theme_with_class_style(theme, ClassStyle::Spaced).unwrap();

    std::fs::write(
        "out.html",
        format!(
            "<style>
    pre {{
        background: oklch(.208 .042 265.755);
        padding: 1rem;
        color: #ccc;
    }}
    {}
    </style>
    {}
    ",
            css, html
        ),
    )?;
    Ok(())
}
