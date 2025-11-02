use crate::allowed_directions;
use crate::angle_to_direction;
use crate::axis_iterator;
use crate::cell_measurements;
use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;
use crate::coordinates;
use crate::direction_iterator;
use crate::grid_rectangle;
use crate::module_addition;
use crate::module_multiplication;
use crate::path_iterator;
use crate::reflection;
use crate::rotation;
use crate::shapes;
use egui::{Align, Id, Layout, Pos2, Ui};
use endgame_grid::dynamic;
use std::cell::RefCell;
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////////////////////////

/// The demo application state.
pub struct GridDemo {
    /// The currently selected grid kind.
    pub grid_kind: dynamic::Kind,
    /// The current grid inradius length.
    pub grid_size: f32,
    /// The example is currently selected.
    pub example: GridExample,
    /// The state for each of the different example UIs.
    pub example_uis: HashMap<GridExample, RefCell<Box<dyn ExampleUi>>>,
    /// The current panning offset, if any.
    pub offset: Option<Pos2>,
    /// About dialog state.
    pub about_dialog_open: bool,
}

impl Default for GridDemo {
    fn default() -> Self {
        GridDemo {
            grid_kind: dynamic::Kind::Square,
            grid_size: 32.0,
            example: GridExample::Coordinates,
            example_uis: HashMap::from_iter(
                Self::examples()
                    .into_iter()
                    .map(|cell| ({ cell.borrow().example() }, cell)),
            ),
            offset: None,
            about_dialog_open: false,
        }
    }
}

impl GridDemo {
    pub fn examples() -> Vec<RefCell<Box<dyn ExampleUi>>> {
        vec![
            coordinates::Ui::boxed(),
            grid_rectangle::Ui::boxed(),
            module_addition::Ui::boxed(),
            module_multiplication::Ui::boxed(),
            angle_to_direction::Ui::boxed(),
            direction_iterator::Ui::boxed(),
            axis_iterator::Ui::boxed(),
            allowed_directions::Ui::boxed(),
            cell_measurements::Ui::boxed(),
            path_iterator::Ui::boxed(),
            reflection::Ui::boxed(),
            rotation::Ui::boxed(),
            shapes::Ui::boxed(),
        ]
    }

    pub fn run(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("grid_demo_panel")
            .resizable(false)
            .default_width(160.0)
            .min_width(100.0)
            .show(ctx, |ui| {
                self.render_panel(ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_view(ui);
        });
    }

    fn render_panel(&mut self, ui: &mut Ui) {
        if self.about_dialog_open {
            let modal = egui::Modal::new(Id::new("about_modal")).show(ui.ctx(), |ui| {
                ui.heading("grid_demo");
                common::wrapped_str(
                    ui,
                    "This program exercises the functionality of the endgame_grid library for \
                     both pedagogical and debugging purposes.\n",
                );

                ui.heading("Links");
                use egui::special_emojis::GITHUB;
                ui.hyperlink_to(
                    format!("{GITHUB} github.com/gawashburn/endgame"),
                    "https://github.com/gawashburn/endgame",
                );
                ui.hyperlink_to(
                    format!(
                        "{GITHUB} github.com/gawashburn/endgame/tree/master/crates/endgame_grid"
                    ),
                    "https://github.com/gawashburn/endgame/tree/master/crates/endgame_grid",
                );
                ui.hyperlink_to(
                    format!(
                        "{GITHUB} github.com/gawashburn/endgame/tree/master/examples/grid_demo"
                    ),
                    "https://github.com/gawashburn/endgame/tree/master/examples/grid_demo",
                );

                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    if ui.button("Close").clicked() {
                        ui.close()
                    }
                });
            });

            if modal.should_close() {
                self.about_dialog_open = false
            }
        }

        egui::Grid::new("grid_demo_controls")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    if ui.button("About...").clicked() {
                        self.about_dialog_open = true
                    }
                });

                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    if ui.button("Center view on origin").clicked() {
                        self.offset = None;
                    }
                });

                ui.end_row();
                ui.label("Grid kind");
                //  ui.end_row();
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.grid_kind, dynamic::Kind::Square, "Square");
                    ui.radio_value(&mut self.grid_kind, dynamic::Kind::Hex, "Hex");
                    ui.radio_value(&mut self.grid_kind, dynamic::Kind::Triangle, "Triangle");
                });
                ui.end_row();
                ui.label("Grid inradius length");
                //ui.end_row();
                ui.add(egui::Slider::new(&mut self.grid_size, 16.0..=256.0));
                ui.end_row();
                ui.label("Examples:");
                ui.end_row();

                let mut uis: Vec<(&GridExample, &RefCell<Box<dyn ExampleUi>>)> =
                    self.example_uis.iter().collect();
                // Sort so the example order is consistent across application runs.
                uis.sort_by(|(ex1, _), (ex2, _)| (**ex1 as u8).cmp(&(**ex2 as u8)));

                // Only show those examples that support the current grid kind.
                let visible_examples = uis
                    .iter()
                    .filter(|(_, cell)| cell.borrow().supports_grid_kind(self.grid_kind))
                    .collect::<Vec<_>>();

                // Chunk the examples into two columns to save some vertical space.
                for chunk in visible_examples.chunks(2) {
                    for (ex, cell) in chunk {
                        let example_ui = cell.borrow();
                        ui.radio_value(&mut self.example, **ex, example_ui.label());
                    }
                    ui.end_row();
                }
            });

        ui.separator();
        common::wrapped_str(
            ui,
            "Click and drag with the mouse to pan the view.\n\nThe scroll wheel can also adjust \
             the grid size.",
        );

        if let Some(ref_cell) = self.example_uis.get(&self.example) {
            ui.separator();
            ref_cell.borrow_mut().controls(self.grid_kind, ui);
        };
    }

    fn render_view(&mut self, ui: &mut Ui) {
        if let Some(ref_cell) = self.example_uis.get(&self.example) {
            let mut example_ui = ref_cell.borrow_mut();
            let theme = example_ui.cell_theme();

            let mut gv = endgame_egui::GridArea::new(
                &mut self.grid_size,
                &mut self.offset,
                |inradius| dynamic::SizedGrid::new(self.grid_kind, inradius),
                None,
                None,
                12.0,
                128.0,
                example_ui.render_grid(),
                true, // Allow scroll-wheel zooming
                true, // Allow panning with mouse dragging
                true, // Clear the background before drawing.
                *common::LIGHT_BACKGROUND,
                *common::DARK_BACKGROUND,
                |coord: &dynamic::Coord, dark_mode: bool| theme.cell_style(coord, dark_mode),
                |coord| Some(format!("{}", coord)),
            );
            gv.render(ui, |gc| {
                example_ui.render_overlay(&gc);
            });
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

impl eframe::App for GridDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.run(ctx);
    }
}