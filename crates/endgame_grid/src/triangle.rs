use crate::{Coord, DirectionType, Point, SizedGrid};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, ivec3, IVec2, IVec3, Vec2};
use std::f32::consts::PI;
use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl std::ops::BitAnd for TrianglePoint {
    type Output = TrianglePoint;

    fn bitand(self, other: Self) -> Self::Output {
        use TrianglePoint::*;
        match (self, other) {
            (Up, _) => other,
            (_, Up) => self,
            _ => self.opposite(),
        }
    }
}

impl std::ops::BitAndAssign for TrianglePoint {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
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
/// Visually I find the coordinate representation where base upon trapezoid
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
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct TriangleCoord(IVec2, TrianglePoint);

/// Allowed movement `Direction`s for a triangular grid depends on the
/// triangle's orientation.  This a `DirectionSet` for upward facing
/// triangles.
///
/// There is no need for separate vertex directions as they just correspond to
/// the triangle point being inverted.
const ALLOWED_DIRECTIONS_UP: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[NorthEast, South, NorthWest])
};

/// Allowed movement `Direction`s for a triangular grid depends on the
/// triangle's orientation.  This a `DirectionSet` for downward facing
/// triangles.
///
/// There is need for separate vertex directions as they just correspond to
/// the triangle point being inverted.
// TODO Simplify as this is just the negation of the `Direction`s in
// `ALLOWED_DIRECTIONS_UP`.
const ALLOWED_DIRECTIONS_DOWN: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[North, SouthEast, SouthWest])
};

impl TriangleCoord {
    /// Does this `TriangleGridCoord` represent an upward facing triangle?
    pub fn is_up(&self) -> bool {
        self.1 == TrianglePoint::Up
    }

    /// Construct a new `TriangleGridCoord` from x and y coordinates and a
    /// `TrianglePoint` indicating which direction the triangle is pointing.
    pub const fn new(x: i32, y: i32, point: TrianglePoint) -> Self {
        TriangleCoord(ivec2(x, y), point)
    }

    /// Construct a new `TriangleGridCoord` from an `IVec2` coordinate and a
    /// `TrianglePoint` indicating which direction the triangle is pointing.
    pub const fn from_ivec2(coord: IVec2, point: TrianglePoint) -> Self {
        TriangleCoord(coord, point)
    }

    /// Internal helper to convert a cubical coordinate into a `TriangleGridCoord`.
    const fn from_cubical(coord: IVec3) -> Self {
        use TrianglePoint::*;

        let sum = coord.x + coord.y + coord.z;
        // Check that the coordinate is valid.
        assert!(sum == 1 || sum == 2);

        let up = sum == 2;
        let z_offset = if up { 2 } else { 1 };
        let z = coord.z;
        let x = z_offset - coord.y - z;
        let y = z_offset - coord.x - z;
        TriangleCoord(IVec2::new(x, y), if up { Up } else { Down })
    }

    /// Internal helper to convert a `TriangleGridCoord` into the cube
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

    /// Convert the coordinate to an `IVec2` and a `TrianglePoint`.
    pub const fn to_ivec2(&self) -> (IVec2, TrianglePoint) {
        (self.0, self.1)
    }
}

impl Default for TriangleCoord {
    fn default() -> Self {
        TriangleCoord(ivec2(0, 0), TrianglePoint::Up)
    }
}

impl Display for TriangleCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.0.x, self.0.y, self.1)
    }
}

impl std::ops::Neg for TriangleCoord {
    type Output = Self;

    fn neg(self) -> Self {
        TriangleCoord(-self.0, self.1)
    }
}

impl std::ops::Add for TriangleCoord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        TriangleCoord(self.0 + other.0, self.1 & other.1)
    }
}

impl std::ops::Sub for TriangleCoord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        TriangleCoord(self.0 - other.0, self.1 & other.1)
    }
}

impl std::ops::AddAssign for TriangleCoord {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 &= other.1;
    }
}

impl<'a> std::ops::AddAssign<&'a TriangleCoord> for TriangleCoord {
    fn add_assign(&mut self, other: &Self) {
        self.0 += other.0;
        self.1 &= other.1;
    }
}

impl std::ops::SubAssign for TriangleCoord {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
        self.1 &= other.1;
    }
}

