use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;
use endgame_egui::{CellStyle, GridContext, Theme};
use endgame_grid::{dynamic, Coord};
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    clockwise: bool,
    source: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::Rotation
    }

    fn label(&self) -> &'static str {
        "Rotation"
    }

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on a grid cell to see how the coordinate will rotate around the origin.\n",
        );

        common::unary_coordinate_label(ui, &self.source);

        ui.radio_value(&mut self.clockwise, true, "Clockwise");
        ui.radio_value(&mut self.clockwise, false, "Counter-clockwise");
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        common::unary_coordinate_select(ctx, &mut self.source);

        let Some(coord) = self.source else { return };

        let mut spec: &CellStyle = common::SOURCE_CELL_SPEC.deref();
        let mut cur_coord = coord;
        loop {
            grc.render_coord_cell(&cur_coord, &spec, None::<&str>);
            let next_coord = if self.clockwise {
                cur_coord.rotate_clockwise()
            } else {
                cur_coord.rotate_counterclockwise()
            };
            grc.render_hollow_arrow_coords(
                &cur_coord,
                &next_coord,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
            );

            if next_coord == coord {
                break;
            }
            // All other rotated coordinates wll get a different color.
            spec = common::TARGET_CELL_SPEC.deref();
            cur_coord = next_coord;
        }
    }
}
