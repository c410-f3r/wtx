use plotters::{
  prelude::{
    ChartBuilder, IntoDrawingArea, IntoSegmentedCoord, LabelAreaPosition, PathElement, SVGBackend,
    SeriesLabelPosition,
  },
  series::Histogram,
  style::{AsRelative, Color, Palette99, PaletteColor, BLACK, WHITE},
};

#[derive(Debug)]
pub(crate) struct Agent {
  pub(crate) result: u128,
  pub(crate) name: String,
}

pub(crate) fn flush(agents: &[Agent], caption: &str, output: &str) {
  if agents.is_empty() {
    return;
  }
  let x_spec = agents.iter().map(|el| &el.name).cloned().collect::<Vec<_>>();
  let root = SVGBackend::new(output, (1280, 780)).into_drawing_area();
  root.fill(&WHITE).unwrap();
  let mut ctx = ChartBuilder::on(&root)
    .caption(caption, ("sans-serif", (4).percent_height()))
    .margin((1).percent())
    .set_label_area_size(LabelAreaPosition::Left, (10).percent())
    .set_label_area_size(LabelAreaPosition::Bottom, (5).percent())
    .build_cartesian_2d(x_spec.into_segmented(), {
      let start = 0u128;
      let exact_end = agents.iter().map(|el| el.result).max().unwrap_or(5000);
      let surplus_end = ((exact_end / 500) + 1) * 500;
      start..surplus_end
    })
    .unwrap();
  ctx
    .configure_mesh()
    .axis_desc_style(("sans-serif", 15))
    .bold_line_style(WHITE.mix(0.3))
    .y_desc("Time (ms)")
    .draw()
    .unwrap();
  for (idx, agent) in agents.iter().enumerate() {
    let _ = ctx
      .draw_series(
        Histogram::vertical(&ctx)
          .style(PaletteColor::<Palette99>::pick(idx).mix(0.5).filled())
          .data([(&agent.name, agent.result)]),
      )
      .unwrap()
      .label(format!("{} ({}ms)", &agent.name, agent.result))
      .legend(move |(x, y)| {
        PathElement::new([(x, y), (x + 20, y)], PaletteColor::<Palette99>::pick(idx))
      });
  }
  ctx
    .configure_series_labels()
    .border_style(BLACK)
    .background_style(WHITE.mix(0.8))
    .position(SeriesLabelPosition::UpperRight)
    .draw()
    .unwrap();
  root.present().unwrap();
}
