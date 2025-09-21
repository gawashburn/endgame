extern crate core;

use egui::ahash::HashSet;
use egui::emath::{RectTransform, TSTransform};
use egui::epaint::ColorMode::Solid;
use egui::epaint::{PathShape, PathStroke};
use egui::{pos2, Color32, Painter, Pos2, Rect, Sense};
use endgame_direction::{Direction, DirectionSet};
use endgame_grid::Color::{Four, One, Three, Two};
use endgame_grid::{Coord, DirectionType, Shape, ShapeContainer, SizedGrid};
use itertools::Itertools;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
//////////////////////////////////////////////////////////////////////////////

// Conversion helpers as we cannot define From or Into for these types.

/// Given a screen coordinate as  `egui::Pos2`, convert it to a grid coord
/// using the provided `dynamic::SizedGrid`.
pub fn egui_pos2_to_coord<SZ: SizedGrid>(pos: Pos2, szg: &SZ) -> SZ::Coord {
    let mint: mint::Point2<f32> = pos.into();
    szg.screen_to_grid(mint.into())
}

/// Given a grid coordinate as `dynamic::Coord`, convert it to a screen
/// position as `egui::Pos2` using the provided `dynamic::SizedGrid`.
pub fn coord_to_egui_pos2<SZ: SizedGrid>(coord: &SZ::Coord, szg: &SZ) -> Pos2 {
    let mint: mint::Point2<f32> = szg.grid_to_screen(coord).into();
    mint.into()
}

/// Helper to convert a `glam::Vec2` into a `egui::Pos2`.
pub fn glam_vec2_to_egui_pos2(v: glam::Vec2) -> Pos2 {
    let mint: mint::Point2<f32> = v.into();
    mint.into()
}

/// Helper to convert a `egui::Pos2` into a `glam::Vec2`.
pub fn egui_pos2_to_glam_vec2(p: Pos2) -> glam::Vec2 {
    let mint: mint::Point2<f32> = p.into();
    mint.into()
}

//////////////////////////////////////////////////////////////////////////////

/// Helper to adjust the length of a line segment by moving its endpoints,
/// maintaining its orientation but altering where it starts and ends.
/// The segment must not be zero length, and it cannot be shortened to a
/// length of zero or less.
pub fn alter_segment_length(
    from: glam::Vec2,
    to: glam::Vec2,
    start_offset: f32,
    end_offset: f32,
) -> (glam::Vec2, glam::Vec2) {
    let segment_vec = to - from;
    let segment_len = segment_vec.length();
    assert!(
        segment_len > 0.0,
        "Cannot alter length of zero-length segment ({from} to {to})"
    );

    let segment_norm = segment_vec.normalize();
    // Compute the new segment endpoints.
    let new_from = from + segment_norm * start_offset;
    let new_to = to + segment_norm * end_offset;
    let new_vec = new_to - new_from;
    assert!(
        new_vec.length() > 0.0, //&& segment_vec.to_angle() == new_vec.to_angle(),
        "Cannot alter length of segment to be zero or less. Or change orientation."
    );

    (new_from, new_to)
}

//////////////////////////////////////////////////////////////////////////////

/// `LabelStyle` provides styling information for rendering text labels.
#[derive(Debug, Clone)]
pub struct LabelStyle {
    pub color: Color32,
    pub font_size: f32,
    pub add_shadow: Option<Color32>,
}

/// `SolidArrowStyle` provides styling information for rendering arrows.
/// If no heads are specified it is the degenerate case of a line segment
/// with the possibility of a label.a
#[derive(Clone)]
pub struct SolidArrowStyle {
    pub color: Color32,
    pub width: f32,
    // TODO Add arrow head style options.
    pub to_head: bool,
    pub from_head: bool,
    // TODO Add an option to specify the location of the label relative to
    //   the arrow.
    pub label: Option<LabelStyle>,
}

/// `HollowArrowStyle` provides styling information for rendering arrows.
/// Currently, hollow arrows must have a head.
#[derive(Clone)]
pub struct HollowArrowStyle {
    pub fill_color: Color32,
    pub border_color: Color32,
    pub width: f32,
    // TODO Add an option to specify the location of the label relative to
    //   the arrow.
    pub label: Option<LabelStyle>,
}

