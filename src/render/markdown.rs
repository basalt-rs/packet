use std::{num::NonZero, str::FromStr};

use comemo::Track;
use ecow::EcoVec;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag};
use pulldown_cmark_ast::{Ast, Tree};
use serde::{Deserialize, Serialize};
use typst::{
    diag::{EcoString, SourceDiagnostic, SourceResult},
    foundations::{Content, Packed, Scope, Smart, Value},
    layout::{Celled, Length, Ratio, Sizing, TrackSizings},
    model::{
        EnumElem, EnumItem, FigureElem, HeadingElem, LinkElem, LinkTarget, ListElem, ListItem,
        ParbreakElem, TableCell, TableChild, TableElem, TableHeader, TableItem, Url,
    },
    syntax::Span,
    text::{LinebreakElem, RawContent, RawElem, SpaceElem, StrikeElem, TextElem},
    visualize::LineElem,
    World,
};

use crate::render::typst::TypstWrapperWorld;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RenderError {
    #[error("Error while processing typst: {0:?}")]
    TypstError(Vec<SourceDiagnostic>),
}

impl From<EcoVec<SourceDiagnostic>> for RenderError {
    fn from(value: EcoVec<SourceDiagnostic>) -> Self {
        Self::TypstError(value.to_vec())
    }
}

impl From<RenderError> for std::io::Error {
    fn from(val: RenderError) -> Self {
        std::io::Error::other(format!("{}", val))
    }
}

// For some reason, `Options::ENABLE_TABLES | Options::ENABLE_SMART_PUNCTUATION` is not const...
const CMARK_OPTIONS: Options = Options::from_bits_truncate(
    (1 << 1) // Options::ENABLE_TABLES
    | (1 << 5) // Options::ENABLE_SMART_PUNCTUATION
    | (1 << 3) // Options::ENABLE_STRIKETHROUGH
    | (1 << 10), // Options::ENABLE_MATH
);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[repr(transparent)]
#[serde(transparent)]
pub struct MarkdownRenderable(String);

impl From<String> for MarkdownRenderable {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MarkdownRenderable {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl FromStr for MarkdownRenderable {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl MarkdownRenderable {
    pub fn from_raw(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn raw(&self) -> &str {
        &self.0
    }

    /// Renders the given string into HTML
    ///
    /// This uses typst to fill in the maths blocks.
    pub fn html(&self) -> Result<String, RenderError> {
        let parser = Parser::new_ext(self.raw(), CMARK_OPTIONS);
        let mut errors = Vec::new();
        let parser = parser.map(|event| match event {
            pulldown_cmark::Event::InlineMath(cow_str) => {
                // TODO: This should parse the cow_str into a Content and somehow convert that to a
                // page.
                let f = format!(
                    "#set page(width: auto, height: auto, margin: 0em)
                    ${}$",
                    cow_str
                );
                let world = TypstWrapperWorld::new(f);
                match typst::compile(&world).output {
                    Ok(doc) => {
                        let svg = typst_svg::svg(&doc.pages[0]);
                        Event::InlineHtml(svg.into())
                    }
                    Err(err) => {
                        errors.extend(err);
                        Event::Text("".into())
                    }
                }
            }
            pulldown_cmark::Event::DisplayMath(cow_str) => {
                // TODO: This should parse the cow_str into a Content and somehow convert that to a
                // page.
                let f = format!(
                    "
                    #set page(width: auto, height: auto, margin: 0em)
                    $ {} $
                    ",
                    cow_str
                );
                let world = TypstWrapperWorld::new(f);
                match typst::compile(&world).output {
                    Ok(doc) => {
                        let svg = typst_svg::svg(&doc.pages[0]);
                        Event::Html(svg.into())
                    }
                    Err(err) => {
                        errors.extend(err);
                        Event::Text("".into())
                    }
                }
            }
            e => e,
        });
        let mut s = String::new();
        pulldown_cmark::html::push_html(&mut s, parser);
        if !errors.is_empty() {
            Err(RenderError::TypstError(errors))?
        } else {
            Ok(s)
        }
    }

    /// Renders the given string into typst content
    pub fn content(&self, world: &impl World) -> Result<Content, RenderError> {
        render_markdown(self.raw(), world)
    }
}

fn map_align(a: &Alignment) -> Smart<typst::layout::Alignment> {
    match a {
        Alignment::None => Smart::Auto,
        Alignment::Left => {
            Smart::Custom(typst::layout::Alignment::H(typst::layout::HAlignment::Left))
        }
        Alignment::Center => Smart::Custom(typst::layout::Alignment::H(
            typst::layout::HAlignment::Center,
        )),
        Alignment::Right => Smart::Custom(typst::layout::Alignment::H(
            typst::layout::HAlignment::Right,
        )),
    }
}

struct TypstMarkdownRenderer<'a> {
    world: &'a dyn World,
}

impl<'a> TypstMarkdownRenderer<'a> {
    fn new(world: &'a dyn World) -> Self {
        Self { world }
    }

