use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;
use endgame_egui::{GridContext, HollowArrowStyle, LabelStyle, Theme};
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

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on a grid cell to see the allowed traversal directions of the given type.\n",
        );

        common::unary_coordinate_label(ui, &self.source);

        common::direction_type_widget(ui, &mut self.dir_type);
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        common::unary_coordinate_select(ctx, &mut self.source);

        let Some(coord) = self.source else { return };

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
                grc.render_hollow_arrow_coords(
                    &coord,
                    &move_coord,
                    &arrow_style,
                    Some(dir.short_name()),
                );
            }
        }
    }
}
