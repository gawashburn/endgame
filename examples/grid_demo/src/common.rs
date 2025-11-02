use eframe::epaint::text::LayoutJob;
use eframe::epaint::{Color32, FontId};
use egui::{Sense, Ui};
use endgame_direction::Direction;
use endgame_egui::{
    egui_pos2_to_coord, CellBorderStyle, CellStyle, GridContext, HollowArrowStyle, Theme,
};
use endgame_grid::{dynamic, hex, square, triangle, DirectionType};
use std::cell::RefCell;
//////////////////////////////////////////////////////////////////////////////////////////////////

/// An enumeration of all the different examples currently
/// implemented.  The numeric values are used to determine
/// the order in which they appear in the selection list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridExample {
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

//////////////////////////////////////////////////////////////////////////////////////////////////

/// This trait is used to abstract away from exmaple specific behavior.
/// Note, this trait must be dyn compatible currently.
pub trait ExampleUi {
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

    /// Example specific grid theme.
    // TODO Overhaul themeing.
    fn cell_theme(&self) -> Theme { Theme::Map }

    /// This method can be used to add in any additional controls
    /// to the panel specific to the example.
    /// By default, no additional controls are added.
    fn controls(&mut self, _grid_kind: dynamic::Kind, _ui: &mut Ui) {}

    /// This method can be used to render additional overlay
    /// visuals specific to the example.
    /// By default, no additional overlay is rendered.
    fn render_overlay(
        &mut self,
        // TODO Change to move once other arguments are removed?
        _ctx: &GridContext<dynamic::SizedGrid>,
    ) {}
}

//////////////////////////////////////////////////////////////////////////////////////////////////

lazy_static::lazy_static! {
    pub static ref DARK_BACKGROUND: Color32 = Color32::from_rgb(64, 64, 64);
    pub static ref LIGHT_BACKGROUND: Color32 = Color32::from_rgb(232, 232, 232);

    pub static ref AXIS_ONE_COLOR : Color32 = Color32::from_rgb(0, 0, 255);
    pub static ref AXIS_TWO_COLOR : Color32 = Color32::from_rgb(0, 255, 0);
    pub static ref AXIS_THREE_COLOR : Color32 = Color32::from_rgb(255, 0, 0);

    pub static ref AXES_COLORS: [Color32; 3] = [
        *AXIS_ONE_COLOR,
        *AXIS_TWO_COLOR,
        *AXIS_THREE_COLOR,
    ];

    pub static ref SOURCE_CELL_SPEC: CellStyle = CellStyle {
        fill_color: Some(Color32::from_rgba_unmultiplied(252, 182, 5, 64)),
        border: CellBorderStyle::none(),
        label: None,
    };

    pub static ref TARGET_CELL_SPEC: CellStyle = CellStyle {
        fill_color: Some(Color32::from_rgba_unmultiplied(128, 0, 255, 64)),
        border: CellBorderStyle::none(),
        label: None,
    };

    pub static ref HOLLOW_ARROW_STYLE: HollowArrowStyle = HollowArrowStyle {
        fill_color: Color32::from_rgba_unmultiplied(200, 200, 0, 196),
        border_color: Color32::from_rgba_unmultiplied(232, 232, 0, 255),
        width: 12.0,
        label: None,
    };
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn direction_type_widget(ui: &mut Ui, dir_type: &mut DirectionType) {
    egui::Grid::new("direction_type")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Direction type:");
            ui.horizontal(|ui| {
                ui.radio_value(dir_type, DirectionType::Face, "Face");
                ui.radio_value(dir_type, DirectionType::Vertex, "Vertex");
            });
        });
}

pub fn direction_widget(ui: &mut Ui, direction: &mut u8) {
    ui.add(
        egui::Slider::new(direction, 0..=7)
            .integer()
            .text("Direction")
            .custom_formatter(|n, _| Direction::from_u8(n as u8).short_name().to_string())
            .custom_parser(|s| Direction::parse(s).map(|d| (d as u8) as f64)),
    );
}

