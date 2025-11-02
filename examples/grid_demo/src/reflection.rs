use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use eframe::epaint::ColorMode::Solid;
use eframe::epaint::PathStroke;
use endgame_egui::{GridContext, Theme};
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

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Click on a grid cell to see how the coordinate will be reflected along different \
             axes of the origin.\n",
        );

        common::unary_coordinate_label(ui, &self.source);

        if let Some(source) = self.source
            && self.axis.is_some()
        {
            let refl_coord = source.reflect(self.axis.unwrap());
            common::wrapped_string(ui, format!("Reflected coordinate: {refl_coord}\n"));
        }

        common::axis_widget(ui, &mut self.axis, grid_kind);
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        common::unary_coordinate_select(ctx, &mut self.source);

        // Draw a line showing the axis to better visualize how coordinates are reflected.
        let axes_colors: HashMap<dynamic::Axes, egui::Color32> = grc
            .szg
            .kind()
            .axes()
            .into_iter()
            .zip(common::AXES_COLORS.into_iter().cycle())
            .collect();

        let origin = dynamic::Coord::origin(grc.szg.kind());
        let origin_vec = grc.szg.grid_to_screen(&origin);
        let axis_coord = origin
            .axis_iterator(self.axis.unwrap(), true, ..=2)
            .last()
            .unwrap();
        let angle = (grc.szg.grid_to_screen(&axis_coord) - origin_vec).to_angle() + (PI / 2.0);
        // TODO Assumes 10000 will be long enough to extend to the extents of the window.
        //  Revise to compute the exact intersection.
        let vec0 = glam::Vec2::from_angle(angle) * 10000.0;
        let vec1 = glam::Vec2::from_angle(angle + PI) * 10000.0;

        // TODO Add GridRenderContext transforms?  Or just add drawing that bakes in transform?

        let end0 = grc
            .transform
            .transform_pos(endgame_egui::glam_vec2_to_egui_pos2(vec0 + origin_vec));
        let end1 = grc
            .transform
            .transform_pos(endgame_egui::glam_vec2_to_egui_pos2(vec1 + origin_vec));
        grc.painter.line(
            vec![end0, end1],
            PathStroke {
                width: 6.0,
                color: Solid(*axes_colors.get(&self.axis.unwrap()).unwrap()),
                kind: egui::StrokeKind::Middle,
            },
        );

        let Some(coord) = self.source else { return };

        grc.render_coord_cell(&coord, &common::SOURCE_CELL_SPEC, None::<&str>);

        let refl_coord = coord.reflect(self.axis.unwrap());
        grc.render_coord_cell(&refl_coord, &common::TARGET_CELL_SPEC, None::<&str>);

        grc.render_hollow_arrow_coords(
            &coord,
            &refl_coord,
            common::HOLLOW_ARROW_STYLE.deref(),
            None,
        );
    }
}
