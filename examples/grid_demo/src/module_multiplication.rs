use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::Painter;
use endgame_egui::{render_hollow_arrow_coords, CellStyle, GridContext, Theme};
use endgame_grid::dynamic;
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    value: isize,
    coord1: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::CoordinateMultiplication
    }

    fn label(&self) -> &'static str {
        "Module Multiplication"
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn supports_grid_kind(&self, kind: dynamic::Kind) -> bool {
        kind != dynamic::Kind::Triangle
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to experiment with scalar multiplication \
            of coordinates.  Note, that as triangular coordinates \
            do not satisfy the requirements to be an algebraic module, \
            they do not support multiplication.\n"
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();

        ui.label(job);

        let selection_text = if let Some(coord) = self.coord1 {
            format!("Selected coordinate: {}\n", coord)
        } else {
            "No coordinate selected currently\n".to_owned()
        };
        let mut job = LayoutJob::single_section(
            selection_text,
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);

        ui.add(egui::Slider::new(&mut self.value, 0..=8).text("Multiplier"));
    }

    fn render_overlay(
        &mut self,
        _ctx: &GridContext<dynamic::SizedGrid>,
        //demo: &GridDemo,
        _dszg: &dynamic::SizedGrid,
        _transform: &RectTransform,
        _painter: &Painter,
    ) {
        /*
        // No-op if the grid kind does not support ModuleCoord.
        // Also reset the selected coordinates.
        if !demo.grid_kind.is_modular() {
            self.coord1 = None;
            return;
        }

        common::unary_coordinates_select(
            dszg,
            demo.grid_kind,
            &mut demo.clicks.borrow_mut(),
            &mut self.coord1,
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

        // TODO Can we extend dynamic to support ModuleCoord?
        let coord2: dynamic::Coord = match coord1 {
            dynamic::Coord::Square(c) => (c * self.value).into(),
            dynamic::Coord::Hex(c) => (c * self.value).into(),
            _ => unreachable!("Unexpected coordinate kind {}", coord1.kind()),
        };

        endgame_egui::render_coord_cell(
            _dszg,
            &coord2,
            &common::TARGET_CELL_SPEC,
            None::<&str>,
            _transform,
            _painter,
        );

        render_hollow_arrow_coords(
            _dszg,
            &coord1,
            &coord2,
            common::HOLLOW_ARROW_STYLE.deref(),
            None,
            _transform,
            _painter,
        );
    }
}
