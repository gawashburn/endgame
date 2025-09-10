use crate::{common, ExampleUi, GridDemo, GridExample};
use eframe::emath::RectTransform;
use eframe::epaint::text::LayoutJob;
use eframe::epaint::{Color32, FontId};
use egui::scroll_area::ScrollBarVisibility;
use egui::{Painter, ScrollArea};
use endgame_direction::Direction;
use endgame_egui::{CellBorderStyle, CellPrimitiveBorderStyle, CellStyle, Theme};
use endgame_grid::shape::HashShape;
use endgame_grid::{dynamic, Coord, DirectionType, Shape};
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

    fn cell_theme(&self, coord: &dynamic::Coord, dark_mode: bool) -> CellStyle {
        Theme::GraphPaper.cell_style(coord, dark_mode)
    }

    fn controls(&mut self, _demo: &GridDemo, ui: &mut egui::Ui) {
        let mut job = LayoutJob::single_section(
            "Experiment with constructing grid shapes.  The currently active \
            shapes will be combined or subtracted in order to construct a new \
            shape.\n"
                .to_owned(),
            egui::TextFormat::simple(FontId::default(), ui.visuals().text_color()),
        );
        job.wrap = egui::text::TextWrapping::default();

        ui.label(job);

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

    fn render_overlay(
        &mut self,
        demo: &GridDemo,
        dszg: &dynamic::SizedGrid,
        transform: &RectTransform,
        painter: &Painter,
    ) {
        let base_style = common::SOURCE_CELL_SPEC.clone();

        let mut opt_shape = None::<HashShape<dynamic::Coord>>;

        for instance in self.shapes.values() {
            let shape = match instance.choice {
                ShapeChoice::Ring => dynamic::Coord::ring(demo.grid_kind, instance.size),
                ShapeChoice::Range => dynamic::Coord::range(demo.grid_kind, instance.size),
            };

            match opt_shape {
                Some(existing_shape) => {
                    if instance.subtractive {
                        opt_shape = Some(existing_shape - shape);
                    } else {
                        opt_shape = Some(existing_shape.union(&shape));
                    }
                }
                None => {
                    opt_shape = Some(shape);
                }
            }
        }

        let Some(shape) = opt_shape else { return };

        for coord in shape.iter() {
            let allowed_directions = coord.allowed_directions(DirectionType::Face);
            let no_adjacent: Vec<Direction> = allowed_directions
                .into_iter()
                .filter(|d| {
                    let dir_coord = coord
                        .move_in_direction(DirectionType::Face, *d)
                        .expect("Direction should be valid");
                    !shape.contains(&dir_coord)
                })
                .collect();

            let style = CellStyle {
                border: CellBorderStyle::PerEdge(
                    no_adjacent
                        .into_iter()
                        .map(|d| {
                            (
                                d,
                                CellPrimitiveBorderStyle::Uniform(
                                    4.0,
                                    Color32::from_rgba_unmultiplied(252, 182, 5, 192),
                                ),
                            )
                        })
                        .collect(),
                ),
                ..base_style.clone()
            };

            endgame_egui::render_coord_cell(dszg, coord, &style, None::<&str>, transform, painter);
        }
    }
}
