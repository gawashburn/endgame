use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::emath::RectTransform;
use egui::Painter;
use endgame_egui::{render_hollow_arrow_coords, CellStyle, HollowArrowStyle, LabelStyle, Theme};
use endgame_grid::{dynamic, Coord, DirectionType};
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    dir_type: DirectionType,
    source: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::AllowedDirections
    }

    fn label(&self) -> &'static str {
        "Allowed Directions"
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to see the allowed traversal \
            directions of the given type.\n"
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

        common::direction_type_widget(ui, &mut self.dir_type);
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

        let arrow_style = HollowArrowStyle {
            label: Some(LabelStyle {
                font_size: 14.0,
                color: egui::Color32::BLACK,
                add_shadow: Some(egui::Color32::LIGHT_GRAY),
            }),
            ..common::HOLLOW_ARROW_STYLE.deref().clone()
        };

        for dir in &coord.allowed_directions(self.dir_type) {
            if let Some(move_coord) = coord.move_in_direction(self.dir_type, dir) {
                render_hollow_arrow_coords(
                    dszg,
                    &coord,
                    &move_coord,
                    &arrow_style,
                    Some(dir.short_name()),
                    transform,
                    painter,
                );
            }
        }
    }
}
