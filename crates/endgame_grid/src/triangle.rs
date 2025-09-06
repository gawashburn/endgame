use crate::shape::HashShape;
use crate::utils::vertices_to_edges;
use crate::{AllowedCoordIterRange, Color, DirectionType, Point, Shape};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, ivec3, IVec2, IVec3, Vec2, Vec3Swizzles};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::Display;
//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Axes {
    A,
    B,
    C,
}

impl Display for Axes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Axes::*;
        let c = match self {
            A => 'A',
            B => 'B',
            C => 'C',
        };
        write!(f, "{}", c)
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Specify which direction the triangle is pointing.
pub enum TrianglePoint {
    /// Triangle points "upwards" along the y-axis.
    Up,
    /// Triangle points "downwards" along the y-axis.
    Down,
}

impl TrianglePoint {
    /// Obtain the `TrianglePoint` facing the opposite direction.
    const fn opposite(&self) -> TrianglePoint {
        use TrianglePoint::*;
        match self {
            Up => Down,
            Down => Up,
        }
    }
}

impl std::ops::Not for TrianglePoint {
    type Output = TrianglePoint;

    fn not(self) -> Self::Output {
        self.opposite()
    }
}

impl Display for TrianglePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TrianglePoint::*;
        match self {
            Up => write!(f, "∆"),
            Down => write!(f, "∇"),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// This implementation of a triangular grid was influenced significantly by
///
/// <https://web.archive.org/web/20250419151513/https://www.boristhebrave.com/2021/05/23/triangle-grids/>
///
/// Visually I find the coordinate representation based upon trapezoidal
/// coordinates, with a flag to indicate which direction the triangle points,
/// to be the most intuitive.  However, I spent a considerable amount of time
/// trying to work out the math for translating back and forth the between
/// that coordinate system and screen space with only partial success.
///
/// Boris's recommendation to think of the triangle grid as a coordinate
/// system of three "lanes" analogous to the cubical coordinate system for
/// hexagonal grids was extremely helpful.  However, while unit testing,
/// I found that I could not directly make use fo his pseudocode.  So
/// some revisions, particular for clarity have been made.
///
/// I also spent some time investigating the Ω coordinate system described
/// in the paper "Vector Arithmetic in the Triangular Grid" by
/// Khaled Abuhmaidan, Monther Aldwairi, and Benedek Nagy.  However,
/// but it did not seem to provide the desired algebraic properties.
///
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coord(IVec2, TrianglePoint);

// Use lazy_static so we can compute `ALLOWED_DIRECTIONS_DOWN`.
lazy_static::lazy_static! {

/// Allowed movement `Direction`s for a triangular grid depends on the
/// triangle's orientation.  This a `DirectionSet` for upward facing
/// triangles.
///
/// There is no need for separate vertex directions as they just correspond to
/// the triangle point being inverted.
static ref ALLOWED_DIRECTIONS_UP: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[NorthEast, South, NorthWest])
};

/// Allowed movement `Direction`s for a triangular grid depends on the
/// triangle's orientation.  This a `DirectionSet` for downward facing
/// triangles.
///
/// There is need for separate vertex directions as they just correspond to
/// the triangle point being inverted.
static ref ALLOWED_DIRECTIONS_DOWN: DirectionSet = {
    ALLOWED_DIRECTIONS_UP.iter().map(|d| !d).collect::<DirectionSet>()
};

}

impl Coord {
    /// The three axes of movement for triangular grids.
    pub const AXES: [Axes; 3] = [Axes::A, Axes::B, Axes::C];

    /// Does this `Coord` represent an upward facing triangle?
    pub fn is_up(&self) -> bool {
        self.1 == TrianglePoint::Up
    }

    /// Construct a new `Coord` from x and y coordinates and a
    /// `TrianglePoint` indicating which direction the triangle is pointing.
    pub const fn new(x: i32, y: i32, point: TrianglePoint) -> Self {
        Coord(ivec2(x, y), point)
    }

