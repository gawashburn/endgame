use endgame_direction::{Direction, DirectionSet};
use std::fmt::Display;
use std::ops::{RangeBounds, RangeFull, RangeTo, RangeToInclusive};

//////////////////////////////////////////////////////////////////////////////

/// A flag that can be used to indicate whether we are referencing directions
/// for the face or a vertex of a grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectionType {
    /// A direction that is aligned with the faces of a grid cell.
    Face,
    /// A direction that is aligned with the vertices of a grid cell.
    Vertex,
}

impl Display for DirectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DirectionType::*;
        match self {
            Face => write!(f, "Face"),
            Vertex => write!(f, "Vertex"),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// An abstract representation of coordinates in a grid system.
pub trait Coord:
Sized
+ Default // The default coordinate should be the additive identity.
+ PartialEq
+ Eq
+ Clone
+ std::fmt::Debug
+ Display
+ std::ops::Neg<Output=Self>
+ std::ops::Add<Output=Self>
+ std::ops::Sub<Output=Self>
+ std::ops::AddAssign
+ std::ops::SubAssign
where for <'a> Self: std::ops::AddAssign<&'a Self>,
for <'a> Self: std::ops::SubAssign<&'a Self>
{
    /// Convert an angle in radians to a `Direction` on this grid for
    /// this coordinate.  One use case would be for snapping controller
    /// input angles into the nearest `Direction`.
    ///
    /// Grids will be oriented such that the angle π/2 points "upwards" or
    /// a positive direction along the y-axis.
    fn angle_to_direction(&self, dir_type: DirectionType, angle: f32) -> Direction;

    /// Convert a `Direction` to an angle in radians for this coordinate
    /// system.  This is used because for some grid coordinate systems,
    /// the direction we move between coordinates will not strictly
    /// follow that of the directional angles.  For example, on a
    /// hexagonal grid, moving north-east will be at an angle π/6,
    /// not π/4.
    ///
    /// Returns None if the `Direction` is not allowed for this coordinate.
    fn direction_angle(&self, dir_type: DirectionType, dir: Direction) -> Option<f32>;

    /// Produce the offset that when added to this coordinate that would
    /// result in a move in the given `Direction`.  Returns None if the
    /// `Direction` is not allowed for this coordinate.
    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self>;

    /// Produce an iterator that will step through coordinates in the
    /// given `Direction`.  The provided `RangeBounds` can be used to 
    /// constrain the end coordinates of the Iterator.
    /// If the specified `Direction` is not allowed for this coordinate,
    /// the iterator will be empty.
    // TODO Provide a version for `RangeBounds<Self>`?
    // TODO Even better would be a version that returns a `Step`.  However,
    // that is presently a nightly-only experimental feature.
    fn direction_iterator<RB: RangeBounds<usize> + AllowedCoordIterRange>(
        &self,
        dir_type: DirectionType,
        dir: Direction,
        range: RB,
    ) -> CoordIter<Self, RB> {
        // This assertion should always hold with the AllowedCoordIterRange
        // constraint, but also check dynamically be sure.
        assert_eq!(range.start_bound(), std::ops::Bound::Unbounded);
        CoordIter {
                opt_offset: self.offset_in_direction(dir_type, dir),
                index: 0,
                coord: self.clone(),
                range,
            }
    }

    /// Is it possible to move in the given `Direction` from this
    /// coordinate?
    fn allowed_direction(&self, dir_type: DirectionType, dir: Direction) -> bool;

    /// Which `Direction`s are allowed from this coordinate?
    fn allowed_directions(&self, dir_type: DirectionType) -> DirectionSet;

    /// Convert the coordinate to a pair of offsets suitable for
    /// indexing into a 2D array.
    fn grid_to_array_offset(&self) -> (isize, isize);

    /// Convert offsets suitable for indexing into a 2D array into
    /// a coordinate
    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self;
}

//////////////////////////////////////////////////////////////////////////////

/// `AllowedCoordIterRange` is a helper trait to constrain the type of
/// `RangeBounds` we want to allow for `CoordIter`.
pub trait AllowedCoordIterRange {}
impl AllowedCoordIterRange for RangeFull {}
impl AllowedCoordIterRange for RangeTo<usize> {}
impl AllowedCoordIterRange for RangeToInclusive<usize> {}

/// A generic iterator for traversing grids for all kinds of coordinate systems.
#[derive(Debug, Clone)]
pub struct CoordIter<C: Coord, RB: RangeBounds<usize> + AllowedCoordIterRange> {
    opt_offset: Option<C>,
    index: usize,
    coord: C,
    range: RB,
}

impl<C: Coord, RB: RangeBounds<usize> + AllowedCoordIterRange> Iterator for CoordIter<C, RB> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        // The iterator will be empty if the direction is not allowed.
        if self.opt_offset.is_none() {
            return None;
        }

        // Check if iteration has completed.
        match self.range.end_bound() {
            // If the end bound is inclusive, we can use the index as is.
            std::ops::Bound::Included(&end) => {
                if self.index > end {
                    return None;
                }
            }
            // If the end bound is exclusive, we need to check if we are at the end.
            std::ops::Bound::Excluded(&end) => {
                if self.index >= end {
                    return None;
                }
            }
            // If there is no end bound, we can continue indefinitely.
            std::ops::Bound::Unbounded => {}
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

//////////////////////////////////////////////////////////////////////////////

/// Use `glam::Vec2` as the representation of points in screen space.
type Point = glam::Vec2;

/// Extend abstract grid coordinates with a specific cell size.
pub trait SizedGrid {
    type Coord: Coord;

    /// Construct a sized grid of this kind with the provide inradius.
    fn new(inradius: f32) -> Self;

    /// Inradius of a `GridCoord` in screen space.
    fn inradius(&self) -> f32;

    /// Circumradius of a `GridCoord in screen space.
    fn circumradius(&self) -> f32;

    /// Length of an edge of a `GridCoord` in screen space.
    fn edge_length(&self) -> f32;

    /// Obtain the vertices of the `GridCoord` in screen space.
    /// Will be returned in clockwise order.
    // TODO Is there a reason to specify that a particular vertex
    // will be first?
    fn vertices(&self, coord: Self::Coord) -> Vec<Point>;

    /// Convert a `GridCoord` to a point in screen space corresponding
    /// to the center of that coordinate.
    fn grid_to_screen(&self, coord: Self::Coord) -> Point;

    /// Convert a point in screen space to a `GridCoord`.
    fn screen_to_grid(&self, point: Point) -> Self::Coord;
}

//////////////////////////////////////////////////////////////////////////////

pub mod hex;
pub mod square;
pub mod triangle;
