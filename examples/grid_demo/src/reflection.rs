use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::ColorMode::Solid;
use eframe::epaint::{FontId, PathStroke};
use egui::Painter;
use endgame_egui::{render_hollow_arrow_coords, CellStyle, Theme};
use endgame_grid::{dynamic, Coord, SizedGrid};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    axis: Option<dynamic::Axes>,
    source: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::Reflection
    }

    fn label(&self) -> &'static str {
        "Reflection"
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to see how the coordinate will be reflected \
             along different axes of the origin.\n"
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

        if let Some(source) = self.source
            && self.axis.is_some()
        {
            let refl_coord = source.reflect(self.axis.unwrap());
            let refl_text = format!("Reflected coordinate: {}\n", refl_coord);
            let mut job = LayoutJob::single_section(
                refl_text,
                egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
            );
            job.wrap = egui::text::TextWrapping::default();
            ui.label(job);
        }

        common::axis_widget(ui, &mut self.axis, demo.grid_kind);
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

        let axes_specs: HashMap<dynamic::Axes, egui::Color32> = demo
            .grid_kind
            .axes()
            .into_iter()
            .zip(common::AXES_COLORS.into_iter().cycle())
            .collect();

        let origin = dynamic::Coord::origin(demo.grid_kind);
        let origin_vec = dszg.grid_to_screen(&origin);
        let axis_coord = origin
            .axis_iterator(self.axis.unwrap(), true, ..=2)
            .last()
            .unwrap();
        let angle = (dszg.grid_to_screen(&axis_coord) - origin_vec).to_angle() + (PI / 2.0);
        let vec0 = glam::Vec2::from_angle(angle) * 10000.0;
        let vec1 = glam::Vec2::from_angle(angle + PI) * 10000.0;

        // let start = transform.transform_pos(endgame_egui::coord_to_pos2(&origin, dszg));
        let end0 = transform.transform_pos(endgame_egui::glam_vec2_to_egui_pos2(vec0 + origin_vec));
        let end1 = transform.transform_pos(endgame_egui::glam_vec2_to_egui_pos2(vec1 + origin_vec));
        painter.line(
            vec![end0, end1],
            PathStroke {
                width: 6.0,
                color: Solid(*axes_specs.get(&self.axis.unwrap()).unwrap()),
                kind: egui::StrokeKind::Middle,
            },
        );

        let Some(coord) = self.source else {
            return;
        };

        endgame_egui::render_coord_cell(
            dszg,
            &coord,
            &common::SOURCE_CELL_SPEC,
            None::<&str>,
            transform,
            painter,
        );

        let refl_coord = coord.reflect(self.axis.unwrap());
        endgame_egui::render_coord_cell(
            dszg,
            &refl_coord,
            &common::TARGET_CELL_SPEC,
            None::<&str>,
            transform,
            painter,
        );

        render_hollow_arrow_coords(
            dszg,
            &coord,
            &refl_coord,
            common::HOLLOW_ARROW_STYLE.deref(),
            None,
            transform,
            painter,
        );
    }
}
