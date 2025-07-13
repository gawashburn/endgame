#![feature(associated_type_defaults)]

// Bring the macros and other important things into scope.
use endgame_direction::Direction;
use endgame_grid::hex::{HexCoord, HexSizedGrid};
use endgame_grid::square::{SquareCoord, SquareSizedGrid};
use endgame_grid::triangle::{TriangleCoord, TrianglePoint, TriangleSizedGrid};
use endgame_grid::{Coord, DirectionType, SizedGrid};
use glam::{IVec2, Vec2};
use proptest::prelude::*;
use std::f32::consts::PI;
use std::iter::Iterator;

fn ivec2_strategy() -> impl Strategy<Value = IVec2> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (-100000..100000, -100000..100000).prop_map(|(x, y)| IVec2::new(x, y))
}

fn hexcoord_strategy() -> impl Strategy<Value = HexCoord> {
    ivec2_strategy().prop_map(|vec| HexCoord::from_ivec2(vec))
}

fn squarecoord_strategy() -> impl Strategy<Value = SquareCoord> {
    ivec2_strategy().prop_map(|vec| SquareCoord::from_ivec2(vec))
}

fn trianglecoord_strategy() -> impl Strategy<Value = TriangleCoord> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (
        -100000..100000,
        -100000..100000,
        prop_oneof![Just(TrianglePoint::Up), Just(TrianglePoint::Down)],
    )
        .prop_map(|(x, y, p)| TriangleCoord::new(x, y, p))
}

fn coord_neg<C: Coord + Copy>(coord: C) -> Result<(), TestCaseError> {
    let neg_coord = -coord;
    prop_assert_eq!(
        -neg_coord,
        coord,
        "Negating negation should be the identity.",
    );
    prop_assert_eq!(
        coord + neg_coord,
        C::default(),
        "Adding the negative of a coordinate should yield the zero coordinate."
    );
    Ok(())
}

fn coord_add_ident<C: Coord + Copy>(coord: C) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        coord + C::default(),
        coord,
        "Grid coordinate addition respect the additive identity."
    );
    // Verify with AddAssign as well.
    let mut coord_copy = coord;
    coord_copy += C::default();
    prop_assert_eq!(
        coord_copy,
        coord,
        "Grid coordinate addition with AddAssign should respect the additive identity."
    );
    Ok(())
}

fn coord_add_comm<C: Coord + Copy>(coord1: C, coord2: C) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        coord1 + coord2,
        coord2 + coord1,
        "Grid coordinate addition should be commutative."
    );

    // Verify with AddAssign as well.
    let mut coord1_copy = coord1;
    coord1_copy += coord2;
    let mut coord2_copy = coord2;
    coord2_copy += coord1;
    prop_assert_eq!(
        coord1_copy,
        coord2_copy,
        "Grid coordinate addition with AddAssign should be commutative."
    );

    Ok(())
}

fn coord_add_assoc<C: Coord + Copy>(coord1: C, coord2: C, coord3: C) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        (coord1 + coord2) + coord3,
        coord1 + (coord2 + coord3),
        "Grid coordinate addition should be commutative."
    );

    // Verify with AddAssign as well.
    let mut coord_grouped_copy = coord1 + coord2;
    coord_grouped_copy += coord3;
    let mut coord1_copy = coord1;
    coord1_copy += coord2 + coord3;
    prop_assert_eq!(
        coord_grouped_copy,
        coord1_copy,
        "Grid coordinate addition with AddAssign should be associative."
    );

    Ok(())
}

fn coord_sub_anticomm<C: Coord + Copy>(coord1: C, coord2: C) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        coord1 - coord2,
        -(coord2 - coord1),
        "Grid coordinate subtraction should be anti-commutative."
    );

    // Verify with SubAssign as well.
    let mut coord1_copy = coord1;
    coord1_copy -= coord2;
    let mut coord2_copy = coord2;
    coord2_copy -= coord1;
    prop_assert_eq!(
        coord1_copy,
        -coord2_copy,
        "Grid coordinate subtraction with SubAssign should be anti-commutative."
    );

    Ok(())
}