    /// Construct a new `Coord` from an array offset.
    pub const fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        use TrianglePoint::*;
        let remainder = array_offset.0.rem_euclid(2) as i32;
        let point = if remainder == 0 { Up } else { Down };
        Coord::new(
            (array_offset.0 as i32 - remainder) / 2,
            array_offset.1 as i32,
            point,
        )
    }

    /// Construct a new `Coord` from an `IVec2` coordinate and a
    /// `TrianglePoint` indicating which direction the triangle is pointing.
    pub const fn from_ivec2(coord: IVec2, point: TrianglePoint) -> Self {
        Coord(coord, point)
    }

    /// Convert the coordinate to an `IVec2` and a `TrianglePoint`.
    pub const fn to_ivec2(&self) -> (IVec2, TrianglePoint) {
        (self.0, self.1)
    }

    /// Internal helper to convert a cubical coordinate into a `Coord`.
    fn from_cubical(coord: IVec3) -> Self {
        use TrianglePoint::*;

        let sum = coord.element_sum();
        // Check that the coordinate is valid.
        assert!(
            sum == 1 || sum == 2,
            "Invalid cubical coordinate {:?}, elements sum to {}",
            coord,
            sum
        );

        let up = sum == 2;
        let z_offset = if up { 2 } else { 1 };
        let z = coord.z;
        let x = z_offset - coord.y - z;
        let y = z_offset - coord.x - z;
        Coord(IVec2::new(x, y), if up { Up } else { Down })
    }

    /// Internal helper to convert a `Coord` into the cube
    /// coordinate system.
    const fn to_cubical(&self) -> IVec3 {
        use TrianglePoint::*;

        let z_offset = match self.1 {
            Up => 2,
            Down => 1,
        };

        let x = self.0.x;
        let y = self.0.y;

        IVec3::new(x, y, z_offset - x - y)
    }

    pub fn ring(radius: usize) -> HashShape<Coord> {
        if radius == 0 {
            return HashShape::from([Coord::default()]);
        } else if radius == 1 {
            // While we could use the algorithm below, it would retrace
            // the origin multiple times.  It is more pleasing to have
            // rings for each radius to be disjoint.
            return HashShape::from([
                Coord::new(0, 0, TrianglePoint::Down),
                Coord::new(0, -1, TrianglePoint::Down),
                Coord::new(-1, 0, TrianglePoint::Down),
            ]);
        }

        crate::utils::ring(
            Coord::new(
                (radius - 1) as i32,
                (radius - 1) as i32,
                TrianglePoint::Down,
            ),
            Axes::B,
            Axes::A,
            &Coord::AXES,
            1,
        )
    }

    pub fn range(radius: usize) -> HashShape<Coord> {
        // TODO Find a more efficient algorithm.
        let mut coords: Vec<Coord> = Vec::new();
        for r in 0..=radius {
            coords.append(&mut Coord::ring(r).iter().cloned().collect());
        }
        HashShape::from_iter(coords.into_iter())
    }
}

