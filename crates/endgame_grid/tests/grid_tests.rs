#![feature(associated_type_defaults)]

// Bring the macros and other important things into scope.
use endgame_direction::Direction;
use endgame_grid::hex::{HexGridCoord, HexSizedGrid};
use endgame_grid::square::{SquareGridCoord, SquareSizedGrid};
use endgame_grid::triangle::{TriangleGridCoord, TrianglePoint, TriangleSizedGrid};
use endgame_grid::{GridCoord, SizedGrid};
use glam::{vec2, IVec2, Vec2};
use proptest::prelude::*;

fn ivec2_strategy() -> impl Strategy<Value = IVec2> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (-100000..100000, -100000..100000).prop_map(|(x, y)| IVec2::new(x, y))
}

fn hexgridcoord_strategy() -> impl Strategy<Value = HexGridCoord> {
    ivec2_strategy().prop_map(|vec| HexGridCoord::from_ivec2(vec))
}

fn squaregridcoord_strategy() -> impl Strategy<Value = SquareGridCoord> {
    ivec2_strategy().prop_map(|vec| SquareGridCoord::from_ivec2(vec))
}

fn trianglegridcoord_strategy() -> impl Strategy<Value = TriangleGridCoord> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (
        -100000..100000,
        -100000..100000,
        prop_oneof![Just(TrianglePoint::Up), Just(TrianglePoint::Down)],
    )
        .prop_map(|(x, y, p)| TriangleGridCoord::new(x, y, p))
}

/// Helper function that tests that for given grid coordinate, that
/// moving in all allowed directions is possible, and that moving the
/// opposite direction returns to the original coordinate.
fn grid_direction<G: GridCoord + Copy>(coord: G) -> Result<(), TestCaseError> {
    for dir in &coord.allowed_directions() {
        prop_assert!(
            coord.allowed_direction(dir),
            "Direction {dir} should be allowed from coordinate {coord}"
        );
        let opt_moved_coord = coord.move_in_direction(dir);
        prop_assert!(
            opt_moved_coord.is_some(),
            "Direction {dir} should be allowed from coordinate {coord}"
        );
        let moved_coord = opt_moved_coord.unwrap();
        let back_dir = dir.opposite();
        let opt_returned_coord = moved_coord.move_in_direction(back_dir);
        prop_assert!(
            opt_returned_coord.is_some(),
            "Moving in direction {dir} from {coord} to {moved_coord} and then back should be allowed."
        );
        let returned_coord = opt_returned_coord.unwrap();
        prop_assert_eq!(
            coord,
            returned_coord,
            "Moving from {} in direction {} to {} and then returning {} should be the identity.",
            coord,
            dir,
            moved_coord,
            dir.opposite()
        );
    }
    // Check that for all directions that are not allowed, that moving that
    // direction is not possible.
    for dir in &(Direction::VALUES.difference(coord.allowed_directions())) {
        prop_assert!(
            !coord.allowed_direction(dir),
            "Direction {dir} should not be allowed from coordinate {coord}"
        );
        let opt_moved_coord = coord.move_in_direction(dir);
        prop_assert!(
            opt_moved_coord.is_none(),
            "Direction {dir} should not be allowed from coordinate {coord}"
        );
    }
    Ok(())
}

/// Helper that verifies that for all allowed directions of a given grid
/// coordinate, the angle for that direction matches the direction
/// angle_to_direction reports.
fn grid_angle_to_direction<G: GridCoord + Copy>(coord: G) -> Result<(), TestCaseError> {
    for dir in &coord.allowed_directions() {
        let angle = dir.angle();
        let direction = coord.angle_to_direction(angle);
        prop_assert_eq!(direction, dir, "");
        // FIX??
        //  "Direction {dir} should be returned for angle {angle} from coordinate {coord}");
    }

    Ok(())
}

fn sized_grid_radius<SG: SizedGrid>(sized_grid: SG) -> Result<(), TestCaseError> {
    prop_assert!(
        sized_grid.inradius() <= sized_grid.circumradius(),
        "Inradius {} should be less than or equal to circumradius {}.",
        sized_grid.inradius(),
        sized_grid.circumradius()
    );
    Ok(())
}

fn sized_grid_identity<SG: SizedGrid + Copy>(
    sized_grid: SG,
    coord: SG::Coord,
) -> Result<(), TestCaseError>
where
    SG::Coord: Copy,
{
    let screen_coord = sized_grid.grid_to_screen(coord);
    let back_coord = sized_grid.screen_to_grid(screen_coord);
    prop_assert_eq!(coord, back_coord, "With screen coordinate {}", screen_coord);
    Ok(())
}