/// Helper function that tests that for given grid coordinate, that
/// moving in all allowed directions is possible, and that moving the
/// opposite direction returns to the original coordinate.
fn grid_direction<C: Coord + Copy>(coord: C, dir_type: DirectionType) -> Result<(), TestCaseError> {
    for dir in &coord.allowed_directions(dir_type) {
        prop_assert!(
            coord.allowed_direction(dir_type, dir),
            "{dir_type} direction {dir} should be allowed from coordinate {coord}"
        );
        let opt_moved_offset = coord.offset_in_direction(dir_type, dir);
        prop_assert!(
            opt_moved_offset.is_some(),
            "{dir_type} direction {dir} should be allowed from coordinate {coord}"
        );
        let moved_coord = coord + opt_moved_offset.unwrap();
        let back_dir = dir.opposite();
        let opt_returned_offset = moved_coord.offset_in_direction(dir_type, back_dir);
        prop_assert!(
            opt_returned_offset.is_some(),
            "Moving in direction {dir} from {coord} to {moved_coord} and then back should be allowed."
        );
        let returned_coord = moved_coord + opt_returned_offset.unwrap();
        prop_assert_eq!(
            coord,
            returned_coord,
            "Moving from {} in {} direction {} to {} and then returning {} should be the identity.",
            coord,
            dir_type,
            dir,
            moved_coord,
            dir.opposite()
        );
    }
    // Check that for all directions that are not allowed, that moving that
    // direction is not possible.
    for dir in &(Direction::VALUES.difference(coord.allowed_directions(dir_type))) {
        prop_assert!(
            !coord.allowed_direction(dir_type, dir),
            "{dir_type} direction {dir} should not be allowed from coordinate {coord}"
        );
        let opt_moved_offset = coord.offset_in_direction(dir_type, dir);
        prop_assert!(
            opt_moved_offset.is_none(),
            "{dir_type} direction {dir} should not be allowed from coordinate {coord}"
        );
    }
    Ok(())
}

/// Helper that verifies that for all allowed directions of a given grid
/// coordinate, the angle for that direction matches the direction
/// angle_to_direction reports.
fn grid_angle_to_direction<C: Coord + Copy>(
    coord: C,
    dir_type: DirectionType,
) -> Result<(), TestCaseError> {
    for dir in &coord.allowed_directions(dir_type) {
        let angle = dir.angle();
        let direction = coord.angle_to_direction(dir_type, angle);
        prop_assert_eq!(direction, dir, "");
        // FIX??
        //  "Direction {dir} should be returned for angle {angle} from coordinate {coord}");
    }

    Ok(())
}