pub fn axis_widget(ui: &mut Ui, axis: &mut Option<dynamic::Axes>, grid_kind: dynamic::Kind) {
    if axis.is_none() || axis.unwrap().kind() != grid_kind {
        // Set a default axis if none is set, or if there is a mismatch.
        *axis = match grid_kind {
            dynamic::Kind::Square => Some(square::Axes::X.into()),
            dynamic::Kind::Hex => Some(hex::Axes::Q.into()),
            dynamic::Kind::Triangle => Some(triangle::Axes::A.into()),
        };
    }

    for axs in grid_kind.axes() {
        ui.radio_value(axis, Some(axs), format!("{axs} Axis"));
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Helper for generating wrapped text.
// TODO Is this really the simplest way to do this in egui?

pub fn wrapped_string(ui: &mut Ui, string: String) {
    let mut job = LayoutJob::single_section(
        string,
        egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
    );
    job.wrap = egui::text::TextWrapping::default();
    ui.label(job);
}

pub fn wrapped_str(ui: &mut Ui, str: &str) {
    wrapped_string(ui, str.to_owned());
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn unary_coordinate_label(ui: &mut Ui, coord: &Option<dynamic::Coord>) {
    let selection_text = if let Some(coord) = coord {
        format!("Selected coordinate: {coord:#}\n")
    } else {
        "No coordinate selected currently\n".to_owned()
    };
    wrapped_string(ui, selection_text);
}

pub fn binary_coordinates_labels(ui: &mut Ui, label1: &str, coord1: &Option<dynamic::Coord>, label2: &str, coord2: &Option<dynamic::Coord>) {
    let source_text = if let Some(coord) = coord1 {
        format!("Selected {label1} coordinate: {coord:#}\n")
    } else {
        "No source coordinate selected currently\n".to_owned()
    };
    wrapped_string(ui, source_text);
    let target_text = if let Some(coord) = coord2 {
        format!("Selected {label2} coordinate: {coord:#}\n")
    } else {
        "No target coordinate selected currently\n".to_owned()
    };
    wrapped_string(ui, target_text);
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn optional_coordinate_select(ctx: &GridContext<dynamic::SizedGrid>) -> Option<dynamic::Coord> {
    let grc = &ctx.grc;
    let prc = ctx
        .ui
        .interact(ctx.response.rect, ctx.response.id, Sense::click());
    if prc.clicked() {
        let pos = prc.interact_pointer_pos().unwrap();
        let pos2 = grc.transform.inverse().transform_pos(pos);
        return Some(egui_pos2_to_coord(pos2, &grc.szg));
    };
    None
}

/// Helper to reset the selected coordinate if the grid kind has changed.
pub fn reset_coord(ctx: &GridContext<dynamic::SizedGrid>, opt_coord: &mut Option<dynamic::Coord>) {
    // If the coordinate kind has changed, reset the coordinates.
    if let Some(coord) = opt_coord
        && coord.kind() != ctx.grc.szg.kind()
    {
        *opt_coord = None;
    }
}

// TODO Generalize from unary and binary cases?

pub fn unary_coordinate_select(
    ctx: &GridContext<dynamic::SizedGrid>,
    coord: &mut Option<dynamic::Coord>,
) {
    // If the coordinate kind has changed, reset the coordinate.
    reset_coord(ctx, coord);
    // If there was a click, unwrap it.  Otherwise, return.
    let Some(click_coord) = optional_coordinate_select(ctx) else {
        return;
    };
    let _ = coord.insert(click_coord);
}

pub fn binary_coordinates_select(
    ctx: &GridContext<dynamic::SizedGrid>,
    coord1: &mut Option<dynamic::Coord>,
    coord2: &mut Option<dynamic::Coord>,
) {
    // Verify that the second coordinate somehow has not been set without the first.
    assert!(coord2.is_none() || coord1.is_some());

    // If the coordinate kind has changed, reset the coordinates.
    reset_coord(ctx, coord1);
    reset_coord(ctx, coord2);

    // If there was a click, unwrap it.  Otherwise, return.
    let Some(click_coord) = optional_coordinate_select(ctx) else {
        return;
    };

    // If both coordinates are already set, also reset them, so we start by setting
    // the first coordinate.
    if coord1.is_some() && coord2.is_some() {
        *coord1 = None;
        *coord2 = None;
    }

    // If the first coordinate is not set, we will set it.
    // Otherwise, we will set the second coordinate.
    let coord = if coord1.is_none() { coord1 } else { coord2 };
    let _ = coord.insert(click_coord);
}
