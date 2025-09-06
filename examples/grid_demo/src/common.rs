use eframe::epaint::Color32;
use egui::Ui;
use endgame_direction::Direction;
use endgame_egui::{CellBorderStyle, CellStyle, HollowArrowStyle};
use endgame_grid::{dynamic, hex, square, triangle, DirectionType};
use std::cell::RefMut;
use std::collections::VecDeque;

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

//////////////////////////////////////////////////////////////////////////////

// TODO Generalize from unary and binary cases?

pub fn unary_coordinate_select(
    dszg: &dynamic::SizedGrid,
    click: egui::Pos2,
    coord: &mut Option<dynamic::Coord>,
) {
    *coord = Some(endgame_egui::egui_pos2_to_coord(click, dszg));
}

pub fn binary_coordinate_select(
    dszg: &dynamic::SizedGrid,
    click: egui::Pos2,
    coord1: &mut Option<dynamic::Coord>,
    coord2: &mut Option<dynamic::Coord>,
) {
    // If both coordinates are already set, also reset them.
    if coord1.is_some() && coord2.is_some() {
        *coord1 = None;
        *coord2 = None;
    }

    // If the fist coordinate is not set, set it.
    if coord1.is_none() {
        *coord1 = Some(endgame_egui::egui_pos2_to_coord(click, dszg));
        return;
    }

    // If there are still clicks available, set the second coordinate.
    if coord2.is_none() {
        *coord2 = Some(endgame_egui::egui_pos2_to_coord(click, dszg));
    }
}

pub fn unary_coordinates_select(
    dszg: &dynamic::SizedGrid,
    kind: dynamic::Kind,
    clicks: &mut RefMut<Box<VecDeque<egui::Pos2>>>,
    coord: &mut Option<dynamic::Coord>,
) {
    // If the coordinate kind has changed, reset the coordinates.
    if let Some(c) = coord
        && c.kind() != kind
    {
        *coord = None;
    }

    while !clicks.is_empty() {
        let click = clicks.pop_front().expect("clicks is non-empty");
        unary_coordinate_select(dszg, click, coord);
    }
}

pub fn binary_coordinates_select(
    dszg: &dynamic::SizedGrid,
    kind: dynamic::Kind,
    clicks: &mut RefMut<Box<VecDeque<egui::Pos2>>>,
    coord1: &mut Option<dynamic::Coord>,
    coord2: &mut Option<dynamic::Coord>,
) {
    // If the coordinate kind has changed, reset the coordinates.
    if let Some(coord) = coord1
        && coord.kind() != kind
    {
        *coord1 = None;
        *coord2 = None;
    }

    while !clicks.is_empty() {
        let click = clicks.pop_front().expect("clicks is non-empty");
        binary_coordinate_select(dszg, click, coord1, coord2);
    }
}
