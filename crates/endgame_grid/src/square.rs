use crate::shape::HashShape;
use crate::utils::{vertices_to_edges, ModuleCoordIter};
use crate::{AllowedCoordIterRange, Color, DirectionType, ModuleCoord, Point};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, IVec2, Mat2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::Display;

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Axes {
    X,
    Y,
}

impl Display for Axes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Axes::*;
        let c = match self {
            X => 'X',
            Y => 'Y',
        };
        write!(f, "{}", c)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// For a Manhattan square grid, it is possible to move in the same face
/// directions from any coordinate.
const ALLOWED_FACE_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[North, East, South, West])
};

/// For a Manhattan square grid, it is possible to move in the same vertex
/// directions from any coordinate.
const ALLOWED_VERTEX_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[NorthEast, SouthEast, SouthWest, NorthWest])
};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coord(IVec2);

impl Coord {
    /// The axes of a square grid.
    pub const AXES: [Axes; 2] = [Axes::X, Axes::Y];

    /// Construct a new `SquareGridCoord` from x and y coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        Coord(ivec2(x, y))
    }

    pub fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        // For a square grid the grid coordinates and array offsets are
        // essentially identical.
        // TODO switch array offsets to 32-bits?
        Coord(ivec2(array_offset.0 as i32, array_offset.1 as i32))
    }

    /// Construct a new `SquareGridCoord` from an `IVec2`.
    pub const fn from_ivec2(coord: IVec2) -> Self {
        Coord(coord)
    }

    /// Convert the coordinate to an `IVec2`.
    pub const fn to_ivec2(&self) -> IVec2 {
        self.0
    }

    pub fn ring(radius: usize) -> HashShape<Coord> {
        if radius == 0 {
            return HashShape::from([Coord::default()]);
        }

        crate::utils::ring(
            Coord::new(radius as i32, radius as i32),
            Axes::Y,
            Axes::Y,
            &Coord::AXES,
            -1,
        )
    }

    pub fn range(radius: usize) -> HashShape<Coord> {
        let iradius = radius as i32;
        let mut coords = Vec::new();
        for x in -iradius..=iradius {
            for y in -iradius..=iradius {
                coords.push(Coord::new(x, y));
            }
        }
        HashShape::from_iter(coords.into_iter())
    }
}
impl Default for Coord {
    fn default() -> Self {
        Coord(ivec2(0, 0))
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0.x, self.0.y)
    }
}

impl std::ops::Neg for Coord {
    type Output = Self;

    fn neg(self) -> Self {
        Coord(-self.0)
    }
}

impl std::ops::Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Coord(self.0 + other.0)
    }
}

impl std::ops::Add<&Coord> for Coord {
    type Output = Self;

    fn add(self, other: &Self) -> Self {
        Coord(self.0 + other.0)
    }
}

impl<'a, 'b> std::ops::Add<&'b Coord> for &'a Coord {
    type Output = Coord;

    fn add(self, other: &'b Coord) -> Self::Output {
        Coord(self.0 + other.0)
    }
}

impl std::ops::Sub for Coord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Coord(self.0 - other.0)
    }
}

impl std::ops::Sub<&Coord> for Coord {
    type Output = Self;

    fn sub(self, other: &Self) -> Self {
        Coord(self.0 - other.0)
    }
}

impl<'a, 'b> std::ops::Sub<&'b Coord> for &'a Coord {
    type Output = Coord;

    fn sub(self, other: &'b Coord) -> Self::Output {
        Coord(self.0 - other.0)
    }
}

