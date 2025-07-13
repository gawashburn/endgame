use crate::{Coord, DirectionType, Point, SizedGrid};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, Mat2, Vec2};
use std::f32::consts::PI;
use std::fmt::Display;

/// For a hexagonal grid, it is possible to move in the same face directions
/// from any coordinate.
const ALLOWED_FACE_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[North, NorthEast, SouthEast, South, SouthWest, NorthWest])
};

/// For a hexagonal grid, it is possible to move in the same vertext directions
/// from any coordinate.
const ALLOWED_VERTEX_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[NorthEast, East, SouthEast, SouthWest, West, NorthWest])
};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
/// This implementation is based upon the axial coordinate system as described
/// by <https://www.redblobgames.com/grids/hexagons/>.
/// It uses a flat-topped hexagonal grid with even-q offset coordinates.
///
// TODO Add support for pointy top orientation?
// IVec2::x is the same as the axial q and IVec2::y is the axial r.
pub struct HexCoord(glam::IVec2);

impl HexCoord {
    /// Construct a new `HexGridCoord` from x and y coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        HexCoord(ivec2(x, y))
    }

    /// Construct a new `HexGridCoord` from an `IVec2`.
    pub const fn from_ivec2(coord: glam::IVec2) -> Self {
        HexCoord(coord)
    }

    /// Convert the coordinate to an `IVec2`.
    pub const fn to_ivec2(&self) -> glam::IVec2 {
        self.0
    }
}

impl Default for HexCoord {
    fn default() -> Self {
        HexCoord(ivec2(0, 0))
    }
}

impl Display for HexCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0.x, self.0.y)
    }
}

impl std::ops::Neg for HexCoord {
    type Output = Self;

    fn neg(self) -> Self {
        HexCoord(-self.0)
    }
}

impl std::ops::Add for HexCoord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        HexCoord(self.0 + other.0)
    }
}

impl std::ops::Sub for HexCoord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        HexCoord(self.0 - other.0)
    }
}

impl std::ops::AddAssign for HexCoord {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<'a> std::ops::AddAssign<&'a HexCoord> for HexCoord {
    fn add_assign(&mut self, other: &'a Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for HexCoord {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl<'a> std::ops::SubAssign<&'a HexCoord> for HexCoord {
    fn sub_assign(&mut self, other: &Self) {
        self.0 -= other.0;
    }
}

impl Coord for HexCoord {
    fn angle_to_direction(&self, dir_type: DirectionType, angle: f32) -> Direction {
        // We can ignore the coordinate, as angle to direction mapping
        // is the same for any coordinate.

        use Direction::*;
        use DirectionType::*;

        // TODO Can this be simplified?

        let norm_angle = angle.rem_euclid(2.0 * PI);
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

    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Direction::*;
        use DirectionType::*;
        let offset = match dir_type {
            Face => match dir {
                NorthEast => ivec2(1, 0),
                North => ivec2(0, 1),
                NorthWest => ivec2(-1, 1),
                SouthWest => ivec2(-1, 0),
                South => ivec2(0, -1),
                SouthEast => ivec2(1, -1),
                _ => return None,
            },
            Vertex => match dir {
                East => ivec2(2, -1),
                NorthEast => ivec2(1, 1),
                NorthWest => ivec2(-1, 2),
                West => ivec2(-2, 1),
                SouthWest => ivec2(-1, -1),
                SouthEast => ivec2(1, -2),
                _ => return None,
            },
        };
        Some(HexCoord(offset))
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

    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        let (x, y) = (array_offset.0 as i32, array_offset.1 as i32);
        HexCoord(ivec2(x, y - (x + (x & 1)) / 2))
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Regular hexagonal grids with cells of specific size.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct HexSizedGrid {
    inradius: f32,
}

impl HexSizedGrid {
    /// The conversion matrix from hex axial coordinates to screen space.
    // TODO: Allow this to be constant?
    fn conversion_matrix() -> Mat2 {
        Mat2::from_cols(
            Vec2::from_angle(PI / 6.0f32) * 3.0f32.sqrt(),
            Vec2::from_angle(PI / 2.0f32) * 3.0f32.sqrt(),
        )
    }

    /// Helper function for rounding floating point hex axial coordinates to
    /// the nearest integral hex axial coordinate.
    fn hex_round(q: f32, r: f32) -> HexCoord {
        // Convert the axial coordinates to cubical coordinates.
        let x = q;
        let z = r;
        let y = -x - z;
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
            // like it should matter for axial coordinates?
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }
        // Convert back to axial coordinates.
        HexCoord(ivec2(rx as i32, rz as i32))
    }
}

impl SizedGrid for HexSizedGrid {
    type Coord = HexCoord;

    /// Construct a new `HexSizedGrid` with the given inradius.
    fn new(inradius: f32) -> Self {
        HexSizedGrid { inradius }
    }

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

    fn vertices(&self, coord: Self::Coord) -> Vec<Point> {
        let center = self.grid_to_screen(coord);
        (0..6)
            .map(|i| center + Vec2::from_angle(i as f32 * PI / 3.0) * self.circumradius())
            .collect()
    }

    fn grid_to_screen(&self, coord: Self::Coord) -> Point {
        self.circumradius()
            * Self::conversion_matrix()
            * Vec2::new(coord.0.x as f32, coord.0.y as f32)
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let grid = Self::conversion_matrix().inverse() * point / self.circumradius();
        HexSizedGrid::hex_round(grid.x, grid.y)
    }
}
