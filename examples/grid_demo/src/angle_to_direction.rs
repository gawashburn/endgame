use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use egui::{Color32, Sense};
use endgame_egui::{
    alter_segment_length, coord_to_egui_pos2, egui_pos2_to_glam_vec2, glam_vec2_to_egui_pos2, GridContext,
    HollowArrowStyle, LabelStyle, SolidArrowStyle, Theme,
};
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

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on a grid cell to see how the vector from the selected coordinate's center to \
             the mouse would be mapped to a direction of the given type. As long there is \
             sufficient space, the difference in angle will also be reported.\n",
        );

        common::unary_coordinate_label(ui, &self.source);

        common::direction_type_widget(ui, &mut self.dir_type);
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        common::unary_coordinate_select(ctx, &mut self.source);

        let Some(coord) = self.source else { return };

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

        let start_screen = grc
            .transform
            .transform_pos(coord_to_egui_pos2(&coord, &grc.szg));

        let prc = ctx
            .ui
            .interact(ctx.response.rect, ctx.response.id, Sense::click());
        let Some(end_screen) = prc.hover_pos() else {
            return;
        };

        let (start, end) = alter_segment_length(
            egui_pos2_to_glam_vec2(start_screen),
            egui_pos2_to_glam_vec2(end_screen),
            20.0,
            -10.0,
        );

        // Ensure that arrow scales the same way the coordinate one does.
        let width = 12.0 * (grc.szg.inradius() / 64.0);
        let mouse_style = HollowArrowStyle {
            width: width.min(12.0),
            ..common::HOLLOW_ARROW_STYLE.deref().clone()
        };
        endgame_egui::render_hollow_arrow(
            glam_vec2_to_egui_pos2(start),
            glam_vec2_to_egui_pos2(end),
            &mouse_style,
            None,
            &grc.painter,
        );

        let mouse_vec = end - start;
        let angle = mouse_vec.to_angle().rem_euclid(TAU);

        let dir = coord.angle_to_direction(self.dir_type, angle);
        let cell_steps = ((mouse_vec.length() / (2.0 * grc.szg.inradius())) as usize).max(1) + 2;
        let dir_iter = coord.direction_iterator(self.dir_type, dir, ..=cell_steps);
        if let Some(dir_coord) = dir_iter.last() {
            //u_coord.move_in_direction(self.dir_type, dir) {
            let offset_screen = grc
                .transform
                .transform_pos(coord_to_egui_pos2(&dir_coord, &grc.szg));

            grc.render_hollow_arrow_coords(
                &coord,
                &dir_coord,
                common::HOLLOW_ARROW_STYLE.deref(),
                None,
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

            // Arcs smaller than 0.30 radians wind up being degenerate.
            if angle_diff > 0.30 {
                let angle_str = format!("{:.2}°", angle_diff.to_degrees());
                endgame_egui::render_arrow_arc(
                    start_pos,
                    length * 0.75,
                    start_angle + (PI / 32.0),
                    start_angle + angle_diff - (PI / 32.0),
                    &arc_arrow_style,
                    Some(angle_str.as_str()),
                    &grc.painter,
                );
            }
        }
    }
}
