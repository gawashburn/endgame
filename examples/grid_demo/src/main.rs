extern crate core;

mod allowed_directions;
mod angle_to_direction;
mod axis_iterator;
mod cell_measurements;
mod common;
mod coordinates;
mod direction_iterator;
mod grid_rectangle;
mod module_addition;
mod module_multiplication;
mod path_iterator;
mod reflection;
mod rotation;
mod shapes;

use egui::emath::RectTransform;
use egui::text::LayoutJob;
use egui::{Align, FontId, Id, Layout, Painter, Pos2, Rect, Sense, Ui};
use endgame_egui::{CellStyle, Theme};
use endgame_grid::dynamic;
use endgame_grid::SizedGrid;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

/// An enumeration of all the different examples currently
/// implemented.  The numeric values are used to determine
/// the order in which they appear in the selection list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum GridExample {
    #[default]
    Coordinates = 1,
    GridRectangle = 2,
    CellMeasurements = 3,
    AllowedDirections = 4,
    AngleToDirection = 5,
    DirectionIterator = 6,
    AxisIterator = 7,
    PathIterator = 8,
    Reflection = 9,
    Rotation = 10,
    Shapes = 11,
    CoordinateAddition = 12,
    CoordinateMultiplication = 13,
}

//////////////////////////////////////////////////////////////////////////////

/// This trait is used to abstract away from exmaple specific behavior.
/// Note, this trait must be dyn compatible currently.
trait ExampleUi {
    // Helper to instantiate a boxed RefCell of the ExampleUi.
    fn boxed() -> RefCell<Box<dyn ExampleUi>>
    where
        Self: Default + 'static,
    {
        RefCell::new(Box::new(Self::default()))
    }

    fn example(&self) -> GridExample;

    fn label(&self) -> &str;

    /// Does this example support the given grid kind?
    /// By default, most examples support all grid kinds.
    /// The primary exception are those that rely on operations
    /// only defined for `ModuleCoord`s.
    fn supports_grid_kind(&self, _kind: dynamic::Kind) -> bool {
        true
    }

    /// Should the base grid be rendered for this example?
    /// Defaults to being rendered.
    fn render_grid(&self) -> bool {
        true
    }

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::Map.cell_style(coord, dark_mode)
    }

    /// This method can be used to add in any additional controls
    /// to the panel specific to the example.
    /// By default, no additional controls are added.
    fn controls(&mut self, _demo: &GridDemo, _ui: &mut Ui) {}

    /// This method can be used to render additional overlay
    /// visuals specific to the example.
    /// By default, no additional overlay is rendered.
    fn render_overlay(
        &mut self,
        _demo: &GridDemo,
        _dszg: &dynamic::SizedGrid,
        _transform: &RectTransform,
        _painter: &Painter,
    ) {
    }
}

//////////////////////////////////////////////////////////////////////////////

/// The demo application state.
struct GridDemo {
    /// The currently selected grid kind.
    grid_kind: dynamic::Kind,
    /// The current grid inradius length.
    grid_size: f32,
    /// The example is currently selected.
    example: GridExample,
    /// The state for each of the different example UIs.
    example_uis: HashMap<GridExample, RefCell<Box<dyn ExampleUi>>>,
    /// The current panning offset, if any.
    offset: Option<Pos2>,
    /// The current mouse position in screen coordinates.
    mouse: Pos2,
    /// A queue of mouse clicks in screen coordinates.
    clicks: RefCell<Box<VecDeque<Pos2>>>,
    /// About dialog state.
    about_dialog_open: bool,
}

impl Default for GridDemo {
    fn default() -> Self {
        GridDemo {
            grid_kind: dynamic::Kind::Square,
            grid_size: 32.0,
            example: GridExample::Coordinates,
            example_uis: HashMap::from_iter(
                [
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
                .into_iter()
                .map(|cell| ({ cell.borrow().example() }, cell)),
            ),
            offset: None,
            mouse: Pos2::ZERO,
            clicks: RefCell::<Box<VecDeque<Pos2>>>::new(Box::new(VecDeque::new())),
            about_dialog_open: false,
        }
    }
}

impl GridDemo {
    fn render_panel(&mut self, ui: &mut Ui) {
        if self.about_dialog_open {
            let modal = egui::Modal::new(Id::new("about_modal")).show(ui.ctx(), |ui| {
                ui.heading("grid_demo");
                let mut job = LayoutJob::single_section(
                    "This program exercises the functionality of the endgame_grid \
            library for both pedagogical and debugging purposes.\n"
                        .to_owned(),
                    egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
                );
                job.wrap = egui::text::TextWrapping::default();
                ui.label(job);
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
        let mut job = LayoutJob::single_section(
            "Click and drag with the mouse to pan the view.\n\n\
            The scroll wheel can also adjust the grid size."
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();
        ui.label(job);

        if let Some(ref_cell) = self.example_uis.get(&self.example) {
            ui.separator();
            ref_cell.borrow_mut().controls(self, ui);
        };
    }

    fn render_view(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());

        // Check if there was a scroll-wheel delta if the mouse inside the
        // response rectangle.
        let delta = ui.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } if response.contains_pointer() => Some(*delta),
                _ => None,
            })
        });

