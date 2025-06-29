use crate::{GridCoord, Point, SizedGrid};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, Mat2, Vec2};
use std::f32::consts::PI;
use std::fmt::Display;

/// For a hexagonal grid, it is possible to move in the same directions
/// from any coordinate.
const ALLOWED_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[North, NorthEast, SouthEast, South, SouthWest, NorthWest])
};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
/// This implementation is based upon the axial coordinate system as described
/// by <https://www.redblobgames.com/grids/hexagons/>.
/// It uses a flat-topped hexagonal grid with even-q offset coordinates.
///
// TODO Add support for pointy top orientation?
// IVec2::x is the same as the axial q and IVec2::y is the axial r.
pub struct HexGridCoord(glam::IVec2);

impl HexGridCoord {
    /// Construct a new `HexGridCoord` from x and y coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        HexGridCoord(ivec2(x, y))
    }

    /// Construct a new `HexGridCoord` from an `IVec2`.
    pub const fn from_ivec2(coord: glam::IVec2) -> Self {
        HexGridCoord(coord)
    }

    /// Convert the coordinate to an `IVec2`.
    pub const fn to_ivec2(&self) -> glam::IVec2 {
        self.0
    }
}

impl Display for HexGridCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0.x, self.0.y)
    }
}

impl GridCoord for HexGridCoord {
    fn angle_to_direction(&self, angle: f32) -> Direction {
        // We can ignore the coordinate, as angle to direction mapping
        // is the same for any coordinate.

        let norm_angle = angle.rem_euclid(2.0 * PI);
        // After normalization, it is expected that the angle will not have
        // a negative sign.
        assert!(norm_angle.is_sign_positive());
        let hextant = norm_angle / (PI / 3.0);
        use Direction::*;
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

    fn direction_angle(&self, dir: Direction) -> Option<f32> {
        use Direction::*;
        Some(match dir {
            NorthEast => 1.0 * PI / 6.0,
            NorthWest => 5.0 * PI / 6.0,
            SouthWest => 7.0 * PI / 6.0,
            SouthEast => 11.0 * PI / 6.0,
            // North and South correspond to their usual angle.
            North | South => dir.angle(),
            _ => return None,
        })
    }

    fn move_in_direction(&self, dir: Direction) -> Option<Self> {
        use Direction::*;
        let offset = match dir {
            NorthEast => ivec2(1, 0),
            North => ivec2(0, 1),
            NorthWest => ivec2(-1, 1),
            SouthWest => ivec2(-1, 0),
            South => ivec2(0, -1),
            SouthEast => ivec2(1, -1),
            _ => return None,
        };
        Some(HexGridCoord(self.0 + offset))
    }

    fn allowed_direction(&self, dir: Direction) -> bool {
        // We can ignore the coordinate, as the allowed directions
        // are the same from any coordinate.
        ALLOWED_DIRECTIONS.contains(dir)
    }

    fn allowed_directions(&self) -> DirectionSet {
        // We can ignore the coordinate, as the allowed directions
        // are the same from any coordinate.
        ALLOWED_DIRECTIONS.clone()
    }
    fn grid_to_array_offset(&self) -> (isize, isize) {
        let (q, r) = (self.0.x as isize, self.0.y as isize);
        (q, r + (q + (q & 1)) / 2)
    }

    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        let (x, y) = (array_offset.0 as i32, array_offset.1 as i32);
        HexGridCoord(ivec2(x, y - (x + (x & 1)) / 2))
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Regular hexagonal grids with cells of specific size.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct HexSizedGrid {
    inradius: f32,
}

impl HexSizedGrid {
    /// Construct a new `HexSizedGrid` with the given inradius.
    pub const fn new(inradius: f32) -> Self {
        HexSizedGrid { inradius }
    }

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
    fn hex_round(q: f32, r: f32) -> HexGridCoord {
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
        HexGridCoord(ivec2(rx as i32, rz as i32))
    }
}

impl SizedGrid for HexSizedGrid {
    type Coord = HexGridCoord;

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
