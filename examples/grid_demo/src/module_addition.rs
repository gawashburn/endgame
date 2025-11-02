use crate::common;
use crate::common::GridExample;
use crate::common::{wrapped_string, ExampleUi};
use endgame_egui::{GridContext, Theme};
use endgame_grid::dynamic;
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    coord1: Option<dynamic::Coord>,
    coord2: Option<dynamic::Coord>,
}

impl Ui {
    fn add(&self) -> Option<dynamic::Coord> {
        // TODO Can we extend dynamic to support ModuleCoord?
        self.coord1.zip(self.coord2).map(|(c1, c2)| match (c1, c2) {
            (dynamic::Coord::Square(a), dynamic::Coord::Square(b)) => (a + b).into(),
            (dynamic::Coord::Hex(a), dynamic::Coord::Hex(b)) => (a + b).into(),
            _ => unreachable!("Mismatched coordinate kinds {} vs {}", c1.kind(), c2.kind()),
        })
    }
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::CoordinateAddition
    }
    fn label(&self) -> &'static str {
        "Module Addition"
    }

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn supports_grid_kind(&self, kind: dynamic::Kind) -> bool {
        kind != dynamic::Kind::Triangle
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on two grid cells to experiment with coordination addition.  Note, that as \
             triangular coordinates do not satisfy the requirements to be an algebraic module, \
             they do not support addition.\n",
        );

        common::binary_coordinates_labels(ui, "first", &self.coord1, "second", &self.coord2);

        if let Some(sum) = self.add() {
            wrapped_string(ui, format!("Sum coordinate: {sum}\n"));
        }
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        // No-op if the grid kind does not support ModuleCoord.  Also, reset
        // the selected coordinates.
        if !grc.szg.kind().is_modular() {
            self.coord1 = None;
            self.coord2 = None;
            return;
        }

        common::binary_coordinates_select(ctx, &mut self.coord1, &mut self.coord2);

        let Some(coord1) = self.coord1 else { return };

        grc.render_coord_cell(&coord1, &common::SOURCE_CELL_SPEC, None::<&str>);

        let Some(coord2) = self.coord2 else { return };

        grc.render_coord_cell(&coord2, &common::SOURCE_CELL_SPEC, None::<&str>);

        let Some(coord3) = self.add() else { return };

        grc.render_coord_cell(&coord3, &common::TARGET_CELL_SPEC, None::<&str>);

        grc.render_hollow_arrow_coords(&coord1, &coord3, common::HOLLOW_ARROW_STYLE.deref(), None);
        grc.render_hollow_arrow_coords(&coord2, &coord3, common::HOLLOW_ARROW_STYLE.deref(), None);
    }
}
