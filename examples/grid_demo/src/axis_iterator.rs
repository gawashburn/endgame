use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use eframe::epaint::text::LayoutJob;
use eframe::epaint::FontId;
use egui::Pos2;
use endgame_egui::{egui_pos2_to_glam_vec2, CellBorderStyle, CellStyle, GridContext, Theme};
use endgame_grid::{dynamic, Coord, SizedGrid};
use std::ops::Deref;

pub struct Ui {
    axis: Option<dynamic::Axes>,
    positive: bool,
    steps: usize,
    source: Option<dynamic::Coord>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            axis: None,
            positive: true,
            steps: 2,
            source: None,
        }
    }
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::AxisIterator
    }

    fn label(&self) -> &'static str {
        "Axis Iterator"
    }
    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Click on a grid cell to experiment with traversals along \
             the different grid axes from the selected coordinate.\n\n\
             To help visualize the axes, the cells along each axis are highlighted \
                in a different color.\n"
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

        common::axis_widget(ui, &mut self.axis, grid_kind);

        ui.checkbox(&mut self.positive, "Positive axis direction");
        ui.add(egui::Slider::new(&mut self.steps, 1..=16).text("Steps"));
    }

    fn render_overlay(
        &mut self,
        ctx: &GridContext<dynamic::SizedGrid>,
    ) {
        let grc = &ctx.grc;

        common::unary_coordinate_select(
            ctx,
            &mut self.source,
        );

        let Some(coord) = self.source else {
            return
        };

        let base_style = CellStyle {
            fill_color: None,
            border: CellBorderStyle::none(),
            label: None,
        };

        let clip_rect = grc.painter.clip_rect();

        let grid_offset = grc.panning_offset.to_vec2();
        let min = egui_pos2_to_glam_vec2(grid_offset.to_pos2());
        let view_max = Pos2::new(clip_rect.width(), clip_rect.max.y);
        let max = egui_pos2_to_glam_vec2(view_max + grid_offset);

        for (&axis, color) in grc.szg.kind().axes().iter().zip(common::AXES_COLORS.into_iter()) {
            for pos_neg in [true, false] {
                // Skip first coordinate so we do not overdraw the source cell
                // to the point that it is opaque.
                for c in coord.axis_iterator(axis, pos_neg, ..).skip(1) {
                    // TODO Doesn't work in the case that coord is not within the clipping rect.
                    //   Need to do a ray-rectangle intersection test for the axis.
                    if !grc.szg.coord_intersects_rect(&c, min, max) {
                        break;
                    }
                    grc.render_coord_cell(
                        &c,
                        &CellStyle {
                            fill_color: Some(color.linear_multiply(0.33)),
                            ..base_style.clone()
                        },
                        None::<&str>,
                    );
                }
            }
        }

        let mut prev_coord = None;
        for coord in coord.axis_iterator(
            self.axis.expect("Should have an axis"),
            self.positive,
            ..self.steps + 1,
        ) {
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
    }
}
