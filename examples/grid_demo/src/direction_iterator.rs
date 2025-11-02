use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use endgame_direction::Direction;
use endgame_egui::{GridContext, Theme};
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

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on a grid cell to experiment with traversals along the different directions \
             from the selected coordinate.\n",
        );

        common::unary_coordinate_label(ui, &self.source);

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
            common::wrapped_string(
                ui,
                format!(
                    "This coordinate cannot move in the {dir} direction for the selected \
                     direction type."
                ),
            )
        }
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        common::unary_coordinate_select(ctx, &mut self.source);

        let Some(coord) = self.source else {
            return
        };

        grc.render_coord_cell(&coord, &common::SOURCE_CELL_SPEC, None::<&str>);

        let dir = Direction::from_u8(self.direction);
        if !coord.allowed_direction(self.dir_type, dir) {
            let pos = grc
                .transform
                .transform_pos(endgame_egui::coord_to_egui_pos2(&coord, &grc.szg));
            endgame_egui::render_disallowed(
                pos,
                grc.szg.inradius() * 0.66,
                8.0 * (ctx.grc.szg.inradius() / 64.0),
                &grc.painter,
            );
            return;
        }

        let mut prev_coord = None;
        for coord in coord.direction_iterator(self.dir_type, dir, ..self.steps + 1) {
            if let Some(prev) = prev_coord {
                grc.render_hollow_arrow_coords(
                    &prev,
                    &coord,
                    common::HOLLOW_ARROW_STYLE.deref(),
                    None,
                );
            }
            prev_coord = Some(coord.clone());
        }

        if let Some(last_coord) = prev_coord {
            grc.render_coord_cell(&last_coord, &common::TARGET_CELL_SPEC, None::<&str>)
        }
    }
}
