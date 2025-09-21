use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::emath::RectTransform;
use egui::{Color32, Painter};
use endgame_egui::{alter_segment_length, coord_to_egui_pos2, egui_pos2_to_glam_vec2, glam_vec2_to_egui_pos2, render_hollow_arrow_coords, CellStyle, GridContext, HollowArrowStyle, LabelStyle, SolidArrowStyle, Theme};
use endgame_grid::{dynamic, Coord, DirectionType, SizedGrid};
use std::f32::consts::{PI, TAU};
use std::ops::Deref;

#[derive(Default)]
pub struct Ui {
    dir_type: DirectionType,
    source: Option<dynamic::Coord>,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::AngleToDirection
    }

    fn label(&self) -> &'static str {
        "Angle to Direction"
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to see how the vector from the selected \
             coordinate's center to the mouse would be mapped to a direction \
             of the given type. As long there is sufficient space, the \
             difference in angle will also be reported.\n"
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
        _ctx: &GridContext<dynamic::SizedGrid>,
        //demo: &GridDemo,
        _dszg: &dynamic::SizedGrid,
        _transform: &RectTransform,
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

        let arc_arrow_style = SolidArrowStyle {
            color: Color32::BLACK,
            width: 2.0,
            to_head: true,
            from_head: false,
            label: Some(LabelStyle {
                font_size: 14.0,
                color: Color32::BLACK,
                add_shadow: Some(Color32::GRAY),
            }),
        };
        let start_screen = _transform.transform_pos(coord_to_egui_pos2(&coord, _dszg));

        let (start, end) = alter_segment_length(
            egui_pos2_to_glam_vec2(start_screen),
            egui_pos2_to_glam_vec2(start_screen),
            //   egui_pos2_to_glam_vec2(demo.mouse),
            20.0,
            -10.0,
        );

        // Ensure that arrow scales the same way the coordinate one does.
        let width = 12.0 * (_dszg.inradius() / 64.0);
        let mouse_style = HollowArrowStyle {
            width: width.min(12.0),
            ..common::HOLLOW_ARROW_STYLE.deref().clone()
        };
        endgame_egui::render_hollow_arrow(
            glam_vec2_to_egui_pos2(start),
            glam_vec2_to_egui_pos2(end),
            &mouse_style,
            None,
            _painter,
        );

        let mouse_vec = end - start;
        let angle = mouse_vec.to_angle().rem_euclid(TAU);

        let dir = coord.angle_to_direction(self.dir_type, angle);
        let cell_steps = ((mouse_vec.length() / (2.0 * _dszg.inradius())) as usize).max(1) + 2;
        let dir_iter = coord.direction_iterator(self.dir_type, dir, ..=cell_steps);
        if let Some(dir_coord) = dir_iter.last() {
            //u_coord.move_in_direction(self.dir_type, dir) {
            let offset_screen = _transform.transform_pos(coord_to_egui_pos2(&dir_coord, _dszg));

            render_hollow_arrow_coords(
                _dszg,
                &coord,
                &dir_coord,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
                _transform,
                _painter,
            );

            let start_pos = glam_vec2_to_egui_pos2(start);
            let direction_vec = offset_screen.to_vec2() - start_pos.to_vec2();

            let mut start_angle = angle;
            let mut end_angle = direction_vec.angle().rem_euclid(TAU);
            let mut angle_diff = (end_angle - start_angle).rem_euclid(TAU);
            let length = f32::min(mouse_vec.length(), direction_vec.length());

            if angle_diff > PI {
                angle_diff = TAU - angle_diff;
                std::mem::swap(&mut start_angle, &mut end_angle);
            }

            // Arcs smaller thant 0.30 radians wind up being degenerate.
            if angle_diff > 0.30 {
                let angle_str = format!("{:.2}°", angle_diff.to_degrees());
                endgame_egui::render_arrow_arc(
                    start_pos,
                    length * 0.75,
                    start_angle + (PI / 32.0),
                    start_angle + angle_diff - (PI / 32.0),
                    &arc_arrow_style,
                    Some(angle_str.as_str()),
                    _painter,
                );
            }
        }
    }
}
