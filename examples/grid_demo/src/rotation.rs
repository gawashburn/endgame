use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::Painter;
use endgame_egui::{render_hollow_arrow_coords, CellStyle, Theme};
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

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to see how the coordinate will rotate \
            around the origin.\n"
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();

        ui.label(job);

        let selection_text = if let Some(coord) = self.source {
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

        ui.radio_value(&mut self.clockwise, true, "Clockwise");
        ui.radio_value(&mut self.clockwise, false, "Counter-clockwise");
    }

    fn render_overlay(
        &mut self,
        demo: &GridDemo,
        dszg: &dynamic::SizedGrid,
        transform: &RectTransform,
        painter: &Painter,
    ) {
        common::unary_coordinates_select(
            dszg,
            demo.grid_kind,
            &mut demo.clicks.borrow_mut(),
            &mut self.source,
        );

        let Some(coord) = self.source else {
            return;
        };

        let mut spec: &CellStyle = common::SOURCE_CELL_SPEC.deref();
        let mut cur_coord = coord;
        loop {
            endgame_egui::render_coord_cell(
                dszg,
                &cur_coord,
                &spec,
                None::<&str>,
                transform,
                painter,
            );
            let next_coord = if self.clockwise {
                cur_coord.rotate_clockwise()
            } else {
                cur_coord.rotate_counterclockwise()
            };
            render_hollow_arrow_coords(
                dszg,
                &cur_coord,
                &next_coord,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
                transform,
                painter,
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
