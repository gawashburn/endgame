use crate::common;
use crate::grid_demo::{ExampleUi, GridDemo, GridExample};

use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::{Color32, FontId};
use egui::Painter;
use endgame_direction::Direction;
use endgame_egui::{CellStyle, GridContext, LabelStyle, SolidArrowStyle, Theme};
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

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to select visualizations of its  \
             measurements.\n"
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

        use Measurement::*;
        ui.radio_value(&mut self.measurement, Inradius, "Inradius");
        ui.radio_value(&mut self.measurement, Circumradius, "Circumradius");
        ui.radio_value(&mut self.measurement, Edge, "Edge");

        if self.measurement == Edge {
            common::direction_widget(ui, &mut self.direction);
        }

        let dir = Direction::from_u8(self.direction);
        if let Some(coord) = self.source
            && self.measurement == Measurement::Edge
            && !coord.allowed_direction(DirectionType::Face, dir)
        {
            let mut job = LayoutJob::single_section(
                format!("This coordinate does not have an edge in the {dir} direction."),
                egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
            );
            job.wrap = egui::text::TextWrapping::default();
            ui.label(job);
        }

        // TODO
        //ui.radio_value(&mut self.measurement, VertexAngle, "Vertex Angle");
    }

    fn render_overlay(
        &mut self,
        _ctx: &GridContext<dynamic::SizedGrid>,
        //demo: &GridDemo,
        _dszg: &dynamic::SizedGrid,
        transform: &RectTransform,
        _painter: &Painter,
    ) {
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

        let screen = transform.transform_pos(endgame_egui::coord_to_egui_pos2(&coord, _dszg));

        let vertices = _dszg.vertices(&coord);

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
                let label = format!("{:.2}", _dszg.inradius());
                let center = _dszg.grid_to_screen(&coord);
                let rel_v0 = vertices[0] - center;
                let rel_v1 = vertices[1] - center;
                let mid_angle =
                    (rel_v0.to_angle().rem_euclid(TAU) + rel_v1.to_angle().rem_euclid(TAU)) / 2.0;

                let midpoint = center + (glam::Vec2::from_angle(mid_angle) * _dszg.inradius());
                let endpoint =
                    transform.transform_pos(endgame_egui::glam_vec2_to_egui_pos2(midpoint));

                endgame_egui::render_arrow(screen, endpoint, &style, Some(label.as_str()), _painter);

                _painter.circle_stroke(
                    screen,
                    _dszg.inradius(),
                    egui::Stroke {
                        width: 2.0,
                        color: Color32::BLACK,
                    },
                );
            }
            Measurement::Circumradius => {
                let label = format!("{:.2}", _dszg.circumradius());

                let endpoint =
                    transform.transform_pos(endgame_egui::glam_vec2_to_egui_pos2(vertices[0]));
                endgame_egui::render_arrow(screen, endpoint, &style, Some(label.as_str()), _painter);

                _painter.circle_stroke(
                    screen,
                    _dszg.circumradius(),
                    egui::Stroke {
                        width: 2.0,
                        color: Color32::BLACK,
                    },
                );
            }
            Measurement::Edge => {
                let edge_map = _dszg.edges(&coord);
                let dir = Direction::from_u8(self.direction);

                // If the coordinate has this edge, draw the measurement.
                if let Some((start, end)) = edge_map.get(&dir) {
                    let v0 = transform.transform_pos(endgame_egui::glam_vec2_to_egui_pos2(*start));
                    let v1 = transform.transform_pos(endgame_egui::glam_vec2_to_egui_pos2(*end));
                    let label = format!("{:.2}", (end - start).length());

                    endgame_egui::render_arrow(v0, v1, &style, Some(label.as_str()), _painter);
                } else {
                    let pos = transform.transform_pos(endgame_egui::coord_to_egui_pos2(&coord, _dszg));
                    endgame_egui::render_disallowed(
                        endgame_egui::coord_to_egui_pos2(&coord, _dszg),
                        _dszg.inradius() * 0.66,
                        8.0 * (_ctx.szg.inradius() / 64.0),
                        _painter,
                    );
                    return;
                }
            }
        }
    }
}
