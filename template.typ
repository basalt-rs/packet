#set table(
  stroke: (x, y) => if y == 0 {
    (bottom: 0.7pt + black)
  } else { 
    (
      bottom: .1pt + black,
      left: if x == 0 { none } else {
        .1pt + black
      }
    )
  }
)

// Title page
#align(center, {
  text(size: 1.6em, weight: "bold")[#title]
  box(line(length: 100%, stroke: 1pt))
})

#preamble

// Page heading
#set page(
  numbering: "1",
  header: context {
    if counter(page).get().first() > 1 [
      #set text(style: "italic")
      #title
      #block(line(length: 100%, stroke: 0.5pt), above: 0.6em)
    ]
  })

  #let codeblock(content) = {
    box(
      inset: .75em,
      width: 100%,
      raw(content)
    )
  };

  #let test-case(input, output) = {
    let colour = luma(95%);
    box(
      if (input.len() == 0) and (output.len() != 0) [
        #grid(
          columns: (1fr),
          column-gutter: 1em,
          row-gutter: .5em,
          strong[Output],
          grid.cell(codeblock(output), fill: colour, stroke: luma(80%)),
        )
      ] else [
        #grid(
          columns: (1fr, 1fr),
          column-gutter: 1em,
          row-gutter: .5em,
          strong[Input],
          strong[Output],
          grid.cell(codeblock(input), fill: colour, stroke: luma(80%)),
          grid.cell(codeblock(output), fill: colour, stroke: luma(80%)),
        )
      ]
    )
  }

  #for q in problems {
    [
      #pagebreak()
      = #q.title
      #if "description" in q {
        q.description
      }

      #for (i, test) in q.tests.filter(t => t.visible).enumerate() {
        [== Test case #{i+1}]
        test-case(test.input, test.output)
      }
    ]
  }