    fn render_tree(&self, tree: Tree) -> SourceResult<Content> {
        match tree {
            Tree::Group(g) => match g.tag.item {
                Tag::Paragraph => Ok(Content::sequence(
                    std::iter::once(Ok(Content::new(ParbreakElem::new())))
                        .chain(g.stream.0.into_iter().map(|t| self.render_tree(t)))
                        .chain(std::iter::once(Ok(Content::new(ParbreakElem::new()))))
                        .collect::<SourceResult<Vec<_>>>()?,
                )),
                Tag::Heading { level, .. } => Ok(Content::new(
                    HeadingElem::new(self.render_ast(g.stream)?).with_level(
                        typst::foundations::Smart::Custom(
                            NonZero::new(level as usize).expect("1 <= level <= 6"),
                        ),
                    ),
                )),
                Tag::BlockQuote(_) => {
                    // TODO: use block quote kind somehow?
                    // Blockquote ~ #figure()
                    let content = Content::sequence(
                        std::iter::once(Ok(Content::new(ParbreakElem::new())))
                            .chain(g.stream.0.into_iter().map(|t| self.render_tree(t)))
                            .chain(std::iter::once(Ok(Content::new(ParbreakElem::new()))))
                            .collect::<SourceResult<Vec<_>>>()?,
                    );
                    Ok(Content::new(FigureElem::new(content.aligned(
                        typst::layout::Alignment::H(typst::layout::HAlignment::Left),
                    ))))
                }
                Tag::CodeBlock(code_block_kind) => {
                    let content = self.render_ast_to_text(g.stream);
                    let elem = RawElem::new(RawContent::Text(content)).with_block(true);
                    let elem = match code_block_kind {
                        CodeBlockKind::Indented => elem,
                        CodeBlockKind::Fenced(s) => {
                            if s.is_empty() {
                                elem
                            } else {
                                elem.with_lang(Some(s.as_ref().into()))
                            }
                        }
                    };
                    Ok(Content::new(FigureElem::new(Content::new(elem))))
                }
                Tag::HtmlBlock => panic!("HTML blocks not supported"), // TODO: Return error
                Tag::List(ord) => {
                    // TODO: Ordered lists
                    if let Some(ord) = ord {
                        let packed = g
                            .stream
                            .0
                            .into_iter()
                            .enumerate()
                            .map(|(i, t)| -> SourceResult<_> {
                                match t {
                                    Tree::Group(group) => match group.tag.item {
                                        Tag::Item => Ok(Packed::new(
                                            EnumItem::new(self.render_ast(group.stream)?)
                                                .with_number(Some(ord as usize + i)),
                                        )),
                                        _ => unreachable!(),
                                    },
                                    _ => unreachable!(),
                                }
                            })
                            .collect::<SourceResult<Vec<_>>>()?;
                        Ok(Content::new(EnumElem::new(packed)))
                    } else {
                        let packed = g
                            .stream
                            .0
                            .into_iter()
                            .map(|t| self.render_tree(t).map(|c| c.into_packed().unwrap()))
                            .collect::<SourceResult<_>>()?;
                        Ok(Content::new(ListElem::new(packed)))
                    }
                }
                Tag::Item => Ok(Content::new(ListItem::new(self.render_ast(g.stream)?))),
                Tag::FootnoteDefinition(_) => unreachable!("Feature is disabled"),
                Tag::Table(align) => {
                    let mut things = g.stream.0;
                    let mut children = Vec::new();
                    let header = match things.remove(0) {
                        Tree::Group(hg) => match hg.tag.item {
                            Tag::TableHead => hg.stream,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };

                    let cols = header.0.len();

                    children.push(TableChild::Header(Packed::new(TableHeader::new(
                        header
                            .0
                            .into_iter()
                            .map(|t| {
                                self.render_tree(t)
                                    .map(|c| c.into_packed().unwrap())
                                    .map(TableItem::Cell)
                            })
                            .collect::<SourceResult<_>>()?,
                    ))));

                    for thing in things {
                        let row = match thing {
                            Tree::Group(hg) => match hg.tag.item {
                                Tag::TableRow => hg.stream.0,
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        };
                        children.extend_from_slice(
                            &row.into_iter()
                                .map(|t| {
                                    self.render_tree(t)
                                        .map(|c| c.into_packed().unwrap())
                                        .map(TableItem::Cell)
                                        .map(TableChild::Item)
                                })
                                .collect::<SourceResult<Vec<_>>>()?,
                        );
                    }

                    let columns = (0..cols).map(|_| Sizing::Auto).collect::<Vec<_>>();

                    Ok(Content::new(FigureElem::new(Content::new(
                        TableElem::new(children)
                            .with_columns(TrackSizings(columns.into()))
                            .with_align(Celled::Array(align.iter().map(map_align).collect())),
                    ))))
                }
                Tag::TableHead => {
                    let items = g
                        .stream
                        .0
                        .into_iter()
                        .map(|t| {
                            self.render_tree(t)
                                .map(|c| c.into_packed().unwrap())
                                .map(TableItem::Cell)
                        })
                        .collect::<SourceResult<Vec<_>>>()?;
                    Ok(Content::new(TableHeader::new(items)))
                }
                Tag::TableRow => g
                    .stream
                    .0
                    .into_iter()
                    .map(|t| {
                        self.render_tree(t)
                            .map(|c| c.into_packed().unwrap())
                            .map(TableItem::Cell)
                    })
                    .collect::<SourceResult<_>>()
                    .map(TableHeader::new)
                    .map(Content::new),
                Tag::TableCell => self
                    .render_ast(g.stream)
                    .map(TableCell::new)
                    .map(Content::new),
                Tag::Emphasis => self.render_ast(g.stream).map(Content::emph),
                Tag::Strong => self.render_ast(g.stream).map(Content::strong),
                Tag::Strikethrough => self
                    .render_ast(g.stream)
                    .map(StrikeElem::new)
                    .map(Content::new),
                Tag::Link { dest_url, .. } => Ok(Content::new(LinkElem::new(
                    LinkTarget::Dest(typst::model::Destination::Url(
                        Url::new(&*dest_url).unwrap(),
                    )),
                    self.render_ast(g.stream)?,
                ))),
                Tag::Image { .. } => todo!(),
                Tag::MetadataBlock(_) => unreachable!("Feature is disabled"),
            },
            Tree::Text(spanned) => Ok(Content::new(TextElem::new(spanned.item.as_ref().into()))),
            Tree::Code(spanned) => Ok(Content::new(RawElem::new(RawContent::Text(
                spanned.item.as_ref().into(),
            )))),
            Tree::Html(_) => panic!("html is not supported"),
            Tree::InlineHtml(_) => panic!("html is not supported"),
            Tree::FootnoteReference(_) => unreachable!("Feature is disabled"),
            Tree::SoftBreak(_) => Ok(Content::new(SpaceElem::new())),
            Tree::HardBreak(_) => Ok(Content::new(LinebreakElem::new())),
            Tree::Rule(_) => Ok(Content::new(LineElem::new().with_length(
                typst::layout::Rel {
                    rel: Ratio::new(1.),
                    abs: Length::zero(),
                },
            ))),
            Tree::TaskListMarker(_) => unreachable!("Feature is disabled"),
            Tree::InlineMath(spanned) => {
                let content = spanned.item;

                let val = typst::eval::eval_string(
                    self.world.track(),
                    &content,
                    Span::detached(),
                    typst::eval::EvalMode::Math,
                    Scope::new(),
                )?;

                match val {
                    Value::Content(content) => Ok(content),
                    _ => unreachable!(),
                }
            }
            Tree::DisplayMath(spanned) => {
                let content = spanned.item.trim();

                let val = typst::eval::eval_string(
                    self.world.track(),
                    &format!("$ {} $", content),
                    Span::detached(),
                    typst::eval::EvalMode::Markup,
                    self.world.library().math.scope().clone(),
                )?;

                match val {
                    Value::Content(content) => Ok(content),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn render_ast(&self, ast: Ast) -> SourceResult<Content> {
        Ok(Content::sequence(
            ast.0
                .into_iter()
                .map(|t| self.render_tree(t))
                .collect::<SourceResult<Vec<_>>>()?,
        ))
    }

    fn render_ast_to_text(&self, ast: Ast) -> EcoString {
        let mut s = EcoString::new();
        for t in ast.0 {
            match t {
                Tree::Text(spanned) => {
                    s.push_str(&spanned.item);
                }
                s => unreachable!("need to impl {:?}", s),
            }
        }
        s
    }

    fn render(&self, markdown: impl AsRef<str>) -> SourceResult<Content> {
        let markdown = markdown.as_ref();
        let ast = Ast::new_ext(markdown, CMARK_OPTIONS);
        self.render_ast(ast)
    }
}

pub fn render_markdown(
    markdown: impl AsRef<str>,
    world: &impl World,
) -> Result<Content, RenderError> {
    Ok(TypstMarkdownRenderer::new(world).render(markdown)?)
}
