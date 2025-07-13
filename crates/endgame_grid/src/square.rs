use crate::{Coord, DirectionType, Point, SizedGrid};
use endgame_direction::{Direction, DirectionSet};
use glam::{ivec2, IVec2, Mat2, Vec2};
use std::f32::consts::PI;
use std::fmt::Display;

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
pub struct SquareCoord(IVec2);

impl SquareCoord {
    /// Construct a new `SquareGridCoord` from x and y coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        SquareCoord(ivec2(x, y))
    }

    /// Construct a new `SquareGridCoord` from an `IVec2`.
    pub const fn from_ivec2(coord: IVec2) -> Self {
        SquareCoord(coord)
    }

    /// Convert the coordinate to an `IVec2`.
    pub const fn to_ivec2(&self) -> IVec2 {
        self.0
    }
}
impl Default for SquareCoord {
    fn default() -> Self {
        SquareCoord(ivec2(0, 0))
    }
}

impl Display for SquareCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0.x, self.0.y)
    }
}

impl std::ops::Neg for SquareCoord {
    type Output = Self;

    fn neg(self) -> Self {
        SquareCoord(-self.0)
    }
}

impl std::ops::Add for SquareCoord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SquareCoord(self.0 + other.0)
    }
}

impl std::ops::Sub for SquareCoord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SquareCoord(self.0 - other.0)
    }
}

impl std::ops::AddAssign for SquareCoord {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<'a> std::ops::AddAssign<&'a SquareCoord> for SquareCoord {
    fn add_assign(&mut self, other: &Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for SquareCoord {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl<'a> std::ops::SubAssign<&'a SquareCoord> for SquareCoord {
    fn sub_assign(&mut self, other: &Self) {
        self.0 -= other.0;
    }
}

impl Coord for SquareCoord {
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

    fn offset_in_direction(&self, dir_type: DirectionType, dir: Direction) -> Option<Self> {
        use Direction::*;
        use DirectionType::*;
        let offset = match dir_type {
            Face => match dir {
                North => ivec2(0, 1),
                East => ivec2(1, 0),
                South => ivec2(0, -1),
                West => ivec2(-1, 0),
                _ => return None,
            },
            Vertex => match dir {
                NorthEast => ivec2(1, 1),
                SouthEast => ivec2(1, -1),
                SouthWest => ivec2(-1, -1),
                NorthWest => ivec2(-1, 1),
                _ => return None,
            },
        };
        Some(SquareCoord(offset))
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

    fn array_offset_to_grid(array_offset: (isize, isize)) -> Self {
        // For a square grid the grid coordinates and array offsets are
        // essentially identical.
        // TODO switch array offsets to 32-bits?
        SquareCoord(ivec2(array_offset.0 as i32, array_offset.1 as i32))
    }
}

//////////////////////////////////////////////////////////////////////////////

// Regular square grids.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SquareSizedGrid {
    inradius: f32,
}

impl SquareSizedGrid {
    /// The conversion matrix from square grid coordinates to screen space.
    const fn conversion_matrix() -> Mat2 {
        Mat2::from_cols(Vec2::new(2.0, 0.0), Vec2::new(0.0, 2.0))
    }
}

impl SizedGrid for SquareSizedGrid {
    type Coord = SquareCoord;

    fn new(inradius: f32) -> Self {
        SquareSizedGrid { inradius }
    }

    fn inradius(&self) -> f32 {
        self.inradius
    }

    fn circumradius(&self) -> f32 {
        (2.0 * self.inradius) / 2.0f32.sqrt()
    }

    fn edge_length(&self) -> f32 {
        2.0 * self.inradius
    }

    fn vertices(&self, coord: Self::Coord) -> Vec<Point> {
        let center = self.grid_to_screen(coord);
        (0..4)
            .map(|i| {
                center + Vec2::from_angle((PI / 4.0) + i as f32 * (PI / 2.0)) * self.circumradius()
            })
            .collect()
    }

    fn grid_to_screen(&self, coord: Self::Coord) -> Point {
        self.inradius * Self::conversion_matrix() * Vec2::new(coord.0.x as f32, coord.0.y as f32)
    }

    fn screen_to_grid(&self, point: Point) -> Self::Coord {
        let grid = Self::conversion_matrix().inverse() * point / self.inradius;
        SquareCoord(IVec2::new(grid.x.round() as i32, grid.y.round() as i32))
    }
}
