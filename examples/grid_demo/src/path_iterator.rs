use crate::common;
use crate::grid_demo::{ExampleUi, GridDemo, GridExample};

use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::emath::RectTransform;
use egui::{Color32, Painter};
use endgame_egui::{render_arrow, render_hollow_arrow_coords, CellStyle, GridContext, SolidArrowStyle, Theme};
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

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on two grid cells to experiment with traversal paths \
             between the coordinates.\n"
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();

        ui.label(job);

        let source_text = if let Some(coord) = self.source {
            format!("Source coordinate: {}\n", coord)
        } else {
            "No source coordinate selected currently\n".to_owned()
        };
        let mut job = LayoutJob::single_section(
            source_text,
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);
        let target_text = if let Some(coord) = self.target {
            format!("Target coordinate: {}\n", coord)
        } else {
            "No target coordinate selected currently\n".to_owned()
        };
        let mut job = LayoutJob::single_section(
            target_text,
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);
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
        common::binary_coordinates_select(
            dszg,
            demo.grid_kind,
            &mut demo.clicks.borrow_mut(),
            &mut self.source,
            &mut self.target,
        );
*/
        let Some(source) = self.source else {
            return;
        };

        endgame_egui::render_coord_cell(
            _dszg,
            &source,
            &common::SOURCE_CELL_SPEC,
            None::<&str>,
            _transform,
            _painter,
        );

        let Some(target) = self.target else {
            return;
        };

        endgame_egui::render_coord_cell(
            _dszg,
            &target,
            &common::TARGET_CELL_SPEC,
            None::<&str>,
            _transform,
            _painter,
        );

        let source_screen =
            _transform.transform_pos(endgame_egui::coord_to_egui_pos2(&source, _dszg));
        let target_screen =
            _transform.transform_pos(endgame_egui::coord_to_egui_pos2(&target, _dszg));

        if source_screen != target_screen {
            let style = SolidArrowStyle {
                color: Color32::GREEN,
                width: 2.0,
                to_head: true,
                from_head: false,
                label: None,
            };
            render_arrow(source_screen, target_screen, &style, None, _painter);
        }

        // If the source is just the target, we just render a single arrow.
        if source == target {
            render_hollow_arrow_coords(
                _dszg,
                &source,
                &target,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
                _transform,
                _painter,
            );
            return;
        }

        let mut prev_coord = None;
        for coord in source.path_iterator(&target) {
            if let Some(prev) = prev_coord {
                render_hollow_arrow_coords(
                    _dszg,
                    &prev,
                    &coord,
                    common::HOLLOW_ARROW_STYLE.deref(),
                    None,
                    _transform,
                    _painter,
                );
            }
            prev_coord = Some(coord);
        }
    }
}
