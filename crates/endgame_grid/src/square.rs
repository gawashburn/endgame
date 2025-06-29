use crate::{GridCoord, Point, SizedGrid};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, IVec2, Mat2, Vec2};
use std::f32::consts::PI;
use std::fmt::Display;

/// For a Manhattan square grid, it is possible to move in the same directions
/// from any coordinate.
const ALLOWED_DIRECTIONS: DirectionSet = {
    use Direction::*;
    DirectionSet::from_slice(&[North, East, South, West])
};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct SquareGridCoord(IVec2);

impl SquareGridCoord {
    /// Construct a new `SquareGridCoord` from x and y coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        SquareGridCoord(ivec2(x, y))
    }

    /// Construct a new `SquareGridCoord` from an `IVec2`.
    pub const fn from_ivec2(coord: IVec2) -> Self {
        SquareGridCoord(coord)
    }

    /// Convert the coordinate to an `IVec2`.
    pub const fn to_ivec2(&self) -> IVec2 {
        self.0
    }
}

impl Display for SquareGridCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0.x, self.0.y)
    }
}

impl GridCoord for SquareGridCoord {
    fn angle_to_direction(&self, angle: f32) -> Direction {
        // We can ignore the coordinate, as angle to direction mapping
        // is the same for any coordinate.
        let norm_angle = angle.rem_euclid(2.0 * PI);
        // After normalization, it is expected that the angle will not have
        // a negative sign.
        assert!(norm_angle.is_sign_positive());
        let octant = norm_angle / (PI / 4.0);
        use Direction::*;
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

    fn direction_angle(&self, dir: Direction) -> Option<f32> {
        if self.allowed_direction(dir) {
            Some(dir.angle())
        } else {
            None
        }
    }

    fn move_in_direction(&self, dir: Direction) -> Option<Self> {
        use Direction::*;
        let offset = match dir {
            North => ivec2(0, 1),
            East => ivec2(1, 0),
            South => ivec2(0, -1),
            West => ivec2(-1, 0),
            _ => return None,
        };
        Some(SquareGridCoord(self.0 + offset))
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
        // For a square grid the grid coordinates and array offsets are
        // essentially identical.
        (self.0.x as isize, self.0.y as isize)
    }

    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        // For a square grid the grid coordinates and array offsets are
        // essentially identical.
        // TODO switch array offets to 32-bits?
        SquareGridCoord(ivec2(array_offset.0 as i32, array_offset.1 as i32))
    }
}

//////////////////////////////////////////////////////////////////////////////

// Regular square grids.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SquareSizedGrid {
    inradius: f32,
}

impl SquareSizedGrid {
    /// Construct a new `Square\SizedGrid` with the given inradius.

    pub const fn new(inradius: f32) -> Self {
        SquareSizedGrid { inradius }
    }

    /// The conversion matrix from square grid coordinates to screen space.
    const fn conversion_matrix() -> Mat2 {
        Mat2::from_cols(Vec2::new(2.0, 0.0), Vec2::new(0.0, 2.0))
    }
}

impl SizedGrid for SquareSizedGrid {
    type Coord = SquareGridCoord;

    fn inradius(&self) -> f32 {
        self.inradius
    }

    fn circumradius(&self) -> f32 {
        (2.0 * self.inradius) / 2.0f32.sqrt()
    }

    fn edge_length(&self) -> f32 {
        2.0 * self.inradius
    }

    fn grid_to_screen(&self, coord: Self::Coord) -> Point {
        self.inradius * Self::conversion_matrix() * Vec2::new(coord.0.x as f32, coord.0.y as f32)
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let grid = Self::conversion_matrix().inverse() * point / self.inradius;
        SquareGridCoord(IVec2::new(grid.x.round() as i32, grid.y.round() as i32))
    }
}
