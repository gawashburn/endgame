use crate::common;
use crate::common::GridExample;
use crate::common::{wrapped_string, ExampleUi};

use endgame_egui::{GridContext, Theme};
use endgame_grid::dynamic;
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    value: isize,
    coord1: Option<dynamic::Coord>,
}

impl Ui {
    fn multiply(&self) -> Option<dynamic::Coord> {
        // TODO Can we extend dynamic to support ModuleCoord?
        self.coord1.map(|c| match c {
            dynamic::Coord::Square(c) => (c * self.value).into(),
            dynamic::Coord::Hex(c) => (c * self.value).into(),
            _ => unreachable!("Unexpected coordinate kind {}", c.kind()),
        })
    }
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::CoordinateMultiplication
    }

    fn label(&self) -> &'static str {
        "Module Multiplication"
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
            "Click on a grid cell to experiment with scalar multiplication of coordinates.  Note, \
             that as triangular coordinates do not satisfy the requirements to be an algebraic \
             module, they do not support multiplication.\n",
        );

        common::unary_coordinate_label(ui, &self.coord1);

        ui.add(egui::Slider::new(&mut self.value, 0..=8).text("Multiplier"));

        if let Some(product) = self.multiply() {
            wrapped_string(ui, format!("Product coordinate: {product}\n"));
        }
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        // No-op if the grid kind does not support ModuleCoord.
        // Also, reset the selected coordinates.
        if !grc.szg.kind().is_modular() {
            self.coord1 = None;
            return;
        }

        common::unary_coordinate_select(ctx, &mut self.coord1);

        let Some(coord1) = self.coord1 else { return };

        grc.render_coord_cell(&coord1, &common::SOURCE_CELL_SPEC, None::<&str>);

        let Some(coord2) = self.multiply() else {
            return;
        };

        grc.render_coord_cell(&coord2, &common::TARGET_CELL_SPEC, None::<&str>);
        grc.render_hollow_arrow_coords(&coord1, &coord2, common::HOLLOW_ARROW_STYLE.deref(), None);
    }
}
