use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;
use eframe::emath::Pos2;
use eframe::epaint::Color32;
use endgame_egui::{egui_pos2_to_glam_vec2, CellStyle, GridContext, Theme};
use endgame_grid::dynamic;

#[derive(Default)]
pub struct Ui {
    x_margin: f32,
    y_margin: f32,
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::GridRectangle
    }

    fn label(&self) -> &'static str {
        "Grid Rectangle"
    }

    fn render_grid(&self) -> bool {
        false
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(ui, "This example is for illustrating the support for \
            tessellating a rectangle with grid.  Normally, we would clip the grid cells \
            against the rectangle, but to better examine how well the \
            tesselation performs, we show all cells in full. \n\n\
            Adjust the margins to visualize the tesslation on differently \
            sized rectangles.\n");

        ui.add(egui::Slider::new(&mut self.x_margin, 0.0..=512.0).text("X Margin"));
        ui.add(egui::Slider::new(&mut self.y_margin, 0.0..=512.0).text("Y Margin"));
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;

        let clip_rect = grc.painter.clip_rect();

        let min = egui::pos2(
            clip_rect.min.x + self.x_margin,
            clip_rect.min.y + self.y_margin,
        );
        let max = egui::pos2(
            clip_rect.max.x - self.x_margin,
            clip_rect.max.y - self.y_margin,
        );

        fn theme_fun(coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
            Theme::Map.cell_style(coord, dark_mode)
        }

        // Render the restricted grid.
        grc.render_grid_rect(
            theme_fun,
            |coord| Some(format!("{}", coord)),
            false, /* clip to rect */
            // TODO It can be a bit confusing that grid_rect coordinates are relative to the
            //   view while the clipping rectangle is relative to the overall window.
            //   Investigate revising if it would more consistent with egui.
            egui_pos2_to_glam_vec2(Pos2::new(self.x_margin, min.y)),
            egui_pos2_to_glam_vec2(Pos2::new(clip_rect.width() - self.x_margin, max.y)),
        );

        // Render the translucent overlay of the margins.
        grc.painter.rect_filled(
            egui::Rect { min, max },
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 0, 32),
        );
    }
}