fn sized_grid_commutation<SG: SizedGrid + Copy>(
    sized_grid: SG,
    coord: SG::Coord,
) -> Result<(), TestCaseError>
where
    SG::Coord: Copy,
{
    let screen_coord = sized_grid.grid_to_screen(coord);

    for dir in &coord.allowed_directions() {
        let opt_moved_coord = coord.move_in_direction(dir);
        prop_assert!(
            opt_moved_coord.is_some(),
            "Direction {dir} should be allowed from coordinate {coord}"
        );
        let moved_coord = opt_moved_coord.unwrap();
        let moved_screen_coord = sized_grid.grid_to_screen(moved_coord);
        let opt_angle = coord.direction_angle(dir);
        prop_assert!(
            opt_angle.is_some(),
            "Direction {dir} should be allowed from coordinate {moved_coord}"
        );
        let angle = opt_angle.unwrap();
        let opt_back_angle = moved_coord.direction_angle(dir.opposite());
        prop_assert!(
            opt_back_angle.is_some(),
            "Direction {dir} should be allowed from coordinate {moved_coord}"
        );
        let back_angle = opt_back_angle.unwrap();

        // Twice the inradius should always be enough to get us back to the
        // original coordinate.
        let back_vec = Vec2::from_angle(back_angle) * 2.0f32 * sized_grid.inradius();
        let moved_back_coord = moved_screen_coord + back_vec;
        prop_assert_eq!(
            coord,
            sized_grid.screen_to_grid(moved_back_coord),
            "Moved from {} (screen {}) via {} direction (angle {}) to \
                {:?} (screen {}) but got back to {} via {} (angle {}).",
            coord,
            screen_coord,
            dir,
            angle,
            moved_coord,
            moved_screen_coord,
            moved_back_coord,
            back_vec,
            back_angle,
        );

        let moved_grid_coord = sized_grid.screen_to_grid(moved_screen_coord);
        prop_assert_eq!(moved_coord, moved_grid_coord, "");
    }
    Ok(())
}

static SIZE_RANGE: std::ops::Range<f32> = 0.001..1000000.0f32;

proptest! {
    #[test]
    fn hex_angle_to_direction(coord in hexgridcoord_strategy()) {
                println!(
            "{} {}",
            vec2(3.0 / 2.0, 3.0f32.sqrt() / 2.0).to_angle(),
            vec2(0.0, 3.0f32.sqrt()).to_angle()
        );

        grid_angle_to_direction(coord)?;
    }

    #[test]
    fn square_angle_to_direction(coord in squaregridcoord_strategy()) {
        grid_angle_to_direction(coord)?;
    }

    #[test]
    fn triangle_angle_to_direction(coord in trianglegridcoord_strategy()) {
        grid_angle_to_direction(coord)?;
    }

    #[test]
    fn hex_grid_direction(coord in hexgridcoord_strategy()) {
        grid_direction(coord)?;
    }

    #[test]
    fn square_grid_direction(coord in squaregridcoord_strategy()) {
        grid_direction(coord)?;
    }

    #[test]
    fn triangle_grid_direction(coord in trianglegridcoord_strategy()) {
        grid_direction(coord)?;
    }

    #[test]
    fn hex_grid_to_array_offset(coord in hexgridcoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, HexGridCoord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);
    }

    #[test]
    fn square_grid_to_array_offset(coord in squaregridcoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, SquareGridCoord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);

    }

    #[test]
    fn triangle_grid_to_array_offset(coord in trianglegridcoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, TriangleGridCoord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);
    }

    #[test]
    fn hex_sized_grid(size in &SIZE_RANGE, coord in hexgridcoord_strategy()) {
        let sized_grid = HexSizedGrid::new(size);
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn square_sized_grid(size in &SIZE_RANGE, coord in squaregridcoord_strategy()) {
        let sized_grid = SquareSizedGrid::new(size);
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn triangle_sized_grid(size in &SIZE_RANGE, coord in trianglegridcoord_strategy()) {
        let sized_grid = TriangleSizedGrid::new(size);
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn hex_sized_grid_commutation(size in 0.0001..1000000.0f32,
        coord in hexgridcoord_strategy()) {
        sized_grid_commutation(HexSizedGrid::new(size), coord)?;
    }

    #[test]
    fn square_sized_grid_commutation(size in 0.0001..1000000.0f32,
        coord in squaregridcoord_strategy()) {
        sized_grid_commutation(SquareSizedGrid::new(size), coord)?;
    }

    #[test]
    fn triangle_sized_grid_commutation(size in 0.0001..1000000.0f32,
        coord in trianglegridcoord_strategy()) {
        sized_grid_commutation(TriangleSizedGrid::new(size), coord)?;
    }
}
