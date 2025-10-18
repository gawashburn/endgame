use crate::shape::HashShape;
use crate::utils::{vertices_to_edges, ModuleCoordIter};
use crate::{AllowedCoordIterRange, Color, DirectionType, ModuleCoord, Point};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, IVec2, IVec3, Mat2, Vec2, Vec3, Vec3Swizzles};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::fmt::Display;
use std::ops::Neg;

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Axes {
    Q,
    R,
    S,
}

impl Display for Axes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Axes::*;
        let c = match self {
            Q => 'Q',
            R => 'R',
            S => 'S',
        };
        write!(f, "{}", c)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// For a hexagonal grid, it is possible to move in the same face directions
/// from any coordinate.
const ALLOWED_FACE_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[North, NorthEast, SouthEast, South, SouthWest, NorthWest])
};

/// For a hexagonal grid, it is possible to move in the same vertex directions
/// from any coordinate.
const ALLOWED_VERTEX_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[NorthEast, East, SouthEast, SouthWest, West, NorthWest])
};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// This implementation is based upon the axial coordinate system as described
/// by <https://www.redblobgames.com/grids/hexagons/>.
/// It uses a flat-topped hexagonal grid with even-q offset coordinates.
///
// TODO Add support for pointy top orientation?
// IVec2::x is the same as the axial q and IVec2::y is the axial r.
pub struct Coord(glam::IVec2);

impl Coord {
    /// The three axes of a hexagonal grid.
    pub const AXES: [Axes; 3] = [Axes::Q, Axes::R, Axes::S];

