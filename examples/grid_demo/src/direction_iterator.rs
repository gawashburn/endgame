use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::emath::RectTransform;
use egui::Painter;
use endgame_direction::Direction;
use endgame_egui::{render_hollow_arrow_coords, CellStyle, Theme};
use endgame_grid::{dynamic, Coord, DirectionType, SizedGrid};
use std::ops::Deref;

pub struct Ui {
    direction: u8,
    steps: usize,
    dir_type: DirectionType,
    source: Option<dynamic::Coord>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            direction: Direction::North as u8,
            steps: 2,
            dir_type: DirectionType::Face,
            source: None,
        }
    }
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::DirectionIterator
    }

    fn label(&self) -> &'static str {
        "Direction Iterator"
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to experiment with traversals along \
             the different directions from the selected coordinate.\n"
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

        egui::Grid::new("direction_type")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                common::direction_type_widget(ui, &mut self.dir_type);
            });

        let dir = Direction::from_u8(self.direction);
        common::direction_widget(ui, &mut self.direction);
        ui.add(egui::Slider::new(&mut self.steps, 1..=16).text("Steps"));
        if let Some(coord) = self.source
            && !coord.allowed_direction(self.dir_type, dir)
        {
            let mut job = LayoutJob::single_section(
                format!(
                    "This coordinate cannot move in the {dir} \
                direction for the selected direction type."
                ),
                egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
            );
            job.wrap = egui::text::TextWrapping::default();
            ui.label(job);
        }
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

        endgame_egui::render_coord_cell(
            dszg,
            &coord,
            &common::SOURCE_CELL_SPEC,
            None::<&str>,
            transform,
            painter,
        );

        let dir = Direction::from_u8(self.direction);
        if !coord.allowed_direction(self.dir_type, dir) {
            endgame_egui::render_disallowed(
                endgame_egui::coord_to_egui_pos2(&coord, dszg),
                dszg.inradius() * 0.66,
                8.0 * (demo.grid_size / 64.0),
                transform,
                painter,
            );
            return;
        }

        let mut prev_coord = None;
        for coord in coord.direction_iterator(self.dir_type, dir, ..self.steps + 1) {
            if let Some(prev) = prev_coord {
                render_hollow_arrow_coords(
                    dszg,
                    &prev,
                    &coord,
                    common::HOLLOW_ARROW_STYLE.deref(),
                    None,
                    transform,
                    painter,
                );
            }
            prev_coord = Some(coord.clone());
        }

        if let Some(last_coord) = prev_coord {
            endgame_egui::render_coord_cell(
                dszg,
                &last_coord,
                &common::TARGET_CELL_SPEC,
                None::<&str>,
                transform,
                painter,
            )
        }
    }
}
