use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use egui::Color32;
use endgame_egui::{render_arrow, GridContext, SolidArrowStyle, Theme};
use endgame_grid::{dynamic, Coord};
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    source: Option<dynamic::Coord>,
    target: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::PathIterator
    }

    fn label(&self) -> &'static str {
        "Path Iterator"
    }

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on two grid cells to experiment with traversal paths between the coordinates.\n",
        );

        common::binary_coordinates_labels(ui, "source", &self.source, "target", &self.target);
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        common::binary_coordinates_select(ctx, &mut self.source, &mut self.target);

        let Some(source) = self.source else { return };

        grc.render_coord_cell(&source, &common::SOURCE_CELL_SPEC, None::<&str>);

        let Some(target) = self.target else { return };

        grc.render_coord_cell(&target, &common::TARGET_CELL_SPEC, None::<&str>);

        let source_screen = grc
            .transform
            .transform_pos(endgame_egui::coord_to_egui_pos2(&source, &grc.szg));
        let target_screen = grc
            .transform
            .transform_pos(endgame_egui::coord_to_egui_pos2(&target, &grc.szg));

        if source_screen != target_screen {
            let style = SolidArrowStyle {
                color: Color32::GREEN,
                width: 2.0,
                to_head: true,
                from_head: false,
                label: None,
            };
            render_arrow(source_screen, target_screen, &style, None, &grc.painter);
        }

        // If the source is just the target, we just render a single self-arrow.
        // TODO Can we just fold this into the loop by changing the base case?
        if source == target {
            grc.render_hollow_arrow_coords(
                &source,
                &target,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
            );
            return;
        }

        let mut prev_coord = None;
        for coord in source.path_iterator(&target) {
            if let Some(prev) = prev_coord {
                grc.render_hollow_arrow_coords(
                    &prev,
                    &coord,
                    common::HOLLOW_ARROW_STYLE.deref(),
                    None,
                );
            }
            prev_coord = Some(coord);
        }
    }
}
