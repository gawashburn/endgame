use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use eframe::epaint::Color32;
use endgame_direction::Direction;
use endgame_egui::{GridContext, LabelStyle, SolidArrowStyle, Theme};
use endgame_grid::{dynamic, Coord, DirectionType, SizedGrid};
use std::f32::consts::TAU;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Measurement {
    #[default]
    Inradius,
    Circumradius,
    Edge,
    // TODO Draw angles between vertices.
    //VertexAngle,
}

#[derive(Default)]
pub struct Ui {
    measurement: Measurement,
    source: Option<dynamic::Coord>,
    direction: u8,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::CellMeasurements
    }

    fn label(&self) -> &'static str {
        "Cell Measurements"
    }

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on a grid cell to select visualizations of its  measurements.\n",
        );

        common::unary_coordinate_label(ui, &self.source);

        use Measurement::*;
        ui.radio_value(&mut self.measurement, Inradius, "Inradius");
        ui.radio_value(&mut self.measurement, Circumradius, "Circumradius");
        ui.radio_value(&mut self.measurement, Edge, "Edge");

        if self.measurement == Edge {
            common::direction_widget(ui, &mut self.direction);
        }

        let dir = Direction::from_u8(self.direction);
        if let Some(coord) = self.source
            && self.measurement == Edge
            && !coord.allowed_direction(DirectionType::Face, dir)
        {
            common::wrapped_string(
                ui,
                format!("This coordinate does not have an edge in the {dir} direction."),
            );
        }

        // TODO
        //ui.radio_value(&mut self.measurement, VertexAngle, "Vertex Angle");
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;
        let szg = &grc.szg;

        common::unary_coordinate_select(ctx, &mut self.source);

        let Some(coord) = self.source else {
            return
        };

        let screen = grc
            .transform
            .transform_pos(endgame_egui::coord_to_egui_pos2(&coord, szg));

        let vertices = szg.vertices(&coord);

        let style = SolidArrowStyle {
            color: Color32::BLACK,
            width: 2.0,
            to_head: true,
            from_head: true,
            label: Some(LabelStyle {
                font_size: 14.0,
                color: Color32::BLACK,
                add_shadow: Some(Color32::LIGHT_GRAY),
            }),
        };

        match self.measurement {
            Measurement::Inradius => {
                let label = format!("{:.2}", szg.inradius());
                let center = szg.grid_to_screen(&coord);
                let rel_v0 = vertices[0] - center;
                let rel_v1 = vertices[1] - center;
                let mid_angle =
                    (rel_v0.to_angle().rem_euclid(TAU) + rel_v1.to_angle().rem_euclid(TAU)) / 2.0;

                let midpoint = center + (glam::Vec2::from_angle(mid_angle) * szg.inradius());
                let endpoint = grc
                    .transform
                    .transform_pos(endgame_egui::glam_vec2_to_egui_pos2(midpoint));

                endgame_egui::render_arrow(
                    screen,
                    endpoint,
                    &style,
                    Some(label.as_str()),
                    &grc.painter,
                );

                grc.painter.circle_stroke(
                    screen,
                    szg.inradius(),
                    egui::Stroke {
                        width: 2.0,
                        color: Color32::BLACK,
                    },
                );
            }
            Measurement::Circumradius => {
                let label = format!("{:.2}", szg.circumradius());

                let endpoint = grc
                    .transform
                    .transform_pos(endgame_egui::glam_vec2_to_egui_pos2(vertices[0]));
                endgame_egui::render_arrow(
                    screen,
                    endpoint,
                    &style,
                    Some(label.as_str()),
                    &grc.painter,
                );

                grc.painter.circle_stroke(
                    screen,
                    szg.circumradius(),
                    egui::Stroke {
                        width: 2.0,
                        color: Color32::BLACK,
                    },
                );
            }
            Measurement::Edge => {
                let edge_map = szg.edges(&coord);
                let dir = Direction::from_u8(self.direction);

                // If the coordinate has this edge, draw the measurement.
                if let Some((start, end)) = edge_map.get(&dir) {
                    let v0 = grc
                        .transform
                        .transform_pos(endgame_egui::glam_vec2_to_egui_pos2(*start));
                    let v1 = grc
                        .transform
                        .transform_pos(endgame_egui::glam_vec2_to_egui_pos2(*end));
                    let label = format!("{:.2}", (end - start).length());

                    endgame_egui::render_arrow(v0, v1, &style, Some(label.as_str()), &grc.painter);
                } else {
                    let pos = grc
                        .transform
                        .transform_pos(endgame_egui::coord_to_egui_pos2(&coord, &grc.szg));
                    endgame_egui::render_disallowed(
                        pos, //endgame_egui::coord_to_egui_pos2(&coord, szg),
                        szg.inradius() * 0.66,
                        8.0 * (szg.inradius() / 64.0),
                        &grc.painter,
                    );
                }
            }
        }
    }
}
