use crate::common;
use crate::common::ExampleUi;
use crate::common::GridExample;

use egui::scroll_area::ScrollBarVisibility;
use egui::{Color32, ScrollArea};
use endgame_egui::{CellBorderStyle, CellStyle, GridContext, Theme};
use endgame_grid::shape::HashShape;
use endgame_grid::{dynamic, Shape};
use std::collections::BTreeMap;

#[derive(PartialEq, Eq, Default)]
enum ShapeChoice {
    #[default]
    Ring,
    Range,
}

struct ShapeInstance {
    size: usize,
    choice: ShapeChoice,
    subtractive: bool,
}

pub struct Ui {
    shapes: BTreeMap<usize, ShapeInstance>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            shapes: BTreeMap::from([
                (
                    0,
                    ShapeInstance {
                        size: 1,
                        choice: ShapeChoice::Range,
                        subtractive: false,
                    },
                ),
                (
                    1,
                    ShapeInstance {
                        size: 0,
                        choice: ShapeChoice::Ring,
                        subtractive: true,
                    },
                ),
                (
                    2,
                    ShapeInstance {
                        size: 3,
                        choice: ShapeChoice::Ring,
                        subtractive: false,
                    },
                ),
            ]),
        }
    }
}

impl ExampleUi for Ui {
    fn example(&self) -> GridExample {
        GridExample::Shapes
    }

    fn label(&self) -> &'static str {
        "Shapes"
    }

    fn cell_theme(&self) -> Theme {
        Theme::GraphPaper
    }

    fn controls(&mut self, _grid_kind: dynamic::Kind, ui: &mut egui::Ui) {
        common::wrapped_str(
            ui,
            "Experiment with constructing grid shapes.  The currently active shapes will be \
             combined or subtracted in order to construct a new shape.\n",
        );

        if ui.button("Add Shape").clicked() {
            let next_num = self.shapes.keys().max().map_or(0, |n| n + 1);
            self.shapes.insert(
                next_num,
                ShapeInstance {
                    size: 3,
                    choice: ShapeChoice::Ring,
                    subtractive: false,
                },
            );
        }

        let mut removals = Vec::new();
        ScrollArea::vertical()
            .auto_shrink(false)
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                for (num, instance) in self.shapes.iter_mut() {
                    ui.separator();

                    egui::Grid::new(format!("shape_options_{num}"))
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut instance.choice, ShapeChoice::Range, "Range");
                                ui.radio_value(&mut instance.choice, ShapeChoice::Ring, "Ring");
                            });
                            ui.checkbox(&mut instance.subtractive, "Subtractive");
                            ui.end_row();

                            ui.add(egui::Slider::new(&mut instance.size, 0..=16).text("Size"));

                            ui.button("Remove")
                                .on_hover_text("Remove Shape")
                                .clicked()
                                .then(|| {
                                    removals.push(*num);
                                });

                            ui.end_row();
                        });
                }
            });

        for num in removals {
            self.shapes.remove(&num);
        }
    }

    fn render_overlay(&mut self, ctx: &GridContext<dynamic::SizedGrid>) {
        let grc = &ctx.grc;
        let base_style = CellStyle {
            border: CellBorderStyle::uniform(
                4.0,
                Color32::from_rgba_unmultiplied(252, 182, 5, 192),
            ),
            ..common::SOURCE_CELL_SPEC.clone()
        };

        let mut opt_shape = None::<HashShape<dynamic::Coord>>;

        let grid_kind = grc.szg.kind();
        for instance in self.shapes.values() {
            let shape = match instance.choice {
                ShapeChoice::Ring => dynamic::Coord::ring(grid_kind, instance.size),
                ShapeChoice::Range => dynamic::Coord::range(grid_kind, instance.size),
            };

            let _ = opt_shape.insert(match &opt_shape {
                Some(existing_shape) => {
                    if instance.subtractive {
                        existing_shape - shape
                    } else {
                        existing_shape.union(&shape)
                    }
                }
                None => shape,
            });
        }

        let Some(shape) = opt_shape else { return };
        grc.render_shape(&shape, &base_style, None);
    }
}
