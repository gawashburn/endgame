use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::{Painter, Sense};
use endgame_egui::{egui_pos2_to_coord, render_hollow_arrow_coords, CellStyle, GridContext, Theme};
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
        _ctx: &GridContext<dynamic::SizedGrid>,
        _dszg: &dynamic::SizedGrid,
        _transform: &RectTransform,
        _painter: &Painter,
    ) {
        let prc = _ctx.ui.interact(_ctx.response.rect, _ctx.response.id, Sense::click());
        if prc.clicked() {
            let pos = prc.interact_pointer_pos().unwrap();
            let pos2 = _ctx.to_screen_transform.inverse().transform_pos(pos);
            self.source = Some(egui_pos2_to_coord(pos2, &_ctx.szg));
        }

        /*
        common::unary_coordinates_select(
            dszg,
            demo.grid_kind,
            &mut demo.clicks.borrow_mut(),
            &mut self.source,
        );

         */

        let Some(coord) = self.source else {
            return;
        };

        let mut spec: &CellStyle = common::SOURCE_CELL_SPEC.deref();
        let mut cur_coord = coord;
        loop {
            endgame_egui::render_coord_cell(
                _dszg,
                &cur_coord,
                &spec,
                None::<&str>,
                _transform,
                _painter,
            );
            let next_coord = if self.clockwise {
                cur_coord.rotate_clockwise()
            } else {
                cur_coord.rotate_counterclockwise()
            };
            render_hollow_arrow_coords(
                _dszg,
                &cur_coord,
                &next_coord,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
                _transform,
                _painter,
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