fn grid_iterator<C: Coord + Copy>(coord: C, dir_type: DirectionType) -> Result<(), TestCaseError> {
    // Disallow directions will produce empty iterators.
    for dir in &(Direction::VALUES.difference(coord.allowed_directions(dir_type))) {
        let mut iter = coord.direction_iterator(dir_type, dir, ..);
        prop_assert!(
            iter.count() == 0,
            "Iterator for disallowed {dir_type} direction {dir} should be empty"
        );
    }

    for dir in &coord.allowed_directions(dir_type) {
        let inclusive_iter = coord.direction_iterator(dir_type, dir, ..=10);
        let inclusize_count = (0..=10).count();
        // Ensure that all elements in the iterator have the correct offset
        // relationship.
        let mut current_coord = coord;
        for (index, c) in inclusive_iter.clone().enumerate().skip(1) {
            prop_assert_eq!(
                c - current_coord,
                coord.offset_in_direction(dir_type, dir).unwrap(),
                "Offset between coordinates should match the direction offset."
            );
            current_coord = c;
            prop_assert!(index <= inclusize_count, "Iterator has unexpected length",);
        }
        // Ensure that the inclusive iterator has the correct length.
        prop_assert_eq!(inclusive_iter.count(), inclusize_count);

        // Ensure that the exclusive iterator has the correct length.
        let exclusive_iter = coord.direction_iterator(dir_type, dir, ..10);
        prop_assert_eq!(exclusive_iter.count(), (0..10).count());

        // For allowed directions, the first element of the iterator will always be
        // the coordinate itself, even if the range is empty.
        let unbounded_iter = coord.direction_iterator(dir_type, dir, ..);
        prop_assert!(
            unbounded_iter.take(1).collect::<Vec<_>>().as_slice() == [coord],
            "For inclusive range, first element of iterator for {dir_type} \
            direction {dir} should be the coordinate itself."
        );
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
    dir_type: DirectionType,
    // TODO Improve, because of the way hexagonal vertex directions work, we
    // need to add in a larger back offset than usual.  Find a better
    // solution to generalize this test.
    vertex_back_offset: f32,
) -> Result<(), TestCaseError>
where
    SG::Coord: Copy,
{
    let screen_coord = sized_grid.grid_to_screen(coord);

    for dir in &coord.allowed_directions(dir_type) {
        let opt_moved_offset = coord.offset_in_direction(dir_type, dir);
        prop_assert!(
            opt_moved_offset.is_some(),
            "Direction {dir} should be allowed from coordinate {coord}"
        );
        let moved_coord = coord + opt_moved_offset.unwrap();
        let moved_screen_coord = sized_grid.grid_to_screen(moved_coord.clone());
        let opt_angle = coord.direction_angle(dir_type, dir);
        prop_assert!(
            opt_angle.is_some(),
            "Direction {dir} should be allowed from coordinate {moved_coord}"
        );
        let angle = opt_angle.unwrap();
        let opt_back_angle = moved_coord.direction_angle(dir_type, dir.opposite());
        prop_assert!(
            opt_back_angle.is_some(),
            "Direction {dir} should be allowed from coordinate {moved_coord}"
        );
        let back_angle = opt_back_angle.unwrap();

        // For face directions twice the inradius should always be enough
        // to get us back to the original coordinate.
        // Similarly, for vertex directions, twice the circumradius.
        // However, because hexagonal vertex directions jump further,
        // need to add a fudge factor in that case.
        let back_dist = match dir_type {
            DirectionType::Face => sized_grid.inradius() * 2.0,
            DirectionType::Vertex => sized_grid.circumradius() * 2.0 + vertex_back_offset,
        };
        let back_vec = Vec2::from_angle(back_angle) * back_dist;
        let moved_back_coord = moved_screen_coord + back_vec;
        prop_assert_eq!(
            coord,
            sized_grid.screen_to_grid(moved_back_coord),
            "Moved from {} (screen {}) via {} direction (angle {}) to \
                {} (screen {}) but got back to {} via {} (angle {}).",
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

fn sized_grid_vertices<SG: SizedGrid + Copy>(
    sized_grid: SG,
    coord: SG::Coord,
    expected_size: usize,
) -> Result<(), TestCaseError> {
    let vertices = sized_grid.vertices(coord.clone());
    prop_assert_eq!(
        vertices.len(),
        expected_size,
        "Grid coordinate {} should have {} vertices, but got {}.",
        coord,
        expected_size,
        vertices.len()
    );
    let mut current_angle = 0.0;
    let mut opt_vertex = None;
    for (index, vertex) in vertices.iter().enumerate() {
        // Check that the edges are of the correct length.
        if let Some(prev_vertex) = opt_vertex {
            let edge_vector: Vec2 = *vertex - prev_vertex;
            let edge_vector_len = edge_vector.length();
            let diff = (sized_grid.edge_length() - edge_vector_len).abs();
            prop_assert!(
                // TODO allowing 2% error seems a bit high, but maybe that is the
                // best we can expect with f32 and extreme grid sizes?
                (diff / edge_vector_len) < 0.02,
                "The length between the vertices does not match the expected length: {} vs {} diff {}",
                sized_grid.edge_length(),
                edge_vector_len,
                diff,
            );
        }
        // Need to measure the angles relative to the center of the cell, otherwise
        // the vertices will not appear to be in clockwise order.
        let vertex_angle = (vertex - sized_grid.grid_to_screen(coord.clone()))
            .normalize()
            .to_angle()
            .rem_euclid(2.0 * PI);
        println!(
            "Vertex {index} of {coord} has angle {vertex_angle} (current angle: {current_angle})"
        );
        prop_assert!(
            vertex_angle >= current_angle,
            "Vertex angles are not in clockwise order: for vertex {index} of {coord}, {vertex_angle} < {current_angle}"
        );
        opt_vertex = Some(vertex.clone());
        current_angle = vertex_angle;
    }

    Ok(())
}

static SIZE_RANGE: std::ops::Range<f32> = 0.001..1000000.0f32;

proptest! {
    #[test]
    fn hex_unary_op(coord1 in hexcoord_strategy()) {
        coord_neg(coord1)?;
        coord_add_ident(coord1)?;
    }

    #[test]
    fn square_unary_op(coord1 in squarecoord_strategy()) {
        coord_neg(coord1)?;
        coord_add_ident(coord1)?;
    }

    #[test]
    fn triangle_unary_op(coord1 in trianglecoord_strategy()) {
        // Also verify the `is_up` method for triangle coordinates.
        let coord_point = coord1.to_ivec2().1;
        prop_assert!(!coord1.is_up() || matches!(coord_point, TrianglePoint::Up),
            "The is_up method does not match the TrianglePoint value.");
                // Also verify the `is_up` method for triangle coordinates.
        prop_assert!(!matches!(coord_point, TrianglePoint::Up) || coord1.is_up(),
            "The is_up method does not match the TrianglePoint value.");

        coord_neg(coord1)?;
        coord_add_ident(coord1)?;
    }

    #[test]
    fn hex_binary_op(coord1 in hexcoord_strategy(), coord2 in hexcoord_strategy()) {
        coord_add_comm(coord1, coord2)?;
        coord_sub_anticomm(coord1, coord2)?;
    }

    #[test]
    fn square_binary_op(coord1 in squarecoord_strategy(), coord2 in squarecoord_strategy()) {
        coord_add_comm(coord1, coord2)?;
        coord_sub_anticomm(coord1, coord2)?;
    }

    #[test]
    fn triangle_binary_op(coord1 in trianglecoord_strategy(), coord2 in trianglecoord_strategy()) {
        coord_add_comm(coord1, coord2)?;
        coord_sub_anticomm(coord1, coord2)?;
    }

    #[test]
    fn hex_assoc(coord1 in hexcoord_strategy(), coord2 in hexcoord_strategy(), coord3 in hexcoord_strategy()) {
        coord_add_assoc(coord1, coord2, coord3)?;
    }

    #[test]
    fn square_assoc(coord1 in squarecoord_strategy(), coord2 in squarecoord_strategy(), coord3 in squarecoord_strategy()) {
        coord_add_assoc(coord1, coord2, coord3)?;
    }

    #[test]
    fn triangle_assoc(coord1 in trianglecoord_strategy(), coord2 in trianglecoord_strategy(), coord3 in trianglecoord_strategy()) {
        coord_add_assoc(coord1, coord2, coord3)?;
    }

    #[test]
    fn hex_angle_to_direction(coord in hexcoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_angle_to_direction(coord, dt)?;
        }
    }

    #[test]
    fn square_angle_to_direction(coord in squarecoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_angle_to_direction(coord, dt)?;
        }
    }

    #[test]
    fn triangle_angle_to_direction(coord in trianglecoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_angle_to_direction(coord, dt)?;
        }
    }

    #[test]
    fn hex_grid_direction(coord in hexcoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_direction(coord, dt)?;
            grid_iterator(coord, dt)?;
        }
    }

    #[test]
    fn square_grid_direction(coord in squarecoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_direction(coord, dt)?;
            grid_iterator(coord, dt)?;
        }
    }

    #[test]
    fn triangle_grid_direction(coord in trianglecoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_direction(coord, dt)?;
            grid_iterator(coord, dt)?;
        }
    }

    #[test]
    fn hex_grid_to_array_offset(coord in hexcoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, HexCoord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);
    }

    #[test]
    fn square_grid_to_array_offset(coord in squarecoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, SquareCoord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);

    }

    #[test]
    fn triangle_grid_to_array_offset(coord in trianglecoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, TriangleCoord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);
    }

    #[test]
    fn hex_sized_grid(size in &SIZE_RANGE, coord in hexcoord_strategy()) {
        let sized_grid = HexSizedGrid::new(size);
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn square_sized_grid(size in &SIZE_RANGE, coord in squarecoord_strategy()) {
        let sized_grid = SquareSizedGrid::new(size);
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn triangle_sized_grid(size in &SIZE_RANGE, coord in trianglecoord_strategy()) {
        let sized_grid = TriangleSizedGrid::new(size);
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn hex_sized_grid_commutation(size in 0.0001..1000000.0f32,
        coord in hexcoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            let sz = HexSizedGrid::new(size);
            // Use the hex edge length to handle the larger back offset needed.
            sized_grid_commutation(sz, coord, dt, sz.edge_length())?;
        }
    }

    #[test]
    fn square_sized_grid_commutation(size in 0.0001..1000000.0f32,
        coord in squarecoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            sized_grid_commutation(SquareSizedGrid::new(size), coord, dt, 0.0)?;
        }
    }

    #[test]
    fn triangle_sized_grid_commutation(size in 0.0001..1000000.0f32,
        coord in trianglecoord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            sized_grid_commutation(TriangleSizedGrid::new(size), coord, dt, 0.0)?;
        }
    }

    #[test]
    fn hex_sized_vertices(size in 0.0001..1000000.0f32,
        coord in hexcoord_strategy()) {
        sized_grid_vertices(HexSizedGrid::new(size), coord, 6)?;
    }

    #[test]
    fn square_sized_vertices(size in 0.0001..1000000.0f32,
        coord in squarecoord_strategy()) {
        sized_grid_vertices(SquareSizedGrid::new(size), coord, 4)?;
    }

    #[test]
    fn triangle_sized_vertices(size in 0.0001..1000000.0f32,
        coord in trianglecoord_strategy()) {
        sized_grid_vertices(TriangleSizedGrid::new(size), coord, 3)?;
    }
}