impl Default for Coord {
    fn default() -> Self {
        Coord(ivec2(0, 0), TrianglePoint::Up)
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.0.x, self.0.y, self.1)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl crate::Coord for Coord {
    type Axes = Axes;

    fn is_origin(&self) -> bool {
        *self == Coord(IVec2::ZERO, TrianglePoint::Up)
    }

    fn distance(&self, other: &Self) -> usize {
        (other.to_cubical() - self.to_cubical()).abs().element_sum() as usize
    }

    fn angle_to_direction(&self, dir_type: DirectionType, angle: f32) -> Direction {
        use Direction::*;
        use TrianglePoint::*;
        // We can ignore the coordinate, as angle to direction mapping
        // is the same for any coordinate.
        let norm_angle = angle.rem_euclid(2.0 * PI);
        // After normalization, it is expected that the angle will not have
        // a negative sign.
        assert!(norm_angle.is_sign_positive());
        let dodecant = norm_angle / (PI / 6.0);

        // For the vertex directions, we can simply use the angles for the
        // triangle facing the opposite direction.
        let point = if dir_type == DirectionType::Vertex {
            !self.1 // Invert the triangle point for vertex directions.
        } else {
            self.1
        };

        match point {
            Up => {
                if dodecant >= 11.0 || dodecant < 3.0 {
                    NorthEast
                } else if dodecant < 7.0 {
                    NorthWest
                } else {
                    assert!(dodecant < 11.0);
                    South
                }
            }
            Down => {
                if dodecant >= 9.0 || dodecant < 1.0 {
                    SouthEast
                } else if dodecant < 4.0 {
                    North
                } else {
                    assert!(dodecant < 9.0);
                    SouthWest
                }
            }
        }
    }

    fn direction_angle(&self, dir_type: DirectionType, dir: Direction) -> Option<f32> {
        use Direction::*;
        use TrianglePoint::*;

        // For the vertex directions, we can simply use the angles for the
        // triangle facing the opposite direction.
        let point = if dir_type == DirectionType::Vertex {
            !self.1 // Invert the triangle point for vertex directions.
        } else {
            self.1
        };

        Some(match point {
            Up => match dir {
                NorthEast => PI / 6.0,
                NorthWest => 5.0 * PI / 6.0,
                South => dir.angle(),
                _ => return None,
            },
            Down => match dir {
                SouthWest => 7.0 * PI / 6.0,
                SouthEast => 11.0 * PI / 6.0,
                North => dir.angle(),
                _ => return None,
            },
        })
    }

    fn move_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Direction::*;
        use DirectionType::*;
        use TrianglePoint::*;

        let offset = match (dir_type, self.1, dir) {
            (Face, Up, NorthEast) => ivec2(0, 0),
            (Face, Up, South) => ivec2(0, -1),
            (Face, Up, NorthWest) => ivec2(-1, 0),
            (Face, Down, North) => ivec2(0, 1),
            (Face, Down, SouthEast) => ivec2(1, 0),
            (Face, Down, SouthWest) => ivec2(0, 0),
            (Vertex, Up, North) => ivec2(-1, 1),
            (Vertex, Up, SouthEast) => ivec2(1, -1),
            (Vertex, Up, SouthWest) => ivec2(-1, -1),
            (Vertex, Down, South) => ivec2(1, -1),
            (Vertex, Down, NorthWest) => ivec2(-1, 1),
            (Vertex, Down, NorthEast) => ivec2(1, 1),
            _ => return None,
        };

        Some(Coord(self.0 + offset, !self.1))
    }

    fn move_on_axis(&self, axis: Self::Axes, positive: bool) -> Self {
        use Axes::*;
        use TrianglePoint::*;
        let offset = match (self.1, axis, positive) {
            (Up, A, true) => ivec2(0, 0),
            (Up, A, false) => ivec2(0, -1),
            (Up, B, true) => ivec2(0, 0),
            (Up, B, false) => ivec2(-1, 0),
            (Up, C, true) => ivec2(-1, 0),
            (Up, C, false) => ivec2(0, -1),
            (Down, A, true) => ivec2(0, 1),
            (Down, A, false) => ivec2(0, 0),
            (Down, B, true) => ivec2(1, 0),
            (Down, B, false) => ivec2(0, 0),
            (Down, C, true) => ivec2(0, 1),
            (Down, C, false) => ivec2(1, 0),
        };

        Coord(self.0 + offset, !self.1)
    }

    fn direction_iterator<RB: AllowedCoordIterRange>(
        &self,
        dir_type: DirectionType,
        dir: Direction,
        range: RB,
    ) -> impl Iterator<Item = Self> {
        DirectionIter {
            current: self.clone(),
            dir_type,
            dir,
            index: 0,
            range,
        }
    }

    fn path_iterator(&self, other: &Self) -> impl Iterator<Item = Self> {
        TrianglePathIter::new(self, other)
    }

    fn axis_iterator<RB: AllowedCoordIterRange>(
        &self,
        axis: Self::Axes,
        positive: bool,
        range: RB,
    ) -> impl Iterator<Item = Self> {
        TriangleAxisIter {
            current: *self,
            axis,
            positive,
            index: 0,
            range,
        }
    }

    fn allowed_direction(&self, dir_type: DirectionType, dir: Direction) -> bool {
        self.allowed_directions(dir_type).contains(dir)
    }

