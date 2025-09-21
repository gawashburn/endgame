use crate::{ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::{Color32, FontId};
use egui::Painter;
use endgame_egui::{CellStyle, GridContext, Theme};
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

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "This example is for illustrating the support for tessellating \
            a rectangle with grid.  Normally, we would clip the grid cells \
            against the rectangle, but to better examine how well the \
            tesselation performs, we show all cells in full. \n\n\
            Adjust the margins to visualize the tesslation on differently \
             sized rectangles.\n"
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();

        ui.label(job);

        ui.add(egui::Slider::new(&mut self.x_margin, 0.0..=512.0).text("X Margin"));
        ui.add(egui::Slider::new(&mut self.y_margin, 0.0..=512.0).text("Y Margin"));
    }

    fn render_overlay(
        &mut self,
        _ctx: &GridContext<dynamic::SizedGrid>,
        //demo: &GridDemo,
        _dszg: &dynamic::SizedGrid,
        _transform: &RectTransform,
        _painter: &Painter,
    ) {
        let min = egui::pos2(
            _painter.clip_rect().min.x + self.x_margin,
            _painter.clip_rect().min.y + self.y_margin,
        );
        let max = egui::pos2(
            _painter.clip_rect().max.x - self.x_margin,
            _painter.clip_rect().max.y - self.y_margin,
        );

        fn theme_fun(coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
            Theme::Map.cell_style(coord, dark_mode)
        }

        // FIX!!
        /*
        // Render the base grid, if this particular example wants it.
        endgame_egui::render_grid_rect(
            dszg,
            theme_fun,
            |coord| Some(format!("{}", coord)),
            false, /* dark mode */
            false, /* clip to rect */
            endgame_egui::egui_pos2_to_glam_vec2(min),
            endgame_egui::egui_pos2_to_glam_vec2(max),
            demo.offset.unwrap(),
            &transform,
            &painter,
        );
        
         */

        _painter.rect_filled(
            egui::Rect { min, max },
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 0, 32),
        );
    }
}