/// `CellPrimitiveBorderStyle` provides styling information for rendering
/// the border of a grid cell.
#[derive(Debug, Clone)]
pub enum CellPrimitiveBorderStyle {
    /// Draw no border.
    None,
    /// Draw a border of uniform thickness and color.
    /// TODO allow specifying inside and outside borders.
    Uniform(f32, Color32),
}

impl CellPrimitiveBorderStyle {
    /// Get the color of the border style.
    pub fn color(&self) -> Color32 {
        match self {
            CellPrimitiveBorderStyle::None => Color32::TRANSPARENT,
            CellPrimitiveBorderStyle::Uniform(_, c) => *c,
        }
    }

    /// Get the width of the border style.
    pub fn width(&self) -> f32 {
        match self {
            CellPrimitiveBorderStyle::None => 0.0,
            CellPrimitiveBorderStyle::Uniform(w, _) => *w,
        }
    }
}

/// `CellBorderStyle` provides styling information for rendering the border
/// of a grid cell.  It can either be a primitive style applied to the entire
/// border, or a different style for each edge of the cell.
#[derive(Debug, Clone)]
pub enum CellBorderStyle {
    /// Just use a primitive style.
    Primitive(CellPrimitiveBorderStyle),
    /// Use a different style for each edge.
    /// This map must be a subset of the directions that
    /// correspond to faces of the cell.
    PerEdge(HashMap<Direction, CellPrimitiveBorderStyle>),
}

impl CellBorderStyle {
    /// Helper to create a `CellBorderStyle` with no border.
    pub fn none() -> Self {
        CellBorderStyle::Primitive(CellPrimitiveBorderStyle::None)
    }

    /// Helper to create a `CellBorderStyle` with a uniform border.
    pub fn uniform(width: f32, color: Color32) -> Self {
        CellBorderStyle::Primitive(CellPrimitiveBorderStyle::Uniform(width, color))
    }
}

/// `CellStyle` provides styling information for rendering a grid cell.
#[derive(Debug, Clone)]
pub struct CellStyle {
    pub fill_color: Option<Color32>,
    pub border: CellBorderStyle,
    // TODO Add options for label placement.
    pub label: Option<LabelStyle>,
}

//////////////////////////////////////////////////////////////////////////////

// Color theming.

/// A `Theme` provides some predefined styling for grid cells.
#[derive(Debug, Clone, Copy)]
pub enum Theme {
    /// A theme reminiscent of a map where no adjacent cells have the same
    /// color.
    Map,
    /// A theme reminiscent of classic graph paper with a light background and
    /// blue grid lines.
    GraphPaper,
}

