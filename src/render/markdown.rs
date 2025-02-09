use std::num::NonZero;

use comemo::Track;
use pulldown_cmark::{Alignment, CodeBlockKind, Options, Tag};
use pulldown_cmark_ast::{Ast, Tree};
use typst::{
    diag::EcoString,
    foundations::{Content, Packed, Scope, Smart, Value},
    layout::{BlockBody, BlockElem, Celled, Length, Ratio, Sizing, TrackSizings},
    model::{
        EnumElem, EnumItem, FigureElem, HeadingElem, LinkElem, LinkTarget, ListElem, ListItem,
        ParbreakElem, TableCell, TableChild, TableElem, TableHeader, TableItem, Url,
    },
    syntax::Span,
    text::{LinebreakElem, RawContent, RawElem, SpaceElem, StrikeElem, TextElem},
    visualize::LineElem,
    World,
};

struct MarkdownRenderer<'a> {
    world: &'a dyn World,
}

impl<'a> MarkdownRenderer<'a> {
    fn new(world: &'a dyn World) -> Self {
        Self { world }
    }

    fn render_tree(&self, tree: Tree) -> Content {
        match tree {
            Tree::Group(g) => match g.tag.item {
                Tag::Paragraph => Content::sequence(
                    std::iter::once(Content::new(ParbreakElem::new()))
                        .chain(g.stream.0.into_iter().map(|t| self.render_tree(t)))
                        .chain(std::iter::once(Content::new(ParbreakElem::new()))),
                ),
                Tag::Heading { level, .. } => {
                    Content::new(HeadingElem::new(self.render_ast(g.stream)).with_level(
                        typst::foundations::Smart::Custom(
                            NonZero::new(level as usize).expect("1 <= level <= 6"),
                        ),
                    ))
                }
                Tag::BlockQuote(_) => {
                    // TODO: use block quote kind somehow?
                    // Blockquote ~ #figure()
                    let content = Content::sequence(
                        std::iter::once(Content::new(ParbreakElem::new()))
                            .chain(g.stream.0.into_iter().map(|t| self.render_tree(t)))
                            .chain(std::iter::once(Content::new(ParbreakElem::new()))),
                    );
                    Content::new(FigureElem::new(content.aligned(
                        typst::layout::Alignment::H(typst::layout::HAlignment::Left),
                    )))
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
                    Content::new(FigureElem::new(Content::new(elem)))
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
                            .map(|(i, t)| match t {
                                Tree::Group(group) => match group.tag.item {
                                    Tag::Item => Packed::new(
                                        EnumItem::new(self.render_ast(group.stream))
                                            .with_number(Some(ord as usize + i)),
                                    ),
                                    _ => unreachable!(),
                                },
                                _ => unreachable!(),
                            })
                            .collect();
                        Content::new(EnumElem::new(packed))
                    } else {
                        let packed = g
                            .stream
                            .0
                            .into_iter()
                            .map(|t| self.render_tree(t).into_packed().unwrap())
                            .collect();
                        Content::new(ListElem::new(packed))
                    }
                }
                Tag::Item => Content::new(ListItem::new(self.render_ast(g.stream))),
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
                            .map(|t| TableItem::Cell(self.render_tree(t).into_packed().unwrap()))
                            .collect(),
                    ))));

                    for thing in things {
                        let row = match thing {
                            Tree::Group(hg) => match hg.tag.item {
                                Tag::TableRow => hg.stream.0,
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        };
                        children.extend(row.into_iter().map(|t| {
                            TableChild::Item(TableItem::Cell(
                                self.render_tree(t).into_packed().unwrap(),
                            ))
                        }));
                    }

                    Content::new(FigureElem::new(Content::new(
                        TableElem::new(children)
                            .with_columns(TrackSizings(
                                (0..cols).map(|_| Sizing::Auto).collect::<Vec<_>>().into(),
                            ))
                            .with_align(Celled::Array(
                                align
                                    .iter()
                                    .map(|a| match a {
                                        Alignment::None => Smart::Auto,
                                        Alignment::Left => {
                                            Smart::Custom(typst::layout::Alignment::H(
                                                typst::layout::HAlignment::Left,
                                            ))
                                        }
                                        Alignment::Center => {
                                            Smart::Custom(typst::layout::Alignment::H(
                                                typst::layout::HAlignment::Center,
                                            ))
                                        }
                                        Alignment::Right => {
                                            Smart::Custom(typst::layout::Alignment::H(
                                                typst::layout::HAlignment::Right,
                                            ))
                                        }
                                    })
                                    .collect(),
                            )),
                    )))
                }
                Tag::TableHead => {
                    let items = g
                        .stream
                        .0
                        .into_iter()
                        .map(|t| TableItem::Cell(self.render_tree(t).into_packed().unwrap()))
                        .collect();
                    Content::new(TableHeader::new(items))
                }
                Tag::TableRow => {
                    let items = g
                        .stream
                        .0
                        .into_iter()
                        .map(|t| TableItem::Cell(self.render_tree(t).into_packed().unwrap()))
                        .collect();
                    Content::new(TableHeader::new(items))
                }
                Tag::TableCell => Content::new(TableCell::new(self.render_ast(g.stream))),
                Tag::Emphasis => self.render_ast(g.stream).emph(),
                Tag::Strong => self.render_ast(g.stream).strong(),
                Tag::Strikethrough => Content::new(StrikeElem::new(self.render_ast(g.stream))),
                Tag::Link { dest_url, .. } => Content::new(LinkElem::new(
                    LinkTarget::Dest(typst::model::Destination::Url(
                        Url::new(&*dest_url).unwrap(),
                    )),
                    self.render_ast(g.stream),
                )),
                Tag::Image { .. } => todo!(),
                Tag::MetadataBlock(_) => unreachable!("Feature is disabled"),
            },
            Tree::Text(spanned) => Content::new(TextElem::new(spanned.item.as_ref().into())),
            Tree::Code(spanned) => {
                Content::new(RawElem::new(RawContent::Text(spanned.item.as_ref().into())))
            }
            Tree::Html(_) => panic!("html is not supported"),
            Tree::InlineHtml(_) => panic!("html is not supported"),
            Tree::FootnoteReference(_) => unreachable!("Feature is disabled"),
            Tree::SoftBreak(_) => Content::new(SpaceElem::new()),
            Tree::HardBreak(_) => Content::new(LinebreakElem::new()),
            Tree::Rule(_) => Content::new(LineElem::new().with_length(typst::layout::Rel {
                rel: Ratio::new(1.),
                abs: Length::zero(),
            })),
            Tree::TaskListMarker(_) => unreachable!("Feature is disabled"),
            Tree::InlineMath(spanned) => {
                let content = spanned.item;

                let val = typst::eval::eval_string(
                    self.world.track(),
                    &*content,
                    Span::detached(),
                    typst::eval::EvalMode::Math,
                    Scope::new(),
                )
                .unwrap();

                let content = match val {
                    Value::Content(content) => content,
                    _ => unreachable!(),
                };

                content
            }
            Tree::DisplayMath(spanned) => {
                let content = spanned.item.trim();

                let val = typst::eval::eval_string(
                    self.world.track(),
                    &format!("$ {} $", content),
                    Span::detached(),
                    typst::eval::EvalMode::Markup,
                    self.world.library().math.scope().clone(),
                )
                .unwrap();

                let content = match val {
                    Value::Content(content) => content,
                    _ => unreachable!(),
                };

                content
            }
        }
    }

    fn render_ast(&self, ast: Ast) -> Content {
        Content::sequence(ast.0.into_iter().map(|t| self.render_tree(t)))
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

    fn render(&self, markdown: impl AsRef<str>) -> Content {
        let markdown = markdown.as_ref();
        let options = Options::ENABLE_GFM
            | Options::ENABLE_MATH
            | Options::ENABLE_TABLES
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_STRIKETHROUGH;
        let ast = Ast::new_ext(markdown, options);
        self.render_ast(ast)
    }
}

pub fn render_markdown(markdown: impl AsRef<str>, world: &impl World) -> Content {
    MarkdownRenderer::new(world).render(markdown)
}