    fn allowed_directions(&self, dir_type: DirectionType) -> DirectionSet {
        use TrianglePoint::*;

        let mut point = self.1;
        if dir_type == DirectionType::Vertex {
            point = !point; // Invert the triangle point for vertex directions.
        }

        match point {
            Up => *ALLOWED_DIRECTIONS_UP,
            Down => *ALLOWED_DIRECTIONS_DOWN,
        }
    }

    fn grid_to_array_offset(&self) -> (isize, isize) {
        use TrianglePoint::*;

        let x_offset = match self.1 {
            Up => 0,
            Down => 1,
        };
        let y = self.0.y as isize;
        (self.0.x as isize * 2 + x_offset, y)
    }

    fn to_color(&self) -> Color {
        let (x, y) = self.grid_to_array_offset();
        let num = ((x + (2 * y)).rem_euclid(2) + 1) as usize;
        num.try_into().expect("Unexpected fill color index: {num}")
    }

    fn rotate_clockwise(&self) -> Self {
        let cubical = self.to_cubical() - IVec3::new(0, 0, 2);
        let result = IVec3::ONE - (IVec3::ONE - cubical.yzx()).yzx();
        Coord::from_cubical(result + IVec3::new(0, 0, 2))
    }

    fn rotate_counterclockwise(&self) -> Self {
        let cubical = self.to_cubical() - IVec3::new(0, 0, 2);
        let result = IVec3::ONE - (IVec3::ONE - cubical.zxy()).zxy();
        Coord::from_cubical(result + IVec3::new(0, 0, 2))
    }

    fn reflect(&self, axis: Self::Axes) -> Self {
        use Axes::*;
        // TODO Bake offset into the to_cubical/from_cubical functions?
        let cubical = self.to_cubical() - IVec3::new(0, 0, 2);
        let result = match axis {
            A => cubical.xzy(),
            B => cubical.zyx(),
            C => cubical.yxz(),
        };
        Coord::from_cubical(result + IVec3::new(0, 0, 2))
    }
}

//////////////////////////////////////////////////////////////////////////////

pub struct DirectionIter<RB: AllowedCoordIterRange> {
    pub current: Coord,
    pub dir_type: DirectionType,
    pub dir: Direction,
    pub index: usize,
    pub range: RB,
}

impl<RB: AllowedCoordIterRange> Iterator for DirectionIter<RB> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        // If the direction is not allowed, or the range is complete, we are
        // done iterating.
        if !<Coord as crate::Coord>::allowed_direction(&self.current, self.dir_type, self.dir)
            || self.range.complete(self.index)
        {
            return None;
        }

        let result = self.current.clone();
        self.index += 1;
        self.current =
            <Coord as crate::Coord>::move_in_direction(&self.current, self.dir_type, self.dir)
                .expect(
                    format!(
                        "Direction should have been validated before calling advance {} {}",
                        self.dir_type, self.dir
                    )
                    .as_str(),
                );
        self.dir_type = !self.dir_type;
        Some(result)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Given that our triangular coordinate system cannot be treated as a linear
/// space, we cannot use linear interpolation between two coordinates in the
/// same way we can for square or hexagonal grids.  So for now the
/// cleanest algorithm I could find was to convert to screen coordinates
/// for a unit sized grid, and then interpolate between the two
/// coordinates in screen space.  At each step, we find the adjacent
/// coordinate that minimizes the error with the screen space interpolation.
///
/// There is still some room for improvement, as this implementation
/// can produce correct paths that are not as aesthetically pleasing as
/// would be ideal. All known instances arise in the case where there
/// are multiple possible paths which yield an equivalent error.  For
/// example, consider the path from (0,1,∆) to (1,4,∆).
///
/// When choosing the step after (0,2,∇) the algorithm has the choice of
/// moving to either (0,3,∆) or (1,2,∆).  Visually, (0,3,∆) would appear
/// to be the better choice.  But as implemented the algorithm will choose
/// (1,2,∆) because it comes up first in the list of allowed directions.
/// Re-ordering the allowed directions would resolve this specific case,
/// but there would simply be symmetric cases where the new bias would
/// still produce visual artifacts.
///
/// The path from (0,-1,∆) to (1,5,∆) is also illustrative.
#[derive(Debug, Clone)]
pub struct TrianglePathIter {
    sized_grid: SizedGrid,
    start_frac: Vec2,
    end_frac: Vec2,
    current: Coord,
    index: usize,
    steps: usize,
}

