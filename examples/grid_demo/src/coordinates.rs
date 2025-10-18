use crate::grid_demo::{ExampleUi, GridExample};

#[derive(Default)]
pub struct Ui {}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::Coordinates
    }
    fn label(&self) -> &'static str {
        "Coordinates"
    }
}
