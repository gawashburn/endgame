#![feature(associated_type_defaults)]

use endgame_direction::{Direction, DirectionSet};

//////////////////////////////////////////////////////////////////////////////

/// An abstract representation of coordinates in a grid system.
pub trait GridCoord: Sized + PartialEq + Eq + Clone + std::fmt::Debug + std::fmt::Display {
    /// Convert an angle in radians to a `Direction` on this grid for this
    /// coordinate.
    ///
    /// Grids will be oriented such that the angle π/2 points "upwards" or
    /// a positive direction along the y-axis.
    fn angle_to_direction(&self, angle: f32) -> Direction;

    /// Convert a `Direction` to an angle in radians for this coordinate
    /// system.  This is used because for some grid coordinate systems,
    /// the direction we move between coordinates will not strictly
    /// follow that of the directional angles.  For example, on a
    /// hexagonal grid, moving north-east will be at an angle π/6,
    /// not π/4.
    ///
    /// Returns None if the `Direction` is not allowed for this coordinate.
    fn direction_angle(&self, dir: Direction) -> Option<f32>;

    /// Produce the coordinate obtained by moving in the given `Direction`.
    /// Returns None if the `Direction` is not allowed for this coordinate.
    // TODO Would to be better to define a notion of coordinate offsets?
    fn move_in_direction(&self, dir: Direction) -> Option<Self>;

    /// Is it possible to move in the given `Direction` from this coordinate?
    fn allowed_direction(&self, dir: Direction) -> bool;

    /// Which movement `Direction`s are allowed from this coordinate?
    fn allowed_directions(&self) -> DirectionSet;

    /// Convert the coordinate to a pair of offsets suitable for
    /// indexing into a 2D array.
    fn grid_to_array_offset(&self) -> (isize, isize);

    /// Convert offsets suitable for indexing into a 2D array into
    /// a coordinate
    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self;
}

//////////////////////////////////////////////////////////////////////////////

/// Use `glam::Vec2` as the representation of points in screen space.
type Point = glam::Vec2;

/// Extend abstract grid coordinates with a specific cell size.
pub trait SizedGrid {
    type Coord: GridCoord;

    /// Inradius of a `GridCoord` in screen space.
    fn inradius(&self) -> f32;

    /// Circumradius of a `GridCoord in screen space.
    fn circumradius(&self) -> f32;

    /// Length of an edge of a `GridCoord` in screen space.
    fn edge_length(&self) -> f32;

    /// Convert a `GridCoord` to a point in screen space corresponding
    /// to the center of that coordinate.
    fn grid_to_screen(&self, coord: Self::Coord) -> Point;

    /// Convert a point in screen space to a `GridCoord`.
    fn screen_to_grid(&self, point: Point) -> Self::Coord;
}

//////////////////////////////////////////////////////////////////////////////

// TODO Add a module for square grid variant that allow direct traversal along
// the ordinal directions?  Or just add a notion of diagonals?
pub mod hex;
pub mod square;
pub mod triangle;