impl Theme {
    /// For the given theme, coordinate, and dark mode setting, produce a
    /// `CellStyle`.
    pub fn cell_style<C: Coord>(self, coord: &C, dark_mode: bool) -> CellStyle {
        let coord_color = coord.to_color();
        match self {
            Theme::Map => {
                let fill_color = match coord_color {
                    One => Color32::from_rgb(64, 128, 64),
                    Two => Color32::from_rgb(232, 232, 216),
                    Three => Color32::from_rgb(128, 64, 64),
                    Four => Color32::from_rgb(64, 64, 128),
                };
                let (text_color, shadow_color) = match coord_color {
                    One => (Color32::WHITE, Color32::BLACK),
                    Two => (Color32::BLACK, Color32::GRAY),
                    Three => (Color32::WHITE, Color32::BLACK),
                    Four => (Color32::WHITE, Color32::BLACK),
                };

                let border = if coord.is_origin() {
                    let (r, g, b, a) = fill_color.to_tuple();
                    CellBorderStyle::uniform(
                        4.0,
                        if dark_mode {
                            Color32::from_rgba_premultiplied(
                                r.saturating_add(64),
                                g.saturating_add(64),
                                b.saturating_add(64),
                                a,
                            )
                        } else {
                            Color32::from_rgba_premultiplied(
                                r.saturating_sub(64),
                                g.saturating_sub(64),
                                b.saturating_sub(64),
                                a,
                            )
                        },
                    )
                } else {
                    CellBorderStyle::none()
                };

                CellStyle {
                    fill_color: Some(fill_color),
                    border: border,
                    label: Some(LabelStyle {
                        color: text_color,
                        font_size: 8.0,
                        add_shadow: Some(shadow_color),
                    }),
                }
            }

            Theme::GraphPaper => {
                let color = Color32::from_rgb(98, 213, 250);
                let border = if coord.is_origin() {
                    CellBorderStyle::uniform(4.0, color)
                } else {
                    CellBorderStyle::uniform(2.0, color)
                };
                CellStyle {
                    fill_color: Some(Color32::from_rgb(255, 255, 250)),
                    border: border,
                    label: Some(LabelStyle {
                        color: color,
                        font_size: 8.0,
                        add_shadow: Some(Color32::GRAY),
                    }),
                }
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Helper for drawing a styled label.
pub fn render_label(pos: Pos2, style: LabelStyle, label: &str, painter: &Painter) {
    if let Some(shadow_color) = style.add_shadow {
        painter.text(
            pos + egui::Vec2::new(1.0, 1.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::monospace(style.font_size),
            shadow_color,
        );
    }
    painter.text(
        pos,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(style.font_size),
        style.color,
    );
}

//////////////////////////////////////////////////////////////////////////////

/// Draw a 🚫.
pub fn render_disallowed(
    center: Pos2,
    radius: f32,
    width: f32,
    transform: &RectTransform,
    painter: &Painter,
) {
    let slash_start = center + (egui::Vec2::angled(PI / 4.0) * radius);
    let slash_end = center - (egui::Vec2::angled(PI / 5.0) * radius);

    painter.line(
        vec![
            transform.transform_pos(slash_start),
            transform.transform_pos(slash_end),
        ],
        egui::Stroke {
            width: width,
            color: Color32::RED,
        },
    );
    painter.circle_stroke(
        transform.transform_pos(center),
        radius,
        egui::Stroke {
            width: width,
            color: Color32::RED,
        },
    );
}

/// Helper to produce a solid arrow head shape for drawing solid arrows.
fn solid_arrow_head_shape(tip: Pos2, angle: f32, color: Color32) -> egui::Shape {
    let r_vec = (egui::Vec2::angled(angle + std::f32::consts::FRAC_PI_6) * 6.0) + tip.to_vec2();
    let l_vec = (egui::Vec2::angled(angle - std::f32::consts::FRAC_PI_6) * 6.0) + tip.to_vec2();

    PathShape {
        points: vec![tip, r_vec.to_pos2(), l_vec.to_pos2()],
        closed: true,
        fill: color,
        stroke: PathStroke {
            width: 1.0,
            color: Solid(color),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into()
}

pub fn render_arrow(
    from: Pos2,
    to: Pos2,
    style: &SolidArrowStyle,
    opt_label: Option<&str>,
    painter: &Painter,
) {
    let line_back_vec = from.to_vec2() - to.to_vec2();
    let angle = line_back_vec.angle();
    if style.to_head {
        painter.add(solid_arrow_head_shape(to, angle, style.color));
    }
    if style.from_head {
        painter.add(solid_arrow_head_shape(from, angle + PI, style.color));
    }
    painter.line(
        vec![from, to],
        PathStroke {
            width: style.width, // 2.0
            color: Solid(style.color),
            kind: egui::StrokeKind::Middle,
        },
    );

    if let Some((label_style, label)) = style.label.as_ref().zip(opt_label) {
        let center = (from.to_vec2() + to.to_vec2()) / 2.0;
        let offset =
            egui::Vec2::angled(angle + std::f32::consts::FRAC_PI_2) * label_style.font_size * 2.0;
        render_label(
            (center + offset).to_pos2(),
            label_style.clone(),
            label,
            painter,
        );
    }
}

pub fn render_arrow_arc(
    center: Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    style: &SolidArrowStyle,
    label: Option<&str>,
    painter: &Painter,
) {
    let steps = ((end_angle - start_angle).abs() / 0.01).ceil() as usize;

    let start_vec = egui::Vec2::angled(start_angle) * radius;
    let end_vec = egui::Vec2::angled(end_angle) * radius;
    painter.add(solid_arrow_head_shape(
        start_vec.to_pos2() + center.to_vec2(),
        start_angle + std::f32::consts::FRAC_PI_2,
        style.color,
    ));
    painter.add(solid_arrow_head_shape(
        end_vec.to_pos2() + center.to_vec2(),
        end_angle - std::f32::consts::FRAC_PI_2,
        style.color,
    ));

    let mut from_pos = center + egui::Vec2::angled(start_angle) * radius;
    let step = (end_angle - start_angle) / steps as f32;
    for index in 1..=steps {
        let angle = start_angle + step * index as f32;
        let to_pos = center + egui::Vec2::angled(angle) * radius;
        painter.line(
            vec![from_pos, to_pos],
            PathStroke {
                width: style.width, // 2.0
                color: Solid(style.color),
                kind: egui::StrokeKind::Middle,
            },
        );
        from_pos = to_pos;
    }

    if let Some((label_style, label)) = style.label.as_ref().zip(label) {
        let mid_vec = egui::Vec2::angled((end_angle + start_angle) / 2.0)
            * (radius + label_style.font_size * 3.0);
        render_label(center + mid_vec, label_style.clone(), label, painter);
    }
}

//////////////////////////////////////////////////////////////////////////////

pub fn render_hollow_arrow(
    from: Pos2,
    to: Pos2,
    style: &HollowArrowStyle,
    opt_label: Option<&str>,
    painter: &Painter,
) {
    let from_vec = egui_pos2_to_glam_vec2(from);
    let to_vec = egui_pos2_to_glam_vec2(to);
    let base_vec = to_vec - from_vec;
    let gvec = base_vec.normalize() * (base_vec.length() - style.width * 2.0);

    let perp = gvec.perp().normalize() * style.width * 0.5;
    let lstart = from_vec - perp;
    let rstart = from_vec + perp;
    let lend = lstart + gvec;
    let rend = rstart + gvec;
    let lend_head = lend - 1.0 * perp;
    let rend_head = rend + 1.0 * perp;

    let divot = (gvec.normalize() * style.width * 0.25) + from_vec;

    // TODO There seems to be a bug in egui's concave PathShape rendering,
    //   So we need to break this up into the arrow shaft and head, plus the
    //   border.
    let arrow_head: egui::Shape = PathShape {
        points: vec![
            glam_vec2_to_egui_pos2(lend_head),
            glam_vec2_to_egui_pos2(to_vec),
            glam_vec2_to_egui_pos2(rend_head),
        ],
        closed: true,
        fill: style.fill_color,
        stroke: PathStroke {
            width: 0.0,
            color: Solid(Color32::TRANSPARENT),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into();

    let arrow_shaft: egui::Shape = PathShape {
        points: vec![
            glam_vec2_to_egui_pos2(divot),
            glam_vec2_to_egui_pos2(lstart),
            glam_vec2_to_egui_pos2(lend),
            glam_vec2_to_egui_pos2(rend),
            glam_vec2_to_egui_pos2(rstart),
        ],
        closed: true,
        fill: style.fill_color,
        stroke: PathStroke {
            width: 0.0,
            color: Solid(Color32::TRANSPARENT),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into();

    let arrow_border: egui::Shape = PathShape {
        points: vec![
            glam_vec2_to_egui_pos2(lstart),
            glam_vec2_to_egui_pos2(lend),
            glam_vec2_to_egui_pos2(lend_head),
            glam_vec2_to_egui_pos2(to_vec),
            glam_vec2_to_egui_pos2(rend_head),
            glam_vec2_to_egui_pos2(rend),
            glam_vec2_to_egui_pos2(rstart),
            glam_vec2_to_egui_pos2(divot),
        ],
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: PathStroke {
            width: 1.0,
            color: Solid(style.border_color),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into();

    painter.add(arrow_shaft);
    painter.add(arrow_head);
    painter.add(arrow_border);

    if let Some((label_style, label)) = style.label.as_ref().zip(opt_label) {
        let center = (from.to_vec2() + to.to_vec2()) / 2.0;
        render_label(center.to_pos2(), label_style.clone(), label, painter);
    }
}

pub fn render_hollow_self_arrow(
    pos: Pos2,
    style: &HollowArrowStyle,
    label: Option<&str>,
    painter: &Painter,
) {
    let start_angle = 3.0 * PI / 2.0;
    let end_angle = -PI / 4.0;
    let radius = style.width * 1.5;
    let center = pos + egui::Vec2::new(0.0, radius);

    let steps = ((end_angle - start_angle).abs() / 0.2).ceil() as usize;

    let mut larc_points = Vec::new();
    let mut rarc_points = Vec::new();
    let mut from_pos = center + egui::Vec2::angled(start_angle) * radius;
    let step = (end_angle - start_angle) / steps as f32;
    for index in 1..=steps {
        let angle = start_angle + step * index as f32;
        let to_pos = center + egui::Vec2::angled(angle) * radius;
        let vec = to_pos - from_pos;
        let perp = egui_pos2_to_glam_vec2(vec.to_pos2()).perp().normalize() * style.width * 0.5;
        let pperp = glam_vec2_to_egui_pos2(perp).to_vec2();

        let left = from_pos - pperp;
        let right = from_pos + pperp;

        larc_points.push(left);
        rarc_points.push(right);
        from_pos = to_pos;
    }

    let chunks = larc_points.iter().zip(rarc_points.iter());
    for ((l1, r1), (l2, r2)) in chunks.tuple_windows() {
        let quad: egui::Shape = PathShape {
            points: vec![*l1, *l2, *r2, *r1],
            closed: true,
            fill: style.fill_color,
            stroke: PathStroke {
                width: 0.0,
                color: Solid(Color32::TRANSPARENT),
                kind: egui::StrokeKind::Middle,
            },
        }
            .into();
        painter.add(quad);
    }
    let lend = larc_points.last().unwrap();
    let rend = rarc_points.last().unwrap();
    let (lhead, rhead) = alter_segment_length(
        egui_pos2_to_glam_vec2(*lend),
        egui_pos2_to_glam_vec2(*rend),
        -style.width * 0.5,
        style.width * 0.5,
    );
    let end_vec = rhead - lhead;
    let tip = (end_vec * 0.5) - (end_vec.perp().normalize() * style.width) + lhead;

    let arrow_head: egui::Shape = PathShape {
        points: vec![
            glam_vec2_to_egui_pos2(lhead),
            glam_vec2_to_egui_pos2(tip),
            glam_vec2_to_egui_pos2(rhead),
        ],
        closed: true,
        fill: style.fill_color,
        stroke: PathStroke {
            width: 0.0,
            color: Solid(Color32::TRANSPARENT),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into();

    let mut border = larc_points;
    border.push(glam_vec2_to_egui_pos2(lhead));
    border.push(glam_vec2_to_egui_pos2(tip));
    border.push(glam_vec2_to_egui_pos2(rhead));
    border.append(rarc_points.into_iter().rev().collect::<Vec<_>>().as_mut());

    let arrow_border: egui::Shape = PathShape {
        points: border,
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: PathStroke {
            width: 1.0,
            color: Solid(style.border_color),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into();

    painter.add(arrow_head);
    painter.add(arrow_border);

    if let Some((label_style, label)) = style.label.as_ref().zip(label) {
        let mid_vec = egui::Vec2::angled((end_angle + start_angle) / 2.0)
            * (radius + label_style.font_size * 3.0);
        render_label(center + mid_vec, label_style.clone(), label, painter);
    }
}

//////////////////////////////////////////////////////////////////////////////

pub fn render_hollow_arrow_coords<SZ: SizedGrid>(
    szg: &SZ,
    from: &SZ::Coord,
    to: &SZ::Coord,
    style: &HollowArrowStyle,
    opt_label: Option<&str>,
    transform: &RectTransform,
    painter: &Painter,
) {
    let from_pos = szg.grid_to_screen(from);

    let width = 12.0 * (szg.inradius() / 64.0);
    let style = HollowArrowStyle {
        width: width.min(12.0),
        ..style.clone()
    };

    if from == to {
        render_hollow_self_arrow(
            transform.transform_pos(glam_vec2_to_egui_pos2(from_pos)),
            &style,
            opt_label,
            painter,
        );
        return;
    }

    let to_pos = szg.grid_to_screen(to);
    let d = szg.inradius() * 0.33;
    let (from_adjusted, to_adjusted) = alter_segment_length(from_pos, to_pos, d, -d);

    render_hollow_arrow(
        transform.transform_pos(glam_vec2_to_egui_pos2(from_adjusted)),
        transform.transform_pos(glam_vec2_to_egui_pos2(to_adjusted)),
        &style,
        opt_label,
        painter,
    );
}

//////////////////////////////////////////////////////////////////////////////

pub fn render_coord_cell<SZ: SizedGrid, T: AsRef<str>>(
    szg: &SZ,
    coord: &SZ::Coord,
    style: &CellStyle,
    opt_label: Option<T>,
    transform: &RectTransform,
    painter: &Painter,
) {
    let screen = szg.grid_to_screen(coord);
    let pos = pos2(screen.x, screen.y);

    let verts = szg.vertices(coord);
    let points = verts.iter().map(|v| pos2(v.x, v.y)).collect::<Vec<_>>();

    let prim_style = match &style.border {
        // We are either drawing no broder, or drawing it separately as
        // `egui` does not presently support adjusting the stroke for
        // different segments of a `PathShape`.
        CellBorderStyle::Primitive(ps) => ps,
        CellBorderStyle::PerEdge(_) => &CellPrimitiveBorderStyle::None,
    };

    let mut render_cell: egui::Shape = PathShape {
        points: points.clone(),
        closed: true,
        fill: style.fill_color.unwrap_or(Color32::TRANSPARENT),
        stroke: PathStroke {
            width: prim_style.width(),
            color: Solid(prim_style.color()),
            kind: egui::StrokeKind::Middle,
        },
    }
        .into();

    render_cell.transform(TSTransform {
        scaling: 1.0,
        // TODO This seems a bit awkward.
        translation: transform.transform_pos(Pos2::ZERO).to_vec2(),
    });
    painter.add(render_cell);

    // If we are doing per-edge styling, draw it now.
    if let CellBorderStyle::PerEdge(ref edge_styles) = style.border {
        let edges = szg.edges(coord);
        assert!(
            edge_styles
                .keys()
                .collect::<HashSet<_>>()
                .is_subset(&edges.keys().collect::<HashSet<_>>()),
            "The edge styles must be a subset of the grid cell edges."
        );

        // TODO Seems like there should be a way to zip values by keys?
        for (dir, edge) in edges.iter() {
            let style = edge_styles
                .get(dir)
                .unwrap_or(&CellPrimitiveBorderStyle::None);
            painter.line(
                vec![
                    transform.transform_pos(glam_vec2_to_egui_pos2(edge.0)),
                    transform.transform_pos(glam_vec2_to_egui_pos2(edge.1)),
                ],
                egui::Stroke {
                    width: style.width(),
                    color: style.color(),
                },
            );
        }
    }

    if let Some((label_style, label)) = style.label.as_ref().zip(opt_label) {
        let center_vec = transform.transform_pos(pos).to_vec2();
        let center = ((transform.transform_pos(*points.last().unwrap()).to_vec2() - center_vec)
            / 2.0)
            + center_vec;
        let font_size = 12.0 * (szg.inradius() / 64.0);
        let style = LabelStyle {
            font_size: label_style.font_size.min(font_size),
            ..label_style.clone()
        };
        render_label(center.to_pos2(), style.clone(), label.as_ref(), painter);
    }
}

//////////////////////////////////////////////////////////////////////////////

pub fn render_shape<SZ: SizedGrid, S: Shape<SZ::Coord>>(
    dszg: &SZ,
    shape: &S,
    style: &CellStyle,
    inner_border_style: Option<CellPrimitiveBorderStyle>,
    transform: &RectTransform,
    painter: &Painter,
) {
    // Currently only support primitive border styles.
    let CellBorderStyle::Primitive(prim) = &style.border else {
        return;
    };

    for coord in shape.iter() {
        // TODO There is perhaps a more efficient means of finding the
        //   external edges of a shape.
        let allowed_directions: DirectionSet = coord.allowed_directions(DirectionType::Face);
        let render_coord = coord.clone();
        let no_adjacent: DirectionSet = allowed_directions
            //from_shape_value(square::Coord::range(3), None);
            .into_iter()
            .filter(|d| {
                let dir_coord = coord
                    .move_in_direction(DirectionType::Face, *d)
                    .expect("Direction should be valid");
                !shape.contains(&dir_coord)
            })
            .collect();

        // TODO implement add, sub, etc.
        //let interior = allowed_directions - no_adjacent;
        let interior = allowed_directions.difference(no_adjacent);
        let mut edge_style = Vec::new();
        edge_style.extend(
            no_adjacent
                .into_iter()
                .map(|d| (d, prim.clone())),
        );
        edge_style.extend(
            interior
                .into_iter()
                .map(|d| {
                    (
                        d,
                        inner_border_style
                            .clone()
                            .unwrap_or(CellPrimitiveBorderStyle::None),
                    )
                }),
        );

        let style = CellStyle {
            border: CellBorderStyle::PerEdge(edge_style.into_iter().collect()),
            ..style.clone()
        };

        render_coord_cell(dszg, &render_coord, &style, None::<&str>, transform, painter);
    }
}

//////////////////////////////////////////////////////////////////////////////

pub fn render_shape_container<SZ: SizedGrid, V, SC: ShapeContainer<SZ::Coord, V>>(
    dszg: &SZ,
    shape_container: &SC,
    style: &CellStyle,
    inner_border_style: Option<CellPrimitiveBorderStyle>,
    transform: &RectTransform,
    painter: &Painter,
    render_val: impl Fn(&SZ::Coord, &V, &RectTransform, &Painter) -> (),
)
where
    V: Debug + Clone + PartialEq + Eq + Hash,
{
    let shape = shape_container.as_shape();
    render_shape(
        dszg,
        &shape,
        style,
        inner_border_style,
        transform,
        painter,
    );
    for (coord, v) in shape_container.iter() {
        render_val(coord, v, transform, painter);
    }
}

// TODO Replace with Rust width separators
//////////////////////////////////////////////////////////////////////////////

pub fn render_grid_rect<SZ: SizedGrid>(
    szg: &SZ,
    style_for_coord: impl Fn(&SZ::Coord, bool) -> CellStyle,
    label_for_coord: impl Fn(&SZ::Coord) -> Option<String>,
    dark_mode: bool,
    clip: bool,
    min: glam::Vec2,
    max: glam::Vec2,
    grid_offset: Pos2,
    transform: &RectTransform,
    painter: &Painter,
) {
    // The rectangle is empty, so nothing to render.
    if !min.cmple(max).all() {
        return;
    }

    // Optionally clip all drawing with in the specified rectangle.
    let painter = if clip {
        let rect = Rect::from_min_max(glam_vec2_to_egui_pos2(min), glam_vec2_to_egui_pos2(max));
        &painter.with_clip_rect(rect.clone())
    } else {
        painter
    };

    let offset_vec = egui_pos2_to_glam_vec2(grid_offset);
    let min_offset = min + offset_vec;
    let max_offset = max + offset_vec;

    let mut show_origin = None;
    // Some debugging code to aid in validating screen_rect_to_grid.
    //let mut seen: HashSet<dynamic::Coord> = HashSet::new();
    for coord in szg.screen_rect_to_grid(min_offset, max_offset).unwrap() {
        //assert!(seen.insert(coord.clone()), "Duplicate coordinate: {coord}");

        // Draw the origin last as depending on the styling it could be
        // obscured by other cells.
        if coord.is_origin() {
            show_origin = Some(coord);
            continue;
        }

        render_coord_cell(
            szg,
            &coord,
            &style_for_coord(&coord, dark_mode),
            label_for_coord(&coord),
            transform,
            &painter,
        );
    }

    if let Some(origin) = show_origin {
        render_coord_cell(
            szg,
            &origin,
            &style_for_coord(&origin, dark_mode),
            label_for_coord(&origin),
            transform,
            &painter,
        );
    }
}

//////////////////////////////////////////////////////////////////////////////

pub struct GridView<'l, SZ: SizedGrid> {
    pub show_base_grid: bool,
    // TODO Also allow configuring with modifiers, etc.
    pub scroll_wheel_zoom: bool,
    pub pan_with_drag: bool,
    pub clear_background: bool,
    pub light_clear_color: Color32,
    pub dark_clear_color: Color32,
    // TODO Generalize
    pub style_for_coord: Box<dyn Fn(&SZ::Coord, bool) -> CellStyle + 'l>,
    pub label_for_coord: Box<dyn Fn(&SZ::Coord) -> Option<String> + 'l>,
    // Function to construct a `SizedGrid` with the given inradius.
    szg_fn: Box<dyn Fn(f32) -> SZ + 'l>,
    // Optional limits on panning the view.
    min_coord: Option<SZ::Coord>,
    max_coord: Option<SZ::Coord>,
    min_inradius: f32,
    max_inradius: f32,
    inradius: &'l mut f32,
    panning_offset: &'l mut Option<Pos2>,
    // mouse: Pos2,
}

pub struct GridContext<'l, SZ: SizedGrid> {
    pub ui: &'l mut egui::Ui,
    pub response: egui::Response,
    pub szg: SZ,
    pub to_screen_transform: RectTransform,
    pub dark_mode: bool,
    pub painter: Painter,
}

impl<'l, SZ: SizedGrid> GridView<'l, SZ> {
    // TODO Variant with fixed inradius and panning?
    pub fn new(
        inradius: &'l mut f32,
        panning_offset: &'l mut Option<Pos2>,
        szg_fn: impl Fn(f32) -> SZ + 'l,
        min_coord: Option<SZ::Coord>,
        max_coord: Option<SZ::Coord>,
        min_cell_size: f32,
        max_cell_size: f32,
        show_base_grid: bool,
        scroll_wheel_zoom: bool,
        pan_with_drag: bool,
        clear_background: bool,
        light_clear_color: Color32,
        dark_clear_color: Color32,
        style_for_coord: impl Fn(&SZ::Coord, bool) -> CellStyle + 'l,
        label_for_coord: impl Fn(&SZ::Coord) -> Option<String> + 'l,
    ) -> Self {
        Self {
            show_base_grid,
            scroll_wheel_zoom,
            pan_with_drag,
            clear_background,
            light_clear_color,
            dark_clear_color,
            style_for_coord: Box::new(style_for_coord),
            label_for_coord: Box::new(label_for_coord),
            szg_fn: Box::new(szg_fn),
            min_coord,
            max_coord,
            min_inradius: min_cell_size,
            max_inradius: max_cell_size,
            inradius: inradius,
            panning_offset: panning_offset,
        }
    }

    pub fn render<'a>(&mut self, ui: &mut egui::Ui, mut child: impl FnMut(GridContext<SZ>) -> () + 'a) {
        // TO Will response.rect match ui.available_size(), if so simplify using that.

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());

        // Check if there was a scroll-wheel delta if the mouse inside the
        // response rectangle.
        if self.scroll_wheel_zoom {
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

            // TODO Also need to clamp so the grid doesn't get too small for min and max
            // Apply the scroll-wheel delta to the grid size.
            if let Some(delta) = delta {
                *self.inradius = *self.inradius + delta.y;
            }
        }

        // Construct a dynamic sized grid based upon the current selected
        // grid kind and size.
        let szg = (self.szg_fn)(*self.inradius);

        // TODO Move init out of GridView
        if self.panning_offset.is_none() {
            // Center the grid initially.
            let center = response.rect.center() * -1.0;
            //  let screen_center =
            // szg.grid_to_screen(&dynamic::Coord::origin(self.grid_kind));
            *self.panning_offset = Some(center) //Some((center /*- Pos2::new(screen_center.x, screen_center.y) */).to_pos2());
        }

        // Check if the mouse button was dragged, and if so adjust the
        // panning offset.
        if self.pan_with_drag {
            let prd = ui.interact(response.rect, response.id, Sense::drag());
            if prd.dragged() {
                *self.panning_offset = Some(self.panning_offset.unwrap() + prd.drag_delta());
                // TODO Clamp based on min and max coordinates.
            }
        }

        let dark_mode = ui.visuals().dark_mode;

        // Clear the background if requested.
        if self.clear_background {
            painter.rect_filled(
                painter.clip_rect(),
                0.0,
                if dark_mode {
                    self.dark_clear_color
                } else {
                    self.light_clear_color
                },
            );
        }

        // Construct a transform that maps from the viewport specified by the
        // panning offset and the size of the painting rectangle to the screen
        // coordinates.  Note that we do not want to use the minimum,
        // coordinate of the rect as the target, as its upper left corner is
        // always zero for the purposes of painting.
        let to_screen_transform = RectTransform::from_to(
            Rect::from_min_size(self.panning_offset.unwrap(), response.rect.size()),
            response.rect,
            //Rect::from_min_size(Pos2::ZERO, response.rect.size()),
        );

        //  println!("Transform: {:?}", to_screen_transform);
        //  println!("response.rect: {:?}", response.rect);

        // Render the base grid if requested.
        if self.show_base_grid {
            render_grid_rect(
                &szg,
                self.style_for_coord.deref(),
                self.label_for_coord.deref(),
                dark_mode,
                // TODO clipping rect doesn't match the view rect.
                false, /* clip to rect */
                //true, /* clip to rect */
                egui_pos2_to_glam_vec2(Pos2::ZERO),
                egui_pos2_to_glam_vec2(response.rect.size().to_pos2()),
                //egui_pos2_to_glam_vec2(painter.clip_rect().min),
                //egui_pos2_to_glam_vec2(painter.clip_rect().max),
                // egui_pos2_to_glam_vec2(response.rect.min),
                // egui_pos2_to_glam_vec2(response.rect.max),
                self.panning_offset.unwrap(),
                &to_screen_transform,
                &painter,
            );
        }

        child(GridContext {
            ui,
            response,
            szg,
            to_screen_transform, //.clone(),
            dark_mode,
            painter, //.clone(),
        });
    }
}
