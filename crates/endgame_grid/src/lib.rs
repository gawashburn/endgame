use endgame_direction::{Direction, DirectionSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{RangeBounds, RangeFull, RangeTo, RangeToInclusive};

//////////////////////////////////////////////////////////////////////////////////////////////////

/// A flag that can be used to indicate whether we are referencing directions
/// for the face or a vertex of a grid cell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DirectionType {
    /// A direction that is aligned with the faces of a grid cell.
    #[default]
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

impl std::ops::Not for DirectionType {
    type Output = Self;

    /// Produce the opposite (or perhaps dual) of the given `DirectionType`.
    fn not(self) -> Self::Output {
        use DirectionType::*;
        match self {
            Face => Vertex,
            Vertex => Face,
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Color assignment values for grid coordinates.  The four color theorem
/// proves that for any loopless planar graph no more colors are needed to
/// color adjacent nodes so that no two adjacent nodes have the same color.  
/// So as long as the coordinate system is isomorphic to a loopless planar
/// graph, we can use just four colors provide a suitable coloring  for a grid.
/// Should this library expand beyond such coordinate systems, this design
/// choice will need to be revisited.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Color {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl TryFrom<usize> for Color {
    type Error = String;
    /// Try converting a `usize` to a `Color`.  This will only produce a result
    /// if the value is in the range `1..=4`.
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let color = match value {
            1 => Color::One,
            2 => Color::Two,
            3 => Color::Three,
            4 => Color::Four,
            _ => {
                return Err(format!(
                    "Color value must be in the range 1..=4, got {value}"
                ));
            }
        };
        Ok(color)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Color::*;
        let str = match self {
            One => "One",
            Two => "Two",
            Three => "Three",
            Four => "Four",
        };
        write!(f, "{}", str)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// An abstract representation of coordinates in a grid system.
pub trait Coord: PartialEq + Eq + Clone + Hash + Debug + Display + Sync + Send {
    /// The type of the axes for this coordinate system.
    type Axes: Clone + Copy + PartialEq + Eq + Hash + Debug + Display;

    /// Is this coordinate the origin of the grid?
    fn is_origin(&self) -> bool;

    /// Compute the "Manhattan" distance between the two coordinates.
    /// That is the distance between the two coordinates by only
    /// traversing along the face directions of the grid.
    fn distance(&self, other: &Self) -> usize;

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

    /// Produce the coordinate that result from moving in the given `Direction`.
    /// Returns None if the  `Direction` is not allowed for this coordinate.
    fn move_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self>;

    /// Produce the coordinate that results from moving along the given
    /// axis, either in the positive or negative direction.
    fn move_on_axis(&self, axis: Self::Axes, positive: bool) -> Self;

    /// Produce an iterator that will step through coordinates in the
    /// given `Direction`.  The provided `RangeBounds` can be used to
    /// constrain the end coordinates of the Iterator.
    /// If the specified `Direction` is not allowed for this coordinate,
    /// the iterator will be empty.
    // TODO Even better would be a version that returns a `Step`.  However,
    // that is presently a nightly-only experimental feature.
    fn direction_iterator<RB: AllowedCoordIterRange>(
        &self,
        dir_type: DirectionType,
        dir: Direction,
        range: RB,
    ) -> impl Iterator<Item=Self>;

    /// Produce an iterator that will step through coordinates between
    /// this `Coord` and the `other` `Coord`.  The path produced by the
    /// iterator will be inclusive and contain both the `self` and `other`
    /// coordinates.
    ///
    /// The length is guaranteed to be one more than the `distance` between
    /// the two coordinates.
    ///
    /// Only face directions will be traversed.  This can be thought of
    /// roughly as a "straight line" between the two coordinates.  However,
    /// this is not quite the same as a "line drawing algorithm" as
    /// some line drawing algorithms will traverse diagonally along
    /// vertex directions.
    fn path_iterator(&self, other: &Self) -> impl Iterator<Item=Self>;

    /// Produce an iterator that will step through coordinates along the
    /// given axis, either in the positive or negative direction.
    /// The provided `RangeBounds` can be used to constrain the end
    /// coordinates of the Iterator.
    // TODO If we could use `Step` instead we could omit the `positive`
    // parameter.
    fn axis_iterator<RB: AllowedCoordIterRange>(
        &self,
        axis: Self::Axes,
        positive: bool,
        range: RB,
    ) -> impl Iterator<Item=Self>;

    /// Is it possible to move in the given `Direction` from this
    /// coordinate?
    fn allowed_direction(&self, dir_type: DirectionType, dir: Direction) -> bool;

    /// Which `Direction`s are allowed from this coordinate?
    fn allowed_directions(&self, dir_type: DirectionType) -> DirectionSet;

    /// Convert the coordinate to a pair of offsets suitable for
    /// indexing into a 2D array.
    fn grid_to_array_offset(&self) -> (isize, isize);

    /// Provide a coloring for this coordinate such that no adjacent coordinate
    /// will have the same color.
    fn to_color(&self) -> Color;

    /// Product a coordinate by rotating this one around the origin clockwise.
    fn rotate_clockwise(&self) -> Self;

    /// Product a coordinate by rotating this one around the origin
    /// counter-clockwise.
    fn rotate_counterclockwise(&self) -> Self;

    /// Rotate the coordinate around the origin by the given number of steps.
    /// Positive steps rotate clockwise, negative steps rotate
    /// counter-clockwise. Zero steps is a no-op.
    fn rotate(&self, steps: isize) -> Self {
        let mut result = self.clone();
        if steps > 0 {
            for _ in 0..steps {
                result = result.rotate_clockwise();
            }
        } else {
            for _ in 0..(-steps) {
                result = result.rotate_counterclockwise();
            }
        }
        result
    }

    /// Produce a coordinate by reflecting this one across the given axis,
    /// centered around the origin.
    fn reflect(&self, axis: Self::Axes) -> Self;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// A trait for those coordinate systems that satisfy the properties of
/// being an algebraic module.  While not strictly required by the `Coord`
/// trait, the focus of this crate is on discrete grid systems.  As such,
/// they will not satisfy the properties of a vector or linear space,
/// because their scalar multiplication is not defined for a field.
///
/// Furthermore, `Coord` is general enough to support useful grid
/// systems that do not satisfy the properties of a module (such as
/// triangular grids), so we provide `ModuleCoord` as a refinement
/// for those coordinate systems that do.
pub trait ModuleCoord:
Coord
+ Default // Default of ModuleCoord should also the additive unit.
+ std::ops::Neg<Output=Self>
+ std::ops::Add<Output=Self>
+ std::ops::Sub<Output=Self>
+ std::ops::AddAssign
+ std::ops::SubAssign
+ std::ops::Mul<isize, Output=Self>
+ std::ops::MulAssign<isize>
where
        for<'a> Self: std::ops::Add<&'a Self, Output=Self>,
        for<'a, 'b> &'a Self: std::ops::Add<&'b Self, Output=Self>,
        for<'a> Self: std::ops::AddAssign<&'a Self>,
        for<'a> Self: std::ops::Sub<&'a Self, Output=Self>,
        for<'a, 'b> &'a Self: std::ops::Sub<&'b Self, Output=Self>,
        for<'a> Self: std::ops::SubAssign<&'a Self>,
{
    /// Produce the offset that when added to this coordinate that would
    /// result in a move in the given `Direction`.  Returns None if the
    /// `Direction` is not allowed for this coordinate.
    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self>;

    /// Produce the offset that when added to this coordinate that would
    /// result in a move along the given axis, either in the positive or
    /// negative direction.
    fn offset_on_axis(&self, axis: Self::Axes, positive: bool) -> Self;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// `AllowedCoordIterRange` is a helper trait to constrain the type of
/// `RangeBounds` we want to allow for `CoordIter`.
pub trait AllowedCoordIterRange: RangeBounds<usize> {
    /// Check if iteration is complete based on the provided index.
    fn complete(&self, index: usize) -> bool {
        match self.end_bound() {
            // If the end bound is inclusive, we can use the index as is.
            std::ops::Bound::Included(&end) => index > end,
            // If the end bound is exclusive, we need to check if we are at the end.
            std::ops::Bound::Excluded(&end) => index >= end,
            // If there is no end bound, we can continue indefinitely.
            std::ops::Bound::Unbounded => false,
        }
    }
}

impl AllowedCoordIterRange for RangeFull {}
impl AllowedCoordIterRange for RangeTo<usize> {}
impl AllowedCoordIterRange for RangeToInclusive<usize> {}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Use `glam::Vec2` as the representation of points in screen space.
type Point = glam::Vec2;

/// Extend abstract grid coordinates with a specific cell size.
pub trait SizedGrid {
    type Coord: Coord;

    /// Inradius of a `Coord` in screen space.
    fn inradius(&self) -> f32;

    /// Circumradius of a `Coord in screen space.
    fn circumradius(&self) -> f32;

    /// Length of an edge of a `Coord` in screen space.
    fn edge_length(&self) -> f32;

    /// Obtain the vertices of the `Coord` in screen space.
    /// Will be returned in clockwise order.
    // TODO Is there a reason to specify that a particular vertex
    // will be first?
    fn vertices(&self, coord: &Self::Coord) -> Vec<Point>;

    /// Obtain the edges of the `Coord` in screen space.
    /// Currently, each edge maps to a `Direction`, but this might need
    /// to be revisited in the future.  The edges should be exactly the
    /// same as the allowed face directions.
    fn edges(&self, coord: &Self::Coord) -> HashMap<Direction, (Point, Point)>;

    /// Convert a `Coord` to a point in screen space corresponding
    /// to the center of that coordinate.
    fn grid_to_screen(&self, coord: &Self::Coord) -> Point;

    /// Convert a point in screen space to a `Coord`.
    fn screen_to_grid(&self, point: Point) -> Self::Coord;

    /// Given a rectangle defined by two points in screen space,
    /// produce an iterator over the coordinates that intersect
    /// with that rectangle.
    ///
    /// Both elements on the `min` point must less than or equal to
    /// `max` point, otherwise `None` will be returned.
    fn screen_rect_to_grid(
        &self,
        min: Point,
        max: Point,
    ) -> Option<impl Iterator<Item=Self::Coord>>;

    /// Check if a given `Coord` contains the provided `Point`.
    fn coord_contains(&self, coord: &Self::Coord, point: Point) -> bool {
        self.grid_to_screen(coord) == point
    }

    /// Check if a given `Coord` intersects with the given rectangle.
    fn coord_intersects_rect(&self, coord: &Self::Coord, min: Point, max: Point) -> bool {
        utils::convex_poly_intersects_rect(&self.vertices(coord), min, max)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// An abstraction for representing finite portions of an infinite grid plane.
pub trait Shape<C: Coord>:
Debug + Clone + PartialEq + Eq + Hash + IntoIterator
where
    Self: std::ops::Sub<Output=Self>,
    for<'a> Self: std::ops::Sub<&'a Self, Output=Self>,
    for<'b> Self: std::ops::Sub<&'b Self, Output=Self>,
    for<'a, 'b> &'a Self: std::ops::Sub<&'b Self, Output=Self>,
{
    type Iterator<'a>: ShapeIterator<'a, C>
    where
        Self: 'a,
        C: 'a;

    /// Create an empty `Shape`.
    fn new() -> Self;

    /// Checks whether the given grid coordinate is contained within the
    /// bounds `Shape`.  This is relatively generic as it allows for
    /// grids to have irregularly shaped bounds.
    fn contains(&self, coord: &C) -> bool;

    /// Are the coordinate of this shape contained within the other shape?
    fn is_subshape(&self, other: &Self) -> bool;

    /// Does this shape contain all coordinates of the other shape?
    fn is_supershape(&self, other: &Self) -> bool;

    /// Are the coordinates of this shape disjoint from the other shape?
    fn is_disjoint(&self, other: &Self) -> bool;

    /// Are there no coordinates in this shape?
    fn is_empty(&self) -> bool;

    /// Create a shape by combining the coordinates of this shape with
    /// those of the other shape.
    fn union<'a>(&'a self, other: &'a Self) -> Self
    where
        C: 'a;

    /// Obtain an iterator over coordinates in the `Shape`.
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        C: 'a;
}

/// A trait for iterators over coordinates in a `Shape`.
pub trait ShapeIterator<'a, C: Coord + 'a>: Iterator<Item=&'a C> {}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// As specialization of `Shape` for those coordinate systems that satisfy
/// ModuleCoord and thus support translation.
pub trait ModuleShape<MC: ModuleCoord>: Shape<MC>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
        for<'a, 'b> &'a Self: std::ops::Sub<&'b Self, Output=Self>,
{
    /// Translate the shape by the given coordinate offset.
    fn translate(&self, offset: &MC) -> Self;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// An abstraction for associating values with coordinates in a finite
/// portion of an infinite grid plane.
pub trait ShapeContainer<C: Coord, V>:
Debug + Clone + PartialEq + Eq + Hash + IntoIterator
where
    V: Debug + Clone + PartialEq + Eq + Hash,
    Self::Shape: std::ops::Sub<Output=Self::Shape>,
    for<'a> Self::Shape: std::ops::Sub<&'a Self::Shape, Output=Self::Shape>,
    for<'b> Self::Shape: std::ops::Sub<&'b Self::Shape, Output=Self::Shape>,
    for<'a, 'b> &'a Self::Shape: std::ops::Sub<&'b Self::Shape, Output=Self::Shape>,
{
    type Iterator<'a>: ShapeContainerIterator<'a, C, V>
    where
        Self: 'a,
        C: 'a,
        V: 'a;

    type Shape: Shape<C>;

    /// Checks whether the given grid coordinate is contained within the
    /// bounds `Shape`.  This is relatively generic as it allows for
    /// grids to have irregularly shaped bounds.
    fn contains(&self, coord: &C) -> bool;

    /// Retrieves an immutable reference the value stored at the given
    /// grid coordinate, or `None` if the coordinate is not within the bounds
    /// of the grid.
    fn get(&self, coord: &C) -> Option<&V>;

    /// Retrieves a mutable reference to the value stored at the given
    /// grid coordinate, or `None` if the coordinate is not within the bounds
    /// of the grid.
    fn get_mut(&mut self, coord: &C) -> Option<&mut V>;

    /// Associates a value with a given coordinate on the grid, potentially
    /// replacing an existing value. Returns the previous value associated
    /// with the coordinate, if it exists.
    fn insert(&mut self, coord: C, value: V) -> Option<V>;

    /// Are there no coordinates in this shape?
    fn is_empty(&self) -> bool;

    /// Strip the contents and obtain the cooresponding `Shape`.
    fn as_shape(&self) -> Self::Shape;

    /// Obtain an iterator over coordinates and values in the `ShapeContainer`.
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        C: 'a,
        V: 'a;
}
/// A trait for iterators over coordinates and their values in a
/// `ShapeContainer`.
pub trait ShapeContainerIterator<'a, C: Coord + 'a, V: 'a>:
Iterator<Item=(&'a C, &'a V)>
{}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// As specialization of `ShapeContainer` for those coordinate systems that
/// satisfy ModuleCoord and thus support translation.
pub trait ModuleShapeContainer<MC: ModuleCoord, V>: ShapeContainer<MC, V>
where
    V: Debug + Clone + PartialEq + Eq + Hash,
    for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
    for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
    Self::Shape: std::ops::Sub<Output=Self::Shape>,
    for<'a> Self::Shape: std::ops::Sub<&'a Self::Shape, Output=Self::Shape>,
    for<'b> Self::Shape: std::ops::Sub<&'b Self::Shape, Output=Self::Shape>,
    for<'a, 'b> &'a Self::Shape: std::ops::Sub<&'b Self::Shape, Output=Self::Shape>,
{
    /// Translate the shape by the given coordinate offset.
    fn translate(&self, offset: &MC) -> Self;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub mod dynamic;
pub mod hex;
pub mod shape;
pub mod square;
pub mod triangle;
mod utils;