    /// Construct a new `HexGridCoord` from x and y coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        Coord(ivec2(x, y))
    }

    pub fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        let (x, y) = (array_offset.0 as i32, array_offset.1 as i32);
        Coord(ivec2(x, y - (x + (x & 1)) / 2))
    }

    /// Construct a new `HexGridCoord` from an `IVec2`.
    pub const fn from_ivec2(coord: glam::IVec2) -> Self {
        Coord(coord)
    }

    /// Convert the coordinate to an `IVec2`.
    pub const fn to_ivec2(&self) -> glam::IVec2 {
        self.0
    }

    pub const fn to_cubical(&self) -> IVec3 {
        let x = self.0.x;
        let z = self.0.y;
        let y = -x - z;
        IVec3::new(x, y, z)
    }

    pub fn from_cubical(coord: IVec3) -> Self {
        assert_eq!(
            coord.element_sum(),
            0,
            "Cubical coordinates must satisfy x + y + z = 0."
        );

        Coord(ivec2(coord.x, coord.z))
    }

    /// Helper function for rounding floating point hex axial coordinates to
    /// the nearest integral hex axial coordinate.
    fn hex_round(cube: Vec3) -> IVec3 {
        // Convert the axial coordinates to cubical coordinates.
        let x = cube.x;
        let y = cube.y;
        let z = cube.z;
        let mut rx = x.round();
        let mut ry = y.round();
        let mut rz = z.round();
        let x_diff = (rx - x).abs();
        let y_diff = (ry - y).abs();
        let z_diff = (rz - z).abs();
        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry - rz;
        } else if y_diff > z_diff {
            // TODO if we need this in cubical coordinates, it seems
            //   like it should matter for axial coordinates?
            //   However, in practice it seems to be dead code?
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }

        IVec3::new(rx as i32, ry as i32, rz as i32)
    }

    pub fn ring(radius: usize) -> HashShape<Coord> {
        if radius == 0 {
            return HashShape::from([Coord::default()]);
        }

        crate::utils::ring(
            Coord::new(radius as i32, 0),
            Axes::Q,
            Axes::Q,
            &Coord::AXES,
            -1,
        )
    }

    pub fn range(radius: usize) -> HashShape<Coord> {
        // TODO Revise to use a more efficient algorithm.
        //   Implementing the algorithm from
        //   https://www.redblobgames.com/grids/hexagons/#range
        //   does not appear to work as expected?  Potentially
        //   an issue with the use axial versus cubical coordinates?
        let iradius = radius as i32;
        let mut coords = Vec::new();
        for q in -iradius..=iradius {
            for r in -iradius..=iradius {
                for s in -iradius..=iradius {
                    let vec = IVec3::new(q, s, r);
                    if vec.element_sum() == 0 {
                        coords.push(Coord::from_cubical(vec));
                    }
                }
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
    fn add_assign(&mut self, other: &'a Self) {
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

//////////////////////////////////////////////////////////////////////////////////////////////////

impl crate::Coord for Coord {
    type Axes = Axes;

    fn is_origin(&self) -> bool {
        self.0 == IVec2::ZERO
    }

    fn distance(&self, other: &Self) -> usize {
        let diff = self.0 - other.0;
        (diff.x.abs() + diff.y.abs() + (diff.x + diff.y).abs()) as usize / 2
    }

    fn angle_to_direction(&self, dir_type: DirectionType, angle: f32) -> Direction {
        // We can ignore the coordinate, as angle to direction mapping
        // is the same for any coordinate.

        use Direction::*;
        use DirectionType::*;

        // TODO Can this be simplified?

        let norm_angle = angle.rem_euclid(TAU);
        // After normalization, it is expected that the angle will not have
        // a negative sign.
        assert!(norm_angle.is_sign_positive());
        match dir_type {
            Face => {
                // For face directions, we divide the angle by 60 degrees
                // (π/3 radians).
                let hextant = norm_angle / (PI / 3.0);
                if hextant < 1.0 {
                    NorthEast
                } else if hextant < 2.0 {
                    North
                } else if hextant < 3.0 {
                    NorthWest
                } else if hextant < 4.0 {
                    SouthWest
                } else if hextant < 5.0 {
                    South
                } else {
                    SouthEast
                }
            }
            Vertex => {
                // For vertex directions, we divide the angle by 30 degrees
                // (π/6 radians).
                let dodecant = norm_angle / (PI / 6.0);
                if dodecant > 11.0 || dodecant < 1.0 {
                    East
                } else if dodecant < 3.0 {
                    NorthEast
                } else if dodecant < 5.0 {
                    NorthWest
                } else if dodecant < 7.0 {
                    West
                } else if dodecant < 9.0 {
                    SouthWest
                } else {
                    SouthEast
                }
            }
        }
    }

    fn direction_angle(&self, dir_type: DirectionType, dir: Direction) -> Option<f32> {
        use Direction::*;
        use DirectionType::*;

        match dir_type {
            Face => Some(match dir {
                NorthEast => 1.0 * PI / 6.0,
                NorthWest => 5.0 * PI / 6.0,
                SouthWest => 7.0 * PI / 6.0,
                SouthEast => 11.0 * PI / 6.0,
                // North and South correspond to their usual angle.
                North | South => dir.angle(),
                // East and West do not have face directions.
                _ => return None,
            }),
            Vertex => Some(match dir {
                NorthEast => PI / 3.0,
                NorthWest => 2.0 * PI / 3.0,
                SouthWest => 4.0 * PI / 3.0,
                SouthEast => 5.0 * PI / 3.0,
                // East and West correspond to their usual angle.
                East | West => dir.angle(),
                // North and South do not have vertex directions.
                North | South => return None,
            }),
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
    ) -> impl Iterator<Item=Self> {
        ModuleCoordIter {
            coord: *self,
            opt_offset: self.offset_in_direction(dir_type, dir),
            index: 0,
            range,
        }
    }

    fn path_iterator(&self, other: &Self) -> impl Iterator<Item=Self> {
        HexLineIter::new(
            self.to_cubical().as_vec3(),
            other.to_cubical().as_vec3(),
            self.distance(other),
        )
    }

    fn axis_iterator<RB: AllowedCoordIterRange>(
        &self,
        axis: Axes,
        positive: bool,
        range: RB,
    ) -> impl Iterator<Item=Self> {
        use Axes::*;
        use Direction::*;
        use DirectionType::*;
        match (axis, positive) {
            (Q, true) => self.direction_iterator(Face, North, range),
            (Q, false) => self.direction_iterator(Face, South, range),
            (R, true) => self.direction_iterator(Face, NorthEast, range),
            (R, false) => self.direction_iterator(Face, SouthWest, range),
            (S, true) => self.direction_iterator(Face, SouthEast, range),
            (S, false) => self.direction_iterator(Face, NorthWest, range),
        }
    }

    fn allowed_direction(&self, dir_type: DirectionType, dir: Direction) -> bool {
        // We can ignore the coordinate, as the allowed directions
        // are the same from any coordinate.
        use DirectionType::*;
        match dir_type {
            Face => ALLOWED_FACE_DIRECTIONS.contains(dir),
            Vertex => ALLOWED_VERTEX_DIRECTIONS.contains(dir),
        }
    }

    fn allowed_directions(&self, dir_type: DirectionType) -> DirectionSet {
        // We can ignore the coordinate, as the allowed directions
        // are the same from any coordinate.
        use DirectionType::*;
        match dir_type {
            Face => ALLOWED_FACE_DIRECTIONS.clone(),
            Vertex => ALLOWED_VERTEX_DIRECTIONS.clone(),
        }
    }

    fn grid_to_array_offset(&self) -> (isize, isize) {
        let (q, r) = (self.0.x as isize, self.0.y as isize);
        (q, r + (q + (q & 1)) / 2)
    }

    fn to_color(&self) -> Color {
        let (x, y) = self.grid_to_array_offset();

        let num = ((y + x.rem_euclid(2)).rem_euclid(3) + 1) as usize;
        num.try_into().expect("Unexpected fill color index: {num}")
    }

    fn rotate_clockwise(&self) -> Self {
        Coord::from_cubical(self.to_cubical().zxy().neg())
    }

    fn rotate_counterclockwise(&self) -> Self {
        Coord::from_cubical(self.to_cubical().yzx().neg())
    }

    fn reflect(&self, axis: Self::Axes) -> Self {
        use Axes::*;
        let cubical = self.to_cubical();
        let result = match axis {
            Q => cubical.xzy(),
            R => cubical.yxz(),
            S => cubical.zyx(),
        };
        Self::from_cubical(result)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

impl ModuleCoord for Coord {
    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Direction::*;
        use DirectionType::*;
        let offset = match (dir_type, dir) {
            (Face, NorthEast) => ivec2(1, 0),
            (Face, North) => ivec2(0, 1),
            (Face, NorthWest) => ivec2(-1, 1),
            (Face, SouthWest) => ivec2(-1, 0),
            (Face, South) => ivec2(0, -1),
            (Face, SouthEast) => ivec2(1, -1),
            (Vertex, East) => ivec2(2, -1),
            (Vertex, NorthEast) => ivec2(1, 1),
            (Vertex, NorthWest) => ivec2(-1, 2),
            (Vertex, West) => ivec2(-2, 1),
            (Vertex, SouthWest) => ivec2(-1, -1),
            (Vertex, SouthEast) => ivec2(1, -2),
            _ => return None,
        };
        Some(Coord(offset))
    }

    fn offset_on_axis(&self, axis: Self::Axes, positive: bool) -> Self {
        use Axes::*;
        use Direction::*;
        use DirectionType::*;
        let dir = match (axis, positive) {
            (Q, true) => North,
            (Q, false) => South,
            (R, true) => NorthEast,
            (R, false) => SouthWest,
            (S, true) => SouthEast,
            (S, false) => NorthWest,
        };
        self.offset_in_direction(Face, dir)
            .expect("Offset in direction should always succeed")
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct HexLineIter {
    start: Vec3,
    end: Vec3,
    index: usize,
    steps: usize,
}

impl HexLineIter {
    /// Create a new `HexLineIter` from two cubical coordinates.
    pub fn new(start: Vec3, end: Vec3, steps: usize) -> Self {
        HexLineIter {
            start,
            end,
            index: 0,
            steps,
        }
    }
}

impl Iterator for HexLineIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.steps {
            return None;
        }

        let t = self.index as f32 / self.steps as f32;
        self.index += 1;

        // If the start and end are the same, we return the start.
        if self.steps == 0 {
            return Some(Coord::from_cubical(Coord::hex_round(self.start)));
        }

        Some(Coord::from_cubical(Coord::hex_round(
            self.start.lerp(self.end, t),
        )))
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Regular hexagonal grids with cells of specific size.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SizedGrid {
    inradius: f32,
}

impl SizedGrid {
    /// Construct a new `HexSizedGrid` with the given inradius.
    pub fn new(inradius: f32) -> Self {
        SizedGrid { inradius }
    }

    /// The conversion matrix from hex axial coordinates to screen space.
    // TODO Allow this to be constant?
    fn conversion_matrix() -> Mat2 {
        Mat2::from_cols(
            Vec2::from_angle(PI / 6.0f32) * 3.0f32.sqrt(),
            Vec2::from_angle(PI / 2.0f32) * 3.0f32.sqrt(),
        )
    }
}

impl crate::SizedGrid for SizedGrid {
    type Coord = Coord;

    fn inradius(&self) -> f32 {
        self.inradius
    }

    fn circumradius(&self) -> f32 {
        (2.0 * self.inradius) / 3.0f32.sqrt()
    }

    fn edge_length(&self) -> f32 {
        // For a regular hexagonal grid, the edge length is equal to the circumradius.
        self.circumradius()
    }

    fn vertices(&self, coord: &Self::Coord) -> Vec<Point> {
        let center = self.grid_to_screen(coord);
        (0..6)
            .map(|i| center + Vec2::from_angle(i as f32 * PI / 3.0) * self.circumradius())
            .collect()
    }

    fn edges(&self, coord: &Self::Coord) -> HashMap<Direction, (Point, Point)> {
        use Direction::*;
        HashMap::from_iter(
            [NorthEast, North, NorthWest, SouthWest, South, SouthEast]
                .into_iter()
                .zip(vertices_to_edges(self.vertices(coord).as_slice())),
        )
    }

    fn grid_to_screen(&self, coord: &Self::Coord) -> Point {
        self.circumradius() * Self::conversion_matrix() * coord.0.as_vec2()
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let grid = Self::conversion_matrix().inverse() * point / self.circumradius();
        Coord::from_cubical(Coord::hex_round(Vec3::new(
            grid.x,
            -grid.x - grid.y,
            grid.y,
        )))
    }

    fn screen_rect_to_grid(
        &self,
        min: Point,
        max: Point,
    ) -> Option<impl Iterator<Item=Self::Coord>> {
        if !min.cmple(max).all() {
            return None;
        };

        let mut min_coord = self.screen_to_grid(min);
        let mut max_coord = self.screen_to_grid(max);
        // Expand to ensure full coverage of the rectangle.
        min_coord = <Coord as crate::Coord>::move_in_direction(
            &min_coord,
            DirectionType::Face,
            Direction::SouthWest,
        )
            .expect("Moving SouthEast should always be possible for a hexagonal grid.");
        max_coord = <Coord as crate::Coord>::move_in_direction(
            &max_coord,
            DirectionType::Face,
            Direction::NorthEast,
        )
            .expect("Moving NorthWest should always be possible for a hexagonal grid.");

        Some(GridIterator {
            min: min,
            max: max,
            row_coord: min_coord,
            current_coord: min_coord,
            end_r: max_coord.0.y + ((max_coord.0.x - min_coord.0.x) / 2) + 1,
            row_index: 0,
            row_length: (max_coord.0.x - min_coord.0.x + 1) as usize,
            sized_grid: self.clone(),
        })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

struct GridIterator {
    min: Vec2,
    max: Vec2,
    row_coord: Coord,
    current_coord: Coord,
    end_r: i32,
    row_index: usize,
    row_length: usize,
    sized_grid: SizedGrid,
}

impl Iterator for GridIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have advanced past the maximum r value, we are done.
            if self.row_coord.0.y > self.end_r {
                return None;
            }

            let c = self.current_coord;

            // Advance to the next coordinate.
            let dir = if self.row_index % 2 == 0 {
                Direction::SouthEast
            } else {
                Direction::NorthEast
            };
            self.current_coord = <Coord as crate::Coord>::move_in_direction(
                &self.current_coord,
                DirectionType::Face,
                dir,
            )
                .expect("Direction should be valid");
            self.row_index += 1;

            if self.row_index == self.row_length {
                self.row_coord = <Coord as crate::Coord>::move_in_direction(
                    &self.row_coord,
                    DirectionType::Face,
                    Direction::North,
                )
                    .expect("Direction should be valid");
                self.row_index = 0;
                self.current_coord = self.row_coord;
            }

            // Verify that the coordinate intersects with the rectangle.
            if <SizedGrid as crate::SizedGrid>::coord_intersects_rect(
                &self.sized_grid,
                &c,
                self.min,
                self.max,
            ) {
                return Some(c);
            }
        }
    }
}
