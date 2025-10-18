use crate::common;
use crate::grid_demo::{ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::Painter;
use endgame_egui::{render_hollow_arrow_coords, CellStyle, GridContext, Theme};
use endgame_grid::dynamic;
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    coord1: Option<dynamic::Coord>,
    coord2: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::CoordinateAddition
    }
    fn label(&self) -> &'static str {
        "Module Addition"
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn supports_grid_kind(&self, kind: dynamic::Kind) -> bool {
        kind != dynamic::Kind::Triangle
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on two grid cells to experiment with coordination addition.  \
            Note, that as triangular coordinates do not satisfy the requirements to \
            be an algebraic module, they do not support addition.\n"
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);

        let coord1_text = if let Some(coord) = self.coord1 {
            format!("First coordinate: {}\n", coord)
        } else {
            "No first coordinate selected currently\n".to_owned()
        };
        let mut job = LayoutJob::single_section(
            coord1_text,
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);
        let coord2_text = if let Some(coord) = self.coord2 {
            format!("Second coordinate: {}\n", coord)
        } else {
            "No second coordinate selected currently\n".to_owned()
        };
        let mut job = LayoutJob::single_section(
            coord2_text,
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);

        if let (Some(c1), Some(c2)) = (self.coord1, self.coord2) {
            let sum: dynamic::Coord = match (c1, c2) {
                (dynamic::Coord::Square(a), dynamic::Coord::Square(b)) => (a + b).into(),
                (dynamic::Coord::Hex(a), dynamic::Coord::Hex(b)) => (a + b).into(),
                _ => unreachable!("Mismatched coordinate kinds {} vs {}", c1.kind(), c2.kind()),
            };
            let sum_text = format!("Sum coordinate: {}\n", sum);
            let mut job = LayoutJob::single_section(
                sum_text,
                egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
            );
            job.wrap = egui::text::TextWrapping::default();
            ui.label(job);
        }
    }

    fn render_overlay(
        &mut self,
        _ctx: &GridContext<dynamic::SizedGrid>,

        // demo: &GridDemo,
        _dszg: &dynamic::SizedGrid,
        _transform: &RectTransform,
        _painter: &Painter,
    ) {
        /*
        // No-op if the grid kind does not support ModuleCoord.  Also, reset
        // the selected coordinates.
        if !demo.grid_kind.is_modular() {
            self.coord1 = None;
            self.coord2 = None;
            return;
        }

        common::binary_coordinates_select(
            dszg,
            demo.grid_kind,
            &mut demo.clicks.borrow_mut(),
            &mut self.coord1,
            &mut self.coord2,
        );

         */

        let Some(coord1) = self.coord1 else {
            return;
        };

        endgame_egui::render_coord_cell(
            _dszg,
            &coord1,
            &common::SOURCE_CELL_SPEC,
            None::<&str>,
            _transform,
            _painter,
        );

        let Some(coord2) = self.coord2 else {
            return;
        };

        endgame_egui::render_coord_cell(
            _dszg,
            &coord2,
            &common::SOURCE_CELL_SPEC,
            None::<&str>,
            _transform,
            _painter,
        );

        // TODO Can we extend dynamic to support ModuleCoord?
        let coord3: dynamic::Coord = match (coord1, coord2) {
            (dynamic::Coord::Square(a), dynamic::Coord::Square(b)) => (a + b).into(),
            (dynamic::Coord::Hex(a), dynamic::Coord::Hex(b)) => (a + b).into(),
            _ => unreachable!(
                "Mismatched coordinate kinds {} vs {}",
                coord1.kind(),
                coord2.kind()
            ),
        };

        endgame_egui::render_coord_cell(
            _dszg,
            &coord3,
            &common::TARGET_CELL_SPEC,
            None::<&str>,
            _transform,
            _painter,
        );

        render_hollow_arrow_coords(
            _dszg,
            &coord1,
            &coord3,
            common::HOLLOW_ARROW_STYLE.deref(),
            None,
            _transform,
            _painter,
        );
        render_hollow_arrow_coords(
            _dszg,
            &coord2,
            &coord3,
            common::HOLLOW_ARROW_STYLE.deref(),
            None,
            _transform,
            _painter,
        );
    }
}
