//! This crate provides a "dynamic" coordinate that dispatches to one
//! of the other concrete coordinate types.  This is useful when
//! implementing code that is not only generic in the coordinate system,
//! but must be able to switch between them at runtime.
//!
//! At present, Coord is not dyn compatible, so this appears to be
//! the only way to allow runtime dispatch between different
//! coordinate types.  Should a better solution be found in the future,
//! it may be possible to eliminate this type.  That would be preferable,
//! as it is not possible for it to implement some potentially useful
//! operations (such as `Default`, construction from array
//! coordinates, etc.) on the Coord trait.
//!
//! Overall, the implementation is the obvious boilerplate.  I have
//! investigated various crates that provide macros for generating
//! the needed boilerplate, but they all appear to have deficiencies.

use crate::shape::HashShape;
use crate::{hex, square, AllowedCoordIterRange, DirectionType};
use crate::{triangle, Color, Shape};
use endgame_direction::{Direction, DirectionSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Kind {
    Square,
    Hex,
    Triangle,
}

impl Kind {
    /// Return the number of vertices for this coordinate kind.
    // TODO Offer on coordinates themselves?
    pub fn num_vertices(self) -> usize {
        use Kind::*;
        match self {
            Square => 4,
            Hex => 6,
            Triangle => 3,
        }
    }

    /// Return the axes for this coordinate kind.
    pub fn axes(self) -> Vec<Axes> {
        use Kind::*;
        match self {
            Square => {
                use square::Axes::*;
                vec![X.into(), Y.into()]
            }
            Hex => {
                use hex::Axes::*;
                vec![Q.into(), R.into(), S.into()]
            }
            Triangle => {
                use triangle::Axes::*;
                vec![A.into(), B.into(), C.into()]
            }
        }
    }

    /// Is this a coordinate that also supports ModuleCoord operations?
    // TODO Offer on coordinates themselves?
    pub fn is_modular(self) -> bool {
        use Kind::*;
        matches!(self, Square | Hex)
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Kind::*;
        let str = match self {
            Square => "Square",
            Hex => "Hex",
            Triangle => "Triangle",
        };
        write!(f, "{}", str)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Coord {
    Square(square::Coord),
    Hex(hex::Coord),
    Triangle(triangle::Coord),
}

impl Coord {
    /// Obtain the origin coordinate for the given kind.
    pub fn origin(kind: Kind) -> Self {
        use Kind::*;
        match kind {
            Square => square::Coord::default().into(),
            Hex => hex::Coord::default().into(),
            Triangle => triangle::Coord::default().into(),
        }
    }

    /// Obtain the underlying kind of the coordinate.
    pub fn kind(&self) -> Kind {
        use Coord::*;
        match self {
            Square(_) => Kind::Square,
            Hex(_) => Kind::Hex,
            Triangle(_) => Kind::Triangle,
        }
    }

    // TODO Is there a more efficient way to implement this without
    //   1. having to expose that all versions implement HashShape and
    //   2. having to clone all the coordinates?
    pub fn ring(kind: Kind, radius: usize) -> HashShape<Coord> {
        use Kind::*;
        let coords: Vec<Coord> = match kind {
            Square => square::Coord::ring(radius)
                .iter()
                .cloned()
                .map(Coord::Square)
                .collect(),
            Hex => hex::Coord::ring(radius)
                .iter()
                .cloned()
                .map(Coord::Hex)
                .collect(),
            Triangle => triangle::Coord::ring(radius)
                .iter()
                .cloned()
                .map(Coord::Triangle)
                .collect(),
        };
        HashShape::from_iter(coords)
    }

    pub fn range(kind: Kind, radius: usize) -> HashShape<Coord> {
        use Kind::*;
        let coords: Vec<Coord> = match kind {
            Square => square::Coord::range(radius)
                .iter()
                .cloned()
                .map(Coord::Square)
                .collect(),
            Hex => hex::Coord::range(radius)
                .iter()
                .cloned()
                .map(Coord::Hex)
                .collect(),
            Triangle => triangle::Coord::range(radius)
                .iter()
                .cloned()
                .map(Coord::Triangle)
                .collect(),
        };
        HashShape::from_iter(coords)
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Coord::*;
        match self {
            Square(coord) => coord.fmt(f),
            Hex(coord) => coord.fmt(f),
            Triangle(coord) => coord.fmt(f),
        }
    }
}

impl From<square::Coord> for Coord {
    fn from(value: square::Coord) -> Self {
        Coord::Square(value)
    }
}

impl From<hex::Coord> for Coord {
    fn from(value: hex::Coord) -> Self {
        Coord::Hex(value)
    }
}

impl From<triangle::Coord> for Coord {
    fn from(value: triangle::Coord) -> Self {
        Coord::Triangle(value)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Axes {
    Square(square::Axes),
    Hex(hex::Axes),
    Triangle(triangle::Axes),
}

impl Axes {
    pub fn kind(&self) -> Kind {
        use Axes::*;
        match self {
            Square(_) => Kind::Square,
            Hex(_) => Kind::Hex,
            Triangle(_) => Kind::Triangle,
        }
    }
}

impl Display for Axes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Axes::*;
        match self {
            Square(axis) => axis.fmt(f),
            Hex(axis) => axis.fmt(f),
            Triangle(axis) => axis.fmt(f),
        }
    }
}

impl From<square::Axes> for Axes {
    fn from(value: square::Axes) -> Self {
        Axes::Square(value)
    }
}

impl From<hex::Axes> for Axes {
    fn from(value: hex::Axes) -> Self {
        Axes::Hex(value)
    }
}

impl From<triangle::Axes> for Axes {
    fn from(value: triangle::Axes) -> Self {
        Axes::Triangle(value)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// CoordIterator allows for wrapping iterators over the different coordinate
/// types so that they can be returned from those trait methods that return
/// iterators without having to materialize the entire iterator.
#[derive(Debug, Clone)]
enum CoordIter<
    S: Iterator<Item=square::Coord>,
    H: Iterator<Item=hex::Coord>,
    T: Iterator<Item=triangle::Coord>,
> {
    Square(S),
    Hex(H),
    Triangle(T),
}

impl<
    S: Iterator<Item=square::Coord>,
    H: Iterator<Item=hex::Coord>,
    T: Iterator<Item=triangle::Coord>,
> Iterator for CoordIter<S, H, T>
{
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        use CoordIter::*;
        match self {
            Square(iter) => iter.next().map(Coord::Square),
            Hex(iter) => iter.next().map(Coord::Hex),
            Triangle(iter) => iter.next().map(Coord::Triangle),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

impl crate::Coord for Coord {
    type Axes = Axes;

    fn is_origin(&self) -> bool {
        use Coord::*;
        match self {
            Square(coord) => coord.is_origin(),
            Hex(coord) => coord.is_origin(),
            Triangle(coord) => coord.is_origin(),
        }
    }

    fn distance(&self, other: &Self) -> usize {
        use Coord::*;
        match (self, other) {
            (Square(a), Square(b)) => a.distance(b),
            (Hex(a), Hex(b)) => a.distance(b),
            (Triangle(a), Triangle(b)) => a.distance(b),
            _ => panic!(
                "Cannot compute distance between different kinds of Coords: {} vs {}",
                self.kind(),
                other.kind()
            ),
        }
    }

    fn angle_to_direction(&self, dir_type: DirectionType, angle: f32) -> Direction {
        use Coord::*;
        match self {
            Square(coord) => coord.angle_to_direction(dir_type, angle),
            Hex(coord) => coord.angle_to_direction(dir_type, angle),
            Triangle(coord) => coord.angle_to_direction(dir_type, angle),
        }
    }

    fn direction_angle(&self, dir_type: DirectionType, dir: Direction) -> Option<f32> {
        use Coord::*;
        match self {
            Square(coord) => coord.direction_angle(dir_type, dir),
            Hex(coord) => coord.direction_angle(dir_type, dir),
            Triangle(coord) => coord.direction_angle(dir_type, dir),
        }
    }

    fn move_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Coord::*;
        match self {
            Square(coord) => coord.move_in_direction(dir_type, dir).map(Square),
            Hex(coord) => coord.move_in_direction(dir_type, dir).map(Hex),
            Triangle(coord) => coord.move_in_direction(dir_type, dir).map(Triangle),
        }
    }

    fn move_on_axis(&self, axis: Self::Axes, positive: bool) -> Self {
        use Coord::*;
        match (self, axis) {
            (Square(coord), Axes::Square(axis)) => coord.move_on_axis(axis, positive).into(),
            (Hex(coord), Axes::Hex(axis)) => coord.move_on_axis(axis, positive).into(),
            (Triangle(coord), Axes::Triangle(axis)) => coord.move_on_axis(axis, positive).into(),
            _ => panic!(
                "Cannot move on axis for different kinds of Coords: {} vs {}",
                self.kind(),
                axis.kind()
            ),
        }
    }

    fn direction_iterator<RB: AllowedCoordIterRange>(
        &self,
        dir_type: DirectionType,
        dir: Direction,
        range: RB,
    ) -> impl Iterator<Item=Self> {
        use Coord::*;
        match self {
            Square(coord) => CoordIter::Square(coord.direction_iterator(dir_type, dir, range)),
            Hex(coord) => CoordIter::Hex(coord.direction_iterator(dir_type, dir, range)),
            Triangle(coord) => CoordIter::Triangle(coord.direction_iterator(dir_type, dir, range)),
        }
    }

    fn path_iterator(&self, other: &Self) -> impl Iterator<Item=Self> {
        use Coord::*;
        match (self, other) {
            (Square(a), Square(b)) => CoordIter::Square(a.path_iterator(b)),
            (Hex(a), Hex(b)) => CoordIter::Hex(a.path_iterator(b)),
            (Triangle(a), Triangle(b)) => CoordIter::Triangle(a.path_iterator(b)),
            _ => panic!(
                "Cannot create line iterator for different kinds of Coords: {} vs {}",
                self.kind(),
                other.kind()
            ),
        }
    }

    fn axis_iterator<RB: AllowedCoordIterRange>(
        &self,
        axis: Self::Axes,
        positive: bool,
        range: RB,
    ) -> impl Iterator<Item=Self> {
        use Coord::*;
        match (self, axis) {
            (Square(coord), Axes::Square(axis)) => {
                CoordIter::Square(coord.axis_iterator(axis, positive, range))
            }
            (Hex(coord), Axes::Hex(axis)) => {
                CoordIter::Hex(coord.axis_iterator(axis, positive, range))
            }
            (Triangle(coord), Axes::Triangle(axis)) => {
                CoordIter::Triangle(coord.axis_iterator(axis, positive, range))
            }
            _ => panic!(
                "Cannot create axis iterator for different kinds of Coords: {} vs {}",
                self.kind(),
                axis.kind()
            ),
        }
    }

    fn allowed_direction(&self, dir_type: DirectionType, dir: Direction) -> bool {
        use Coord::*;
        match self {
            Square(coord) => coord.allowed_direction(dir_type, dir),
            Hex(coord) => coord.allowed_direction(dir_type, dir),
            Triangle(coord) => coord.allowed_direction(dir_type, dir),
        }
    }

    fn allowed_directions(&self, dir_type: DirectionType) -> DirectionSet {
        use Coord::*;
        match self {
            Square(coord) => coord.allowed_directions(dir_type),
            Hex(coord) => coord.allowed_directions(dir_type),
            Triangle(coord) => coord.allowed_directions(dir_type),
        }
    }

    fn grid_to_array_offset(&self) -> (isize, isize) {
        use Coord::*;
        match self {
            Square(coord) => coord.grid_to_array_offset(),
            Hex(coord) => coord.grid_to_array_offset(),
            Triangle(coord) => coord.grid_to_array_offset(),
        }
    }

    fn to_color(&self) -> Color {
        use Coord::*;
        match self {
            Square(coord) => coord.to_color(),
            Hex(coord) => coord.to_color(),
            Triangle(coord) => coord.to_color(),
        }
    }

    fn rotate_clockwise(&self) -> Self {
        use Coord::*;
        match self {
            Square(coord) => coord.rotate_clockwise().into(),
            Hex(coord) => coord.rotate_clockwise().into(),
            Triangle(coord) => coord.rotate_clockwise().into(),
        }
    }

    fn rotate_counterclockwise(&self) -> Self {
        use Coord::*;
        match self {
            Square(coord) => coord.rotate_counterclockwise().into(),
            Hex(coord) => coord.rotate_counterclockwise().into(),
            Triangle(coord) => coord.rotate_counterclockwise().into(),
        }
    }

    fn reflect(&self, axis: Self::Axes) -> Self {
        use Coord::*;
        match (self, axis) {
            (Square(coord), Axes::Square(axis)) => coord.reflect(axis).into(),
            (Hex(coord), Axes::Hex(axis)) => coord.reflect(axis).into(),
            (Triangle(coord), Axes::Triangle(axis)) => coord.reflect(axis).into(),
            _ => panic!(
                "Cannot reflect on axis for different kind of Coords: {} vs {}",
                self.kind(),
                axis.kind()
            ),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

// TODO Cannot implement a dynamic version of ModuleCoord, as it currently
//   requires implementing the Default trait to produce the additive unit
//   value.

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SizedGrid {
    Square(square::SizedGrid),
    Hex(hex::SizedGrid),
    Triangle(triangle::SizedGrid),
}

impl SizedGrid {
    pub fn new(kind: Kind, inradius: f32) -> Self {
        match kind {
            Kind::Hex => hex::SizedGrid::new(inradius).into(),
            Kind::Square => square::SizedGrid::new(inradius).into(),
            Kind::Triangle => triangle::SizedGrid::new(inradius).into(),
        }
    }

    pub fn kind(&self) -> Kind {
        use SizedGrid::*;
        match self {
            Square(_) => Kind::Square,
            Hex(_) => Kind::Hex,
            Triangle(_) => Kind::Triangle,
        }
    }
}

impl From<square::SizedGrid> for SizedGrid {
    fn from(value: square::SizedGrid) -> Self {
        SizedGrid::Square(value)
    }
}

impl From<hex::SizedGrid> for SizedGrid {
    fn from(value: hex::SizedGrid) -> Self {
        SizedGrid::Hex(value)
    }
}

impl From<triangle::SizedGrid> for SizedGrid {
    fn from(value: triangle::SizedGrid) -> Self {
        SizedGrid::Triangle(value)
    }
}

type Point = glam::Vec2;

impl crate::SizedGrid for SizedGrid {
    type Coord = Coord;

    fn inradius(&self) -> f32 {
        use SizedGrid::*;
        match self {
            Square(grid) => grid.inradius(),
            Hex(grid) => grid.inradius(),
            Triangle(grid) => grid.inradius(),
        }
    }

    fn circumradius(&self) -> f32 {
        use SizedGrid::*;
        match self {
            Square(grid) => grid.circumradius(),
            Hex(grid) => grid.circumradius(),
            Triangle(grid) => grid.circumradius(),
        }
    }

    fn edge_length(&self) -> f32 {
        use SizedGrid::*;
        match self {
            Square(grid) => grid.edge_length(),
            Hex(grid) => grid.edge_length(),
            Triangle(grid) => grid.edge_length(),
        }
    }

    fn vertices(&self, coord: &Self::Coord) -> Vec<Point> {
        use SizedGrid::*;
        match (self, coord) {
            (Square(grid), Coord::Square(coord)) => grid.vertices(coord),
            (Hex(grid), Coord::Hex(coord)) => grid.vertices(coord),
            (Triangle(grid), Coord::Triangle(coord)) => grid.vertices(coord),
            _ => {
                panic!("Expected matching Coord type for SizedGrid");
            }
        }
    }

    fn edges(&self, coord: &Self::Coord) -> HashMap<Direction, (Point, Point)> {
        use SizedGrid::*;
        match (self, coord) {
            (Square(grid), Coord::Square(coord)) => grid.edges(coord),
            (Hex(grid), Coord::Hex(coord)) => grid.edges(coord),
            (Triangle(grid), Coord::Triangle(coord)) => grid.edges(coord),
            _ => {
                panic!("Expected matching Coord type for SizedGrid");
            }
        }
    }

    fn grid_to_screen(&self, coord: &Self::Coord) -> Point {
        use SizedGrid::*;
        match (self, coord) {
            (Square(grid), Coord::Square(coord)) => grid.grid_to_screen(coord),
            (Hex(grid), Coord::Hex(coord)) => grid.grid_to_screen(coord),
            (Triangle(grid), Coord::Triangle(coord)) => grid.grid_to_screen(coord),
            _ => {
                panic!("Expected matching Coord type for SizedGrid");
            }
        }
    }

    fn screen_to_grid(&self, point: Point) -> Coord {
        use SizedGrid::*;
        match self {
            Square(grid) => grid.screen_to_grid(point).into(),
            Hex(grid) => grid.screen_to_grid(point).into(),
            Triangle(grid) => grid.screen_to_grid(point).into(),
        }
    }

    fn screen_rect_to_grid(
        &self,
        min: crate::Point,
        max: crate::Point,
    ) -> Option<impl Iterator<Item=Self::Coord>> {
        use SizedGrid::*;
        match self {
            Square(grid) => grid.screen_rect_to_grid(min, max).map(CoordIter::Square),
            Hex(grid) => grid.screen_rect_to_grid(min, max).map(CoordIter::Hex),
            Triangle(grid) => grid.screen_rect_to_grid(min, max).map(CoordIter::Triangle),
        }
    }
}