impl<'a> std::ops::SubAssign<&'a TriangleCoord> for TriangleCoord {
    fn sub_assign(&mut self, other: &Self) {
        self.0 -= other.0;
        self.1 &= other.1;
    }
}

impl Coord for TriangleCoord {
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

    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Direction::*;
        use DirectionType::*;
        use TrianglePoint::*;

        let offset = match dir_type {
            Face => match self.1 {
                Up => match dir {
                    NorthEast => ivec2(0, 0),
                    South => ivec2(0, -1),
                    NorthWest => ivec2(-1, 0),
                    _ => return None,
                },
                Down => match dir {
                    North => ivec2(0, 1),
                    SouthEast => ivec2(1, 0),
                    SouthWest => ivec2(0, 0),
                    _ => return None,
                },
            },
            Vertex => match self.1 {
                Up => match dir {
                    North => ivec2(-1, 1),
                    SouthEast => ivec2(1, -1),
                    SouthWest => ivec2(-1, -1),
                    _ => return None,
                },
                Down => match dir {
                    South => ivec2(1, -1),
                    NorthWest => ivec2(-1, 1),
                    NorthEast => ivec2(1, 1),
                    _ => return None,
                },
            },
        };

        Some(TriangleCoord(offset, Down))
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
            Up => ALLOWED_DIRECTIONS_UP,
            Down => ALLOWED_DIRECTIONS_DOWN,
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

    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        use TrianglePoint::*;
        let remainder = array_offset.0.rem_euclid(2) as i32;
        let point = if remainder == 0 { Up } else { Down };
        TriangleCoord::new(
            (array_offset.0 as i32 - remainder) / 2,
            array_offset.1 as i32,
            point,
        )
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Regular triangular grids of a specific size.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct TriangleSizedGrid {
    inradius: f32,
}

impl TriangleSizedGrid {
    /// The basis vector for the "A" lane of the triangle grid.
    fn a_basis() -> Vec2 {
        Vec2::from_angle(11.0 * PI / 6.0)
    }

    /// The basis vector for the "B" lane of the triangle grid.
    fn b_basis() -> Vec2 {
        Vec2::from_angle(PI / 2.0)
    }

    /// The basis vector for the "C" lane of the triangle grid.
    fn c_basis() -> Vec2 {
        Vec2::from_angle(7.0 * PI / 6.0)
    }
}

impl SizedGrid for TriangleSizedGrid {
    type Coord = TriangleCoord;

    fn new(inradius: f32) -> Self {
        TriangleSizedGrid { inradius }
    }

    fn inradius(&self) -> f32 {
        self.inradius
    }

    fn circumradius(&self) -> f32 {
        2.0 * self.inradius
    }

    fn edge_length(&self) -> f32 {
        6.0 * self.inradius / 3.0f32.sqrt()
    }

    fn vertices(&self, coord: Self::Coord) -> Vec<Point> {
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

    fn grid_to_screen(&self, coord: Self::Coord) -> Point {
        let cubical_coord = coord.to_cubical();
        // Offset so that (0,0,∆) is at (0,0)
        let offset_coord = cubical_coord - IVec3::new(0, 0, 2);

        // Compute the contributions of different basis vectors.
        let a_component = TriangleSizedGrid::a_basis() * (offset_coord.x as f32);
        let b_component = TriangleSizedGrid::b_basis() * (offset_coord.y as f32);
        let c_component = TriangleSizedGrid::c_basis() * (offset_coord.z as f32);

        // Combine and scale by the circumradius.
        (a_component + b_component + c_component) * self.circumradius()
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let height = self.inradius + self.circumradius();

        // Offset so that (0,0,∆) is at (0,0)
        let offset_point = point + Vec2::new(-self.edge_length(), -self.circumradius());
        // Use the dot product to determine the relative contributions of
        // each of the basis vectors.
        let a_component = TriangleSizedGrid::a_basis().dot(offset_point);
        let b_component = TriangleSizedGrid::b_basis().dot(offset_point);
        let c_component = TriangleSizedGrid::c_basis().dot(offset_point);

        TriangleCoord::from_cubical(ivec3(
            (a_component / height).ceil() as i32,
            (b_component / height).ceil() as i32,
            (c_component / height).ceil() as i32,
        ))
    }
}