impl TrianglePathIter {
    pub fn new(start: &Coord, end: &Coord) -> Self {
        // Use a unit sized grid for the Cartesian coordinates.
        let sized_grid = SizedGrid::new(1.0);

        TrianglePathIter {
            sized_grid,
            start_frac: <SizedGrid as crate::SizedGrid>::grid_to_screen(&sized_grid, start),
            end_frac: <SizedGrid as crate::SizedGrid>::grid_to_screen(&sized_grid, end),
            current: *start,
            index: 0,
            steps: <Coord as crate::Coord>::distance(start, end),
        }
    }
}

impl Iterator for TrianglePathIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.steps {
            return None;
        } else if self.steps == 0 {
            self.index += 1;
            return Some(self.current);
        }
        // We'll return the current coordinate.
        let c = self.current;
        // Now find the next coordinate.
        let t = (self.index + 1) as f32 / self.steps as f32;
        let frac_target_coord = self.start_frac.lerp(self.end_frac, t);
        // Compute a vector of possible coordinates along with the error.
        let err = <Coord as crate::Coord>::allowed_directions(&self.current, DirectionType::Face)
            .iter()
            .map(|d| {
                let new_coord = <Coord as crate::Coord>::move_in_direction(
                    &self.current,
                    DirectionType::Face,
                    d,
                )
                .expect("Direction should be valid");
                let new_frac =
                    <SizedGrid as crate::SizedGrid>::grid_to_screen(&self.sized_grid, &new_coord);
                (new_coord, (frac_target_coord - new_frac).length())
            })
            .collect::<Vec<(Coord, f32)>>();
        assert!(err.len() > 0, "There should be at least one coordinate");
        // Find the coordinate with the minimum error.
        let (min_coord, _) = err
            .iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .expect("There should be at least one coordinate");
        self.current = *min_coord;
        self.index += 1;

        Some(c)
    }
}

//////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct TriangleAxisIter<RB: AllowedCoordIterRange> {
    pub current: Coord,
    pub axis: Axes,
    pub positive: bool,
    pub index: usize,
    pub range: RB,
}

impl<RB: AllowedCoordIterRange> Iterator for TriangleAxisIter<RB> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range.complete(self.index) {
            // The range is complete, so the iterator is empty.
            return None;
        }

        let result = self.current;
        self.current =
            <Coord as crate::Coord>::move_on_axis(&self.current, self.axis, self.positive);
        self.index += 1;
        Some(result)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Regular triangular grids of a specific size.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SizedGrid {
    inradius: f32,
}

impl SizedGrid {
    pub fn new(inradius: f32) -> Self {
        SizedGrid { inradius }
    }

    /// The basis vector for the "A" lanes of the triangle grid.
    fn a_basis() -> Vec2 {
        Vec2::from_angle(11.0 * PI / 6.0)
    }

    /// The basis vector for the "B" lanes of the triangle grid.
    fn b_basis() -> Vec2 {
        Vec2::from_angle(PI / 2.0)
    }

    /// The basis vector for the "C" lanes of the triangle grid.
    fn c_basis() -> Vec2 {
        Vec2::from_angle(7.0 * PI / 6.0)
    }
}

impl crate::SizedGrid for SizedGrid {
    type Coord = Coord;

    fn inradius(&self) -> f32 {
        self.inradius
    }

    fn circumradius(&self) -> f32 {
        2.0 * self.inradius
    }

    fn edge_length(&self) -> f32 {
        6.0 * self.inradius / 3.0f32.sqrt()
    }