impl std::ops::AddAssign for Coord {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<'a> std::ops::AddAssign<&'a Coord> for Coord {
    fn add_assign(&mut self, other: &Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Coord {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl<'a> std::ops::SubAssign<&'a Coord> for Coord {
    fn sub_assign(&mut self, other: &Self) {
        self.0 -= other.0;
    }
}

impl std::ops::Mul<isize> for Coord {
    type Output = Self;

    fn mul(self, other: isize) -> Self {
        Coord(self.0 * (other as i32))
    }
}

impl std::ops::MulAssign<isize> for Coord {
    fn mul_assign(&mut self, other: isize) {
        *self = *self * other;
    }
}

//////////////////////////////////////////////////////////////////////////////

impl crate::Coord for Coord {
    type Axes = Axes;

    fn is_origin(&self) -> bool {
        self.0 == IVec2::ZERO
    }

    fn distance(&self, other: &Self) -> usize {
        // For a square grid, the distance is the Manhattan distance.
        (other.0 - self.0).abs().element_sum() as usize
    }

    fn angle_to_direction(&self, dir_type: DirectionType, angle: f32) -> Direction {
        use Direction::*;
        use DirectionType::*;

        // For vertex directions, can simply offset the angle by π/4 and then
        // select the counter_clockwise direction.
        match dir_type {
            Vertex => self
                .angle_to_direction(Face, angle - (PI / 4.0))
                .counter_clockwise(),
            Face => {
                // We can ignore the coordinate, as angle to direction mapping
                // is the same for any coordinate.
                let norm_angle = angle.rem_euclid(2.0 * PI);
                // After normalization, it is expected that the angle will not have
                // a negative sign.
                assert!(norm_angle.is_sign_positive());
                let octant = norm_angle / (PI / 4.0);
                if octant >= 7.0 || octant < 1.0 {
                    East
                } else if octant < 3.0 {
                    North
                } else if octant < 5.0 {
                    West
                } else {
                    assert!(octant < 7.0);
                    South
                }
            }
        }
    }

    fn direction_angle(&self, dir_type: DirectionType, dir: Direction) -> Option<f32> {
        if self.allowed_direction(dir_type, dir) {
            Some(dir.angle())
        } else {
            None
        }
    }

    fn move_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        let offset = <Self as ModuleCoord>::offset_in_direction(self, dir_type, dir)?;
        Some(*self + offset)
    }

    fn move_on_axis(&self, axis: Self::Axes, positive: bool) -> Self {
        let offset = <Self as ModuleCoord>::offset_on_axis(self, axis, positive);
        *self + offset
    }

    fn direction_iterator<RB: AllowedCoordIterRange>(
        &self,
        dir_type: DirectionType,
        dir: Direction,
        range: RB,
    ) -> impl Iterator<Item = Self> {
        ModuleCoordIter {
            opt_offset: self.offset_in_direction(dir_type, dir),
            index: 0,
            coord: *self,
            range,
        }
    }

    fn path_iterator(&self, other: &Self) -> impl Iterator<Item = Self> {
        SquarePathIter::new(self, other)
    }

    fn axis_iterator<RB: AllowedCoordIterRange>(
        &self,
        axis: Self::Axes,
        positive: bool,
        range: RB,
    ) -> impl Iterator<Item = Self> {
        use Axes::*;
        use Direction::*;
        use DirectionType::Face;
        match (axis, positive) {
            (Y, true) => self.direction_iterator(Face, North, range),
            (Y, false) => self.direction_iterator(Face, South, range),
            (X, true) => self.direction_iterator(Face, East, range),
            (X, false) => self.direction_iterator(Face, West, range),
        }
    }

    fn allowed_direction(&self, dir_type: DirectionType, dir: Direction) -> bool {
        use DirectionType::*;
        // We can ignore the coordinate, as the allowed directions
        // are the same from any coordinate.
        match dir_type {
            Face => ALLOWED_FACE_DIRECTIONS.contains(dir),
            Vertex => ALLOWED_VERTEX_DIRECTIONS.contains(dir),
        }
    }

    fn allowed_directions(&self, dir_type: DirectionType) -> DirectionSet {
        use DirectionType::*;
        // We can ignore the coordinate, as the allowed directions
        // are the same from any coordinate.
        match dir_type {
            Face => ALLOWED_FACE_DIRECTIONS.clone(),
            Vertex => ALLOWED_VERTEX_DIRECTIONS.clone(),
        }
    }

    fn grid_to_array_offset(&self) -> (isize, isize) {
        // For a square grid the grid coordinates and array offsets are
        // essentially identical.
        (self.0.x as isize, self.0.y as isize)
    }

    fn to_color(&self) -> Color {
        let (x, y) = self.grid_to_array_offset();
        let num = ((x + y).rem_euclid(2) + 1) as usize;
        num.try_into().expect("Unexpected fill color index: {num}")
    }

    fn rotate_clockwise(&self) -> Self {
        Coord(ivec2(-self.0.y, self.0.x))
    }

    fn rotate_counterclockwise(&self) -> Self {
        Coord(ivec2(self.0.y, -self.0.x))
    }

    fn reflect(&self, axis: Self::Axes) -> Self {
        use Axes::*;
        let transform = match axis {
            X => ivec2(-1, 1),
            Y => ivec2(1, -1),
        };
        Coord(self.0 * transform)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl ModuleCoord for Coord {
    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Direction::*;
        use DirectionType::*;
        let offset = match (dir_type, dir) {
            (Face, North) => ivec2(0, 1),
            (Face, East) => ivec2(1, 0),
            (Face, South) => ivec2(0, -1),
            (Face, West) => ivec2(-1, 0),
            (Vertex, NorthEast) => ivec2(1, 1),
            (Vertex, SouthEast) => ivec2(1, -1),
            (Vertex, SouthWest) => ivec2(-1, -1),
            (Vertex, NorthWest) => ivec2(-1, 1),
            _ => return None,
        };
        Some(Coord(offset))
    }

    fn offset_on_axis(&self, axis: Self::Axes, positive: bool) -> Self {
        use Axes::*;
        use Direction::*;
        use DirectionType::Face;
        let dir = match (axis, positive) {
            (Y, true) => North,
            (Y, false) => South,
            (X, true) => East,
            (X, false) => West,
        };
        self.offset_in_direction(Face, dir)
            .expect("Offset in direction should always succeed")
    }
}

//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct SquarePathIter {
    current: IVec2,
    delta: IVec2,
    start_frac: Vec2,
    end_frac: Vec2,
    index: usize,
    steps: usize,
}

impl SquarePathIter {
    /// Create a new `SquarePathIter` that will traverse the path between
    /// `start` and `end`.
    pub fn new(start: &Coord, end: &Coord) -> Self {
        SquarePathIter {
            current: start.0,
            delta: (end.0 - start.0).signum(),
            start_frac: start.0.as_vec2(),
            end_frac: end.0.as_vec2(),
            index: 0,
            steps: <Coord as crate::Coord>::distance(start, end),
        }
    }
}

/// SquareLineIter is an iterator that traverses a path between two c
/// in a square grid.  It uses an algorithm that choose which axis to
/// move upon based on minimizing the error between possible next steps
/// and the linear interpolation between the start and end coordinates.
/// In testing, strictly relying on linear interpolation led to
/// awkward heuristics to deal the case where the linear path traverses
/// on, or extremely close to, a grid vertex.  This algorithm is biased
/// towards moving along the axes if either has equivalent error.
// TODO Is there a better algorithm?
impl Iterator for SquarePathIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.steps {
            return None;
        }
        // We'll return the current coordinate.
        let c = self.current;
        // Now find the next coordinate.
        let t = (self.index + 1) as f32 / self.steps as f32;
        let frac_target_coord = self.start_frac.lerp(self.end_frac, t);
        // Find which axes has the minimum error.
        let err: Vec2 = Vec2::from_slice(
            IVec2::AXES
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    // If the delta is zero, we cannot move in that direction,
                    // so we return infinity to ensure that this axes is not
                    // selected.
                    if self.delta.to_array()[i] != 0 {
                        let delta_coord = self.current + self.delta * a;
                        (frac_target_coord - delta_coord.as_vec2()).length()
                    } else {
                        f32::INFINITY
                    }
                })
                .collect::<Vec<f32>>()
                .as_slice(),
        );

