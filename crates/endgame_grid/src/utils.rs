use crate::shape::HashShape;
use crate::Coord;
pub(crate) use crate::{AllowedCoordIterRange, ModuleCoord};
use glam::Vec2;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::HashSet;

//////////////////////////////////////////////////////////////////////////////

/// Helper to convert a slice of polygon vertices into an iterator over its edges.
pub fn vertices_to_edges(vertices: &[Vec2]) -> impl Iterator<Item = (Vec2, Vec2)> {
    assert!(vertices.len() >= 3, "Polygon must have at least 3 vertices");
    // To ensure that we handle the end from the last vertex to the first,
    // we add it to the end before windowing.
    let mut edges = Vec::from(vertices);
    edges.push(vertices.first().unwrap().clone());
    edges.into_iter().tuple_windows::<(_, _)>()
}

/// Helper to check if a convex polygon intersects a rectangle.
/// Returns true if the two intersect (touching does not count).
/// The algorithm is based on a
/// specialization of the more general Separating Axis Theorem (SAT).
/// See https://en.wikipedia.org/wiki/Hyperplane_separation_theorem#Use_in_collision_detection
/// Essentially, we check if any of the normals of the polygon
/// or rectangle can be used as a separating axis. This is done by
/// projecting each vertex onto these potential axes and checking
/// if the intervals overlap.
pub fn convex_poly_intersects_rect(polygon: &[Vec2], min: Vec2, max: Vec2) -> bool {
    assert!(polygon.len() >= 3, "Polygon must have at least 3 vertices");

    // Project a slice of vertices onto a candidate axis.
    // Returns the minium and maximum of the projections.
    fn project_verts(vertices: &[Vec2], axis: Vec2) -> (f32, f32) {
        let dots: Vec<OrderedFloat<f32>> =
            vertices.iter().map(|v| OrderedFloat(v.dot(axis))).collect();
        // TODO Optimize to use a single pass
        (
            dots.iter()
                .min()
                .expect("Polygon has at least 3 verticies")
                .0,
            dots.iter()
                .max()
                .expect("Polygon has at least 3 verticies")
                .0,
        )
    }

    let rect_vertices = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];

    // Helper to check if the axis can be used as a separating axis.
    let check_axis = |axis: Vec2| -> bool {
        let (pmin, pmax) = project_verts(polygon, axis);
        let (rmin, rmax) = project_verts(&rect_vertices, axis);
        // Strict interval overlap check, such that touching is not
        // considered as overlapping.
        (pmax > rmin + f32::EPSILON) && (rmax > pmin + f32::EPSILON)
    };

    // Test the rectangle's axes first
    for axis in [Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)] {
        if !check_axis(axis) {
            return false; // Separating axis found.
        }
    }

    // Test the polygon's edge normals.
    for (a, b) in vertices_to_edges(polygon) {
        let edge = b - a;
        // Skip degenerate edges.
        if edge.length_squared() <= f32::EPSILON {
            continue;
        }
        if !check_axis(edge.perp()) {
            return false; // Separating axis found.
        }
    }

    // No separating axis found, so the two must overlap.
    true
}

//////////////////////////////////////////////////////////////////////////////

/// A generic implementation for producing rings by rotating to
/// find the corners, and then moving between them along the axes.
pub fn ring<C: Coord>(
    start: C,
    start_axis: C::Axes,
    flip_axis: C::Axes,
    axes: &[C::Axes],
    rotation_step: isize,
) -> HashShape<C> {
    let mut coords = HashSet::new();
    let mut current_coord = start.clone();
    let mut axis_iterator = axes
        .into_iter()
        .cycle()
        .skip_while(|a| **a != start_axis)
        .peekable();
    let mut axis_sign = false;
    // Loop until we return to the start coordinate.
    loop {
        let next_corner_coord = current_coord.rotate(rotation_step); //&current_coord);
        let mut next_coord = current_coord;
        let axis = axis_iterator
            .next()
            .expect("Axis iterator should be infinite");
        // Loop until we reach the next corner coordinate.
        loop {
            coords.insert(next_coord.clone());
            let coord = next_coord.move_on_axis(*axis, axis_sign);
            if coord == next_corner_coord {
                break;
            }
            next_coord = coord;
        }
        current_coord = next_corner_coord;
        if current_coord == start {
            break;
        }

        // Whenever we complete a full cycle of axes, flip the axis sign.
        if let Some(axis) = axis_iterator.peek()
            && **axis == flip_axis
        {
            axis_sign = !axis_sign;
        }
    }

    HashShape::from_iter(coords.into_iter())
}

//////////////////////////////////////////////////////////////////////////////

/// A generic iterator for traversing grids where the coordinates form an
/// algebraic module.
#[derive(Debug, Clone)]
pub struct ModuleCoordIter<MC: ModuleCoord, RB: AllowedCoordIterRange>
where
    for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output = MC>,
    for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output = MC>,
{
    pub coord: MC,
    pub opt_offset: Option<MC>,
    pub index: usize,
    pub range: RB,
}

impl<MC: ModuleCoord, RB: AllowedCoordIterRange> Iterator for ModuleCoordIter<MC, RB>
where
    for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output = MC>,
    for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output = MC>,
{
    type Item = MC;

    fn next(&mut self) -> Option<Self::Item> {
        // If the direction is not allowed, or the range is complete,
        // we are done
        if self.opt_offset.is_none() || self.range.complete(self.index) {
            return None;
        }

        let result = self.coord.clone();
        self.index += 1;
        self.coord += self
            .opt_offset
            .as_ref()
            .expect("Direction should have been validated before calling advance");
        Some(result)
    }
}