    fn vertices(&self, coord: &Self::Coord) -> Vec<Point> {
        use TrianglePoint::*;
        let start_angle = match coord.1 {
            Up => PI / 2.0,
            Down => PI / 6.0,
        };

        let center = self.grid_to_screen(coord);

        (0..3)
            .map(|i| {
                center
                    + Vec2::from_angle(start_angle + i as f32 * (2.0 * PI / 3.0))
                        * self.circumradius()
            })
            .collect()
    }

    fn edges(&self, coord: &Self::Coord) -> HashMap<Direction, (Point, Point)> {
        use Direction::*;
        let dirs = if coord.1 == TrianglePoint::Up {
            [NorthWest, South, NorthEast]
        } else {
            [North, SouthWest, SouthEast]
        };
        HashMap::from_iter(
            dirs.into_iter()
                .zip(vertices_to_edges(self.vertices(coord).as_slice())),
        )
    }

    fn grid_to_screen(&self, coord: &Self::Coord) -> Point {
        let cubical_coord = coord.to_cubical();
        // Offset so that (0,0,∆) is at (0,0)
        let offset_coord = cubical_coord - IVec3::new(0, 0, 2);

        // Compute the contributions of different basis vectors.
        let a_component = SizedGrid::a_basis() * (offset_coord.x as f32);
        let b_component = SizedGrid::b_basis() * (offset_coord.y as f32);
        let c_component = SizedGrid::c_basis() * (offset_coord.z as f32);

        // Combine and scale by the circumradius.
        (a_component + b_component + c_component) * self.circumradius()
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let height = self.inradius + self.circumradius();

        // Offset so that (0,0,∆) is at (0,0)
        let offset_point = point + Vec2::new(-self.edge_length(), -self.circumradius());
        // Use the dot product to determine the relative contributions of
        // each of the basis vectors.
        let a_component = SizedGrid::a_basis().dot(offset_point);
        let b_component = SizedGrid::b_basis().dot(offset_point);
        let c_component = SizedGrid::c_basis().dot(offset_point);

        Coord::from_cubical(ivec3(
            (a_component / height).ceil() as i32,
            (b_component / height).ceil() as i32,
            (c_component / height).ceil() as i32,
        ))
    }

    fn screen_rect_to_grid(
        &self,
        min: Point,
        max: Point,
    ) -> Option<impl Iterator<Item = Self::Coord>> {
        if !min.cmple(max).all() {
            return None;
        };
        let mut min_coord = self.screen_to_grid(min);
        let mut max_coord = self.screen_to_grid(max);
        // Expand to ensure full coverage of the rectangle.
        min_coord = <Coord as crate::Coord>::move_on_axis(&min_coord, Axes::B, false);
        max_coord = <Coord as crate::Coord>::move_on_axis(&max_coord, Axes::B, true);

        Some(GridIterator {
            min,
            max,
            row_coord: min_coord,
            current_coord: min_coord,
            end_y: max_coord.0.y,
            row_index: 0,
            row_length: ((max_coord.0.x - min_coord.0.x) * 2 + (max_coord.0.y - min_coord.0.y) + 2)
                as usize,
            sized_grid: self.clone(),
        })
    }
}

//////////////////////////////////////////////////////////////////////////////

struct GridIterator {
    min: Point,
    max: Point,
    row_coord: Coord,
    current_coord: Coord,
    end_y: i32,
    row_index: usize,
    row_length: usize,
    sized_grid: SizedGrid,
}

impl Iterator for GridIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.row_coord.0.y > self.end_y {
                return None;
            }

            let c = self.current_coord;

            // Advance to the next coordinate.
            self.row_index += 1;
            self.current_coord =
                <Coord as crate::Coord>::move_on_axis(&self.current_coord, Axes::B, true);

            if self.row_index == self.row_length {
                self.row_coord = if self.row_coord.is_up() {
                    <Coord as crate::Coord>::move_in_direction(
                        &self.row_coord,
                        DirectionType::Vertex,
                        Direction::North,
                    )
                    .expect("Direction should be valid")
                } else {
                    <Coord as crate::Coord>::move_in_direction(
                        &self.row_coord,
                        DirectionType::Face,
                        Direction::North,
                    )
                    .expect("Direction should be valid")
                };
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