        self.current += self.delta * IVec2::AXES[err.min_position()];
        self.index += 1;

        Some(Coord::from_ivec2(c))
    }
}

//////////////////////////////////////////////////////////////////////////////

// Regular square grids.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SizedGrid {
    inradius: f32,
}

impl SizedGrid {
    pub fn new(inradius: f32) -> Self {
        SizedGrid { inradius }
    }

    /// The conversion matrix from square grid coordinates to screen space.
    const fn conversion_matrix() -> Mat2 {
        Mat2::from_cols(Vec2::new(2.0, 0.0), Vec2::new(0.0, 2.0))
    }
}

impl crate::SizedGrid for SizedGrid {
    type Coord = Coord;

    fn inradius(&self) -> f32 {
        self.inradius
    }

    fn circumradius(&self) -> f32 {
        (2.0 * self.inradius) / 2.0f32.sqrt()
    }

    fn edge_length(&self) -> f32 {
        2.0 * self.inradius
    }

    fn vertices(&self, coord: &Self::Coord) -> Vec<Point> {
        let center = self.grid_to_screen(coord);
        (0..4)
            .map(|i| {
                center + Vec2::from_angle((PI / 4.0) + i as f32 * (PI / 2.0)) * self.circumradius()
            })
            .collect()
    }

    fn edges(&self, coord: &Self::Coord) -> HashMap<Direction, (Point, Point)> {
        use Direction::*;
        HashMap::from_iter(
            [North, West, South, East]
                .into_iter()
                .zip(vertices_to_edges(self.vertices(coord).as_slice())),
        )
    }

    fn grid_to_screen(&self, coord: &Self::Coord) -> Point {
        self.inradius * Self::conversion_matrix() * Vec2::new(coord.0.x as f32, coord.0.y as f32)
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let grid = Self::conversion_matrix().inverse() * point / self.inradius;
        Coord(IVec2::new(grid.x.round() as i32, grid.y.round() as i32))
    }

    fn screen_rect_to_grid(
        &self,
        min: Point,
        max: Point,
    ) -> Option<impl Iterator<Item = Self::Coord>> {
        if !min.cmple(max).all() {
            return None;
        };
        let min_coord = self.screen_to_grid(min);
        let max_coord = self.screen_to_grid(max);
        Some(GridIterator {
            current_y: min_coord.0.y,
            end_y: max_coord.0.y,
            start_x: min_coord.0.x,
            current_x: min_coord.0.x,
            end_x: max_coord.0.x,
        })
    }
}

//////////////////////////////////////////////////////////////////////////////

struct GridIterator {
    current_y: i32,
    end_y: i32,
    start_x: i32,
    current_x: i32,
    end_x: i32,
}

impl Iterator for GridIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y > self.end_y {
            return None;
        }
        let c = Coord::new(self.current_x, self.current_y);
        self.current_x += 1;
        if self.current_x > self.end_x {
            self.current_x = self.start_x;
            self.current_y += 1;
        }
        Some(c)
    }
}
