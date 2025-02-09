//! This is not really an example, mainly just testing to see how typst generates its output

use bedrock::render::typst::TypstWrapperWorld;

fn main() {
    let welt = TypstWrapperWorld::new(include_str!("../template.typ").into());

    let res = typst::eval::eval_string(
        comemo::Track::track(&welt),
        r#"
= hello world

1. foo
2. bar
3. baz
           "#,
        typst::syntax::Span::detached(),
        typst::eval::EvalMode::Markup,
        typst::foundations::Scope::new(),
    );

    dbg!(res.unwrap());
}