        // Apply the scroll-wheel delta to the grid size.
        if let Some(delta) = delta {
            self.grid_size = self.grid_size + delta.y;
        }

        // Construct a dynamic sized grid based upon the current selected
        // grid kind and size.
        let dszg = dynamic::SizedGrid::new(self.grid_kind, self.grid_size);

        if self.offset.is_none() {
            // Center the grid initially.
            let center = response.rect.center() * -1.0;
            let screen_center = dszg.grid_to_screen(&dynamic::Coord::origin(self.grid_kind));
            self.offset = Some((center - Pos2::new(screen_center.x, screen_center.y)).to_pos2());
        }

        // Check if the mouse button was dragged, and if so adjust the
        // panning offset.
        let prd = ui.interact(response.rect, response.id, Sense::drag());
        if prd.dragged() {
            if prd.dragged() {
                self.offset = Some(self.offset.unwrap() + prd.drag_delta());
            }
        }

        // Construct a transform that maps from the viewport specified by the
        // panning offset and the size of the painting rectangle to the screen
        // coordinates.  Note that we do not want to use the minimum,
        // coordinate of the rect as the target, as its upper left corner is
        // always zero for the purposes of painting.
        let to_screen_transform = RectTransform::from_to(
            Rect::from_min_size(self.offset.unwrap(), response.rect.size()),
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
        );

        let prc = ui.interact(response.rect, response.id, Sense::click());
        if prc.clicked() {
            let pos = prc.interact_pointer_pos().unwrap();
            let pos2 = to_screen_transform.inverse().transform_pos(pos);
            self.clicks.get_mut().push_back(pos2.into());
        }

        // Update the current mouse position for use by examples.
        if let Some(pos) = prc.hover_pos() {
            self.mouse = pos;
        }

        let dark_mode = ui.visuals().dark_mode;

        // Clear the background.
        painter.rect_filled(
            Rect {
                min: painter.clip_rect().min,
                max: painter.clip_rect().max,
            },
            0.0,
            if dark_mode {
                // TODO Annoying that lazy_static doesn't handle types that
                //   implement Copy well.
                common::DARK_BACKGROUND.clone()
            } else {
                common::LIGHT_BACKGROUND.clone()
            },
        );

        // Possibly render the base grid, and then any example specific overlay
        // visualization.
        if let Some(ref_cell) = self.example_uis.get(&self.example) {
            // If the offset isn't yet defined, we can't render anything.
            if self.offset.is_none() {
                return;
            }

            let mut example_ui = ref_cell.borrow_mut();
            let theme_fun =
                |coord: &dynamic::Coord, dark_mode: bool| example_ui.cell_theme(coord, dark_mode);
            // Render the base grid, if this particular example wants it.
            if example_ui.render_grid() {
                endgame_egui::render_grid_rect(
                    &dszg,
                    theme_fun,
                    |coord| Some(format!("{}", coord)),
                    dark_mode,
                    true, /* clip to rect */
                    endgame_egui::egui_pos2_to_glam_vec2(response.rect.min),
                    endgame_egui::egui_pos2_to_glam_vec2(response.rect.max),
                    self.offset.unwrap(),
                    &to_screen_transform,
                    &painter,
                );
            }
            // Render example specific visuals.
            example_ui.render_overlay(self, &dszg, &to_screen_transform, &painter);
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

impl eframe::App for GridDemo {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
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
}

//////////////////////////////////////////////////////////////////////////////

use wasm_bindgen::prelude::*;

// Boilerplate essentially copied straight from eframe examples.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "endgame library grid demo",
        native_options,
        Box::new(|_| Ok(Box::<GridDemo>::default())),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::<GridDemo>::default())),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
