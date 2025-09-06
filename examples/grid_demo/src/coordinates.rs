use crate::ExampleUi;

#[derive(Default)]
pub struct Ui {}

impl ExampleUi for Ui {
    fn example(&self) -> crate::GridExample {
        crate::GridExample::Coordinates
    }
    fn label(&self) -> &'static str {
        "Coordinates"
    }
}
