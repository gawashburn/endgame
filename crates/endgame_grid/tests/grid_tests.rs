#![feature(associated_type_defaults)]

// Bring the macros and other important things into scope.
use endgame_direction::Direction;
use endgame_grid::triangle::TrianglePoint;
use endgame_grid::{dynamic, hex, square, triangle, ModuleCoord, Shape};
use endgame_grid::{Coord, DirectionType, SizedGrid};
use glam::{IVec2, Vec2};
use proptest::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::iter::Iterator;

//////////////////////////////////////////////////////////////////////////////

fn kind_strategy() -> impl Strategy<Value=dynamic::Kind> {
    prop_oneof![
        Just(dynamic::Kind::Hex),
        Just(dynamic::Kind::Square),
        Just(dynamic::Kind::Triangle),
    ]
}

/// Helper function to generate a strategy for coordinate
/// components.  This was added because apparently,
/// just using a range does not guarantee that all small
/// values like 0, 1, -1 will be explored.
fn coord_strategy() -> impl Strategy<Value=i32> {
    prop_oneof![
        -100000..100000,
        Just(0),
        Just(1),
        Just(2),
        Just(-1),
        Just(-2),
        Just(100000),
        Just(-100000),
    ]
}

/// A helper function to generate a strategy for smaller
/// coordinate values.  This is useful when testing
/// operations that are linear or worse in their runtime.
fn small_coord_strategy() -> impl Strategy<Value=i32> {
    prop_oneof![
        -1000..1000,
        Just(0),
        Just(1),
        Just(2),
        Just(-1),
        Just(-2),
        Just(1000),
        Just(-1000),
    ]
}

fn ivec2_strategy() -> impl Strategy<Value=IVec2> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (coord_strategy(), coord_strategy()).prop_map(|(x, y)| IVec2::new(x, y))
}

fn hexcoord_strategy() -> impl Strategy<Value=hex::Coord> {
    ivec2_strategy().prop_map(|vec| hex::Coord::from_ivec2(vec))
}

fn small_hexcoord_strategy() -> impl Strategy<Value=hex::Coord> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (small_coord_strategy(), small_coord_strategy()).prop_map(|(x, y)| hex::Coord::new(x, y))
}

fn squarecoord_strategy() -> impl Strategy<Value=square::Coord> {
    ivec2_strategy().prop_map(|vec| square::Coord::from_ivec2(vec))
}

fn small_squarecoord_strategy() -> impl Strategy<Value=square::Coord> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (small_coord_strategy(), small_coord_strategy()).prop_map(|(x, y)| square::Coord::new(x, y))
}

fn trianglecoord_strategy() -> impl Strategy<Value=triangle::Coord> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (
        coord_strategy(),
        coord_strategy(),
        prop_oneof![Just(TrianglePoint::Up), Just(TrianglePoint::Down)],
    )
        .prop_map(|(x, y, p)| triangle::Coord::new(x, y, p))
}

fn small_trianglecoord_strategy() -> impl Strategy<Value=triangle::Coord> {
    // Use a restricted range to avoid issues with overflowing in tests.
    // TODO Investigate how other Rust libraries handle this?
    (
        small_coord_strategy(),
        small_coord_strategy(),
        prop_oneof![Just(TrianglePoint::Up), Just(TrianglePoint::Down)],
    )
        .prop_map(|(x, y, p)| triangle::Coord::new(x, y, p))
}

fn dynamic_coord_strategy() -> impl Strategy<Value=dynamic::Coord> {
    prop_oneof![
        hexcoord_strategy().prop_map(dynamic::Coord::Hex),
        squarecoord_strategy().prop_map(dynamic::Coord::Square),
        trianglecoord_strategy().prop_map(dynamic::Coord::Triangle),
    ]
}

fn small_dynamic_coord_strategy() -> impl Strategy<Value=dynamic::Coord> {
    prop_oneof![
        small_hexcoord_strategy().prop_map(dynamic::Coord::Hex),
        small_squarecoord_strategy().prop_map(dynamic::Coord::Square),
        small_trianglecoord_strategy().prop_map(dynamic::Coord::Triangle),
    ]
}

//////////////////////////////////////////////////////////////////////////////

/// Helper to check if two coordinates are adjacent by face direction.
/// As an added check it also validates that there is nothing strange
/// going wrong and that there are multiple directions that can be used
/// to move between the two coordinates.
fn check_adjacent<C: Coord + Copy>(coord1: C, coord2: C) -> bool {
    coord2
        .allowed_directions(DirectionType::Face)
        .iter()
        .filter(|d| {
            coord1
                == coord2
                .move_in_direction(DirectionType::Face, *d)
                .expect("Allowed direction should have an offset")
        })
        .count()
        == 1
}

//////////////////////////////////////////////////////////////////////////////

fn coord_neg<MC: ModuleCoord + Copy>(coord: MC) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    let neg_coord = -coord;
    prop_assert_eq!(
        -neg_coord,
        coord,
        "Negating negation should be the identity.",
    );
    prop_assert_eq!(
        coord + neg_coord,
        MC::default(),
        "Adding the negative of a coordinate should yield the zero coordinate."
    );
    Ok(())
}

fn coord_add_ident<MC: ModuleCoord + Copy>(coord: MC) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    let add_unit = MC::default();
    prop_assert_eq!(
        coord + add_unit,
        coord,
        "Grid coordinate addition respect the additive identity."
    );
    // Verify with AddAssign as well.
    let mut coord_copy = coord;
    coord_copy += add_unit;
    prop_assert_eq!(
        coord_copy,
        coord,
        "Grid coordinate addition with AddAssign should respect the additive identity."
    );

    if coord != add_unit {
        prop_assert_ne!(
            coord + coord,
            add_unit,
            "Grid coordinates other than the additive identity should not be equal to the \
             additive identity when added to themselves."
        );

        prop_assert_ne!(
            coord + coord,
            coord,
            "Grid coordinates other than the additive identity should not be equal when added to \
             themselves."
        );
    }

    prop_assert_eq!(
        coord - coord,
        add_unit,
        "Subtracting a coordinate from itself should yield the additive identity."
    );

    Ok(())
}

fn coord_add_comm<MC: ModuleCoord + Copy>(coord1: MC, coord2: MC) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
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

fn coord_add_assoc<MC: ModuleCoord + Copy>(
    coord1: MC,
    coord2: MC,
    coord3: MC,
) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
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

fn coord_sub_unit<MC: ModuleCoord + Copy>(coord: MC) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    let origin = MC::default();
    prop_assert_eq!(
        coord - origin,
        coord,
        "Subtracting the additive identity from a grid coordinate should yield the same \
         coordinate.",
    );

    prop_assert_eq!(
        coord - coord,
        origin,
        "Subtracting a coordinate from itself should yield the additive identity."
    );

    // Verify with SubAssign as well.
    let mut coord_copy = coord;
    coord_copy -= coord;
    prop_assert_eq!(
        coord_copy,
        origin,
        "Subtracting a coordinate from itself should yield the additive identity."
    );

    Ok(())
}

fn coord_sub_anticomm<MC: ModuleCoord + Copy>(coord1: MC, coord2: MC) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
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

fn coord_mul_unit<MC: ModuleCoord + Copy>(coord: MC) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    let origin = MC::default();
    prop_assert_eq!(
        coord * 0isize,
        origin,
        "Multiplying a grid coordinate by zero should yield the origin.",
    );

    prop_assert_eq!(
        coord * 1isize,
        coord,
        "Multiplying a grid coordinate by one should yield the same coordinate.",
    );

    Ok(())
}

fn coord_mul_assoc<MC: ModuleCoord + Copy>(
    coord: MC,
    x: isize,
    y: isize,
) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    prop_assert_eq!(
        coord * (x * y),
        (coord * x) * y,
        "Multiplying a grid coordinate should be associative.",
    );

    // Verify with MulAssign as well.
    let mut coord1 = coord;
    coord1 *= x * y;
    let mut coord2 = coord;
    coord2 *= x;
    coord2 *= y;
    prop_assert_eq!(
        coord1,
        coord2,
        "Multiplying a grid with assignment should be associative.",
    );

    Ok(())
}

fn coord_mul_distributive_coord<MC: ModuleCoord + Copy>(
    coord1: MC,
    coord2: MC,
    x: isize,
) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    prop_assert_eq!(
        (coord1 + coord2) * x,
        (coord1 * x) + (coord2 * x),
        "Multiplying a grid coordinate should be distributive over coordinates.",
    );

    Ok(())
}

fn coord_mul_distributive_ring<MC: ModuleCoord + Copy>(
    coord: MC,
    x: isize,
    y: isize,
) -> Result<(), TestCaseError>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    prop_assert_eq!(
        coord * (x + y),
        (coord * x) + (coord * y),
        "Multiplying a grid should be distributive over the ring.",
    );

    Ok(())
}

fn grid_path<C: Coord + Copy>(coord1: C, coord2: C) -> Result<(), TestCaseError> {
    let mut prev: Option<C> = None;
    let mut seen: HashSet<C> = HashSet::new();
    for coord in coord1.path_iterator(&coord2) {
        if let Some(prev_coord) = prev {
            prop_assert_ne!(
                prev_coord,
                coord,
                "Adjacent coordinates should be different."
            );
            prop_assert!(
                !seen.contains(&coord),
                "There should be no duplicate coordinates in the line."
            );
            prop_assert!(
                check_adjacent(prev_coord, coord),
                "It should be possible to move from {prev_coord} to {coord} via exactly one \
                 allowed face direction."
            );
        } else {
            prop_assert_eq!(
                coord,
                coord1,
                "The first coordinate in the line should be the start coordinate {}.",
                coord1
            );
        }
        prev = Some(coord);
        seen.insert(coord);
    }

    prop_assert_eq!(
        seen.len(),
        coord1.distance(&coord2) + 1,
        "The number of coordinates in the line should match the distance between the coordinates \
         (plus the initial coordinate)."
    );

    Ok(())
}

/// Helper function that tests that for given grid coordinate, that
/// moving in all allowed directions is possible, and that moving the
/// opposite direction returns to the original coordinate.
fn grid_direction<C: Coord + Copy>(coord: C, dir_type: DirectionType) -> Result<(), TestCaseError> {
    let mut seen_coords: HashSet<C> = HashSet::new();
    let mut seen_array_coords = HashSet::new();
    for dir in &coord.allowed_directions(dir_type) {
        prop_assert!(
            coord.allowed_direction(dir_type, dir),
            "{dir_type} direction {dir} should be allowed from coordinate {coord}"
        );
        let opt_moved_coord = coord.move_in_direction(dir_type, dir);
        prop_assert!(
            opt_moved_coord.is_some(),
            "{dir_type} direction {dir} should be allowed from coordinate {coord}"
        );
        let moved_coord = opt_moved_coord.unwrap();
        prop_assert_ne!(
            coord,
            moved_coord,
            "Moving in direction {} from {} should yield a different coordinate.",
            dir,
            coord
        );
        let back_dir = dir.opposite();
        let opt_returned_coord = moved_coord.move_in_direction(dir_type, back_dir);
        prop_assert!(
            opt_returned_coord.is_some(),
            "Moving in direction {dir} from {coord} to {moved_coord} and then back should be \
             allowed."
        );
        let returned_coord = opt_returned_coord.unwrap();
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
        prop_assert!(
            !seen_coords.contains(&moved_coord),
            "Moving in direction {dir} from {coord} should yield a unique coordinate from all \
             other allowed directions, but got duplicate {moved_coord}"
        );
        seen_coords.insert(moved_coord);

        let arrary_coord = moved_coord.grid_to_array_offset();
        prop_assert!(
            !seen_array_coords.contains(&arrary_coord),
            "Moving in direction {dir} from {coord} should yield a unique array coordinate from \
             all other allowed directions, but got duplicate array coordinate {:?}",
            arrary_coord
        );
        seen_array_coords.insert(arrary_coord);
    }
    // Check that for all directions that are not allowed, that moving that
    // direction is not possible.
    for dir in &(Direction::VALUES.difference(coord.allowed_directions(dir_type))) {
        prop_assert!(
            !coord.allowed_direction(dir_type, dir),
            "{dir_type} direction {dir} should not be allowed from coordinate {coord}"
        );
        let opt_moved_coord = coord.move_in_direction(dir_type, dir);
        prop_assert!(
            opt_moved_coord.is_none(),
            "{dir_type} direction {dir} should not be allowed from coordinate {coord}"
        );
    }
    Ok(())
}

fn grid_color<C: Coord + Copy>(coord: C) -> Result<(), TestCaseError> {
    let coord_color = coord.to_color();
    for dir in &coord.allowed_directions(DirectionType::Face) {
        let moved = coord
            .move_in_direction(DirectionType::Face, dir)
            .expect("Allowed direction should have an offset");
        let adjacent_color = moved.to_color();
        assert_ne!(
            coord_color, adjacent_color,
            "Adjacent coordinates should not have the same color: {} and {} are colored the same",
            coord, moved
        );
    }
    Ok(())
}

fn grid_rotation<C: Coord + Copy>(coord: C) -> Result<(), TestCaseError> {
    let rotated = coord.rotate_clockwise();
    let rotated_back = rotated.rotate_counterclockwise();
    prop_assert_eq!(
        coord,
        rotated_back,
        "Rotating clockwise and then counter-clockwise should be the identity."
    );

    let mut rotated = coord;
    let mut count = 0usize;
    loop {
        rotated = rotated.rotate_clockwise();
        count += 1;
        if rotated == coord {
            break;
        }
        prop_assert!(
            count < 100,
            "Rotating clockwise should eventually return to the original coordinate.  Still \
             iterating after 100 rotations."
        );
    }

    let mut rotated = coord;
    let mut count = 0usize;
    loop {
        rotated = rotated.rotate_counterclockwise();
        count += 1;
        if rotated == coord {
            break;
        }
        prop_assert!(
            count < 100,
            "Rotating counter-clockwise should eventually return to the original coordinate.  \
             Still iterating after 100 rotations."
        );
    }

    Ok(())
}

fn grid_reflection<C: Coord + Copy>(coord: C, axes: &[C::Axes]) -> Result<(), TestCaseError> {
    for axis in axes {
        let reflected = coord.reflect(*axis);

        // This would be a bit cleaner if we could check if a coordinate lies
        // directly on the line upon which we are reflecting, but this is
        // sufficient to verify that reflection is not vacuous.
        prop_assert!(
            reflected != coord
                || coord
                    .allowed_directions(DirectionType::Face)
                    .into_iter()
                    .any(|d| coord
                        .move_in_direction(DirectionType::Face, d)
                        .unwrap()
                        .reflect(*axis)
                        != coord),
            "Reflecting across the {} axis should yield a different coordinate, or there will be \
             an adjacent coordinate that is different itself when reflected.",
            axis
        );

        let reflected_back = reflected.reflect(*axis);
        prop_assert_eq!(
            coord,
            reflected_back,
            "Reflecting twice across the {} axis should be the identity.",
            axis
        );
    }

    let mut reflected = coord;
    let mut count = 0usize;
    for axis in axes.iter().cycle() {
        reflected = reflected.reflect(*axis);
        if reflected == coord {
            break;
        }
        count += 1;
        prop_assert!(
            count < 100,
            "Reflecting across all axes should eventually return to the original coordinate.  \
             Still iterating after 100 reflections."
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
        //  "Direction {dir} should be returned for angle {angle} from
        // coordinate {coord}");
    }
    for dir in &(Direction::VALUES.difference(coord.allowed_directions(dir_type))) {
        let direction = coord.direction_angle(dir_type, dir);
        prop_assert!(
            direction.is_none(),
            "No direction should be returned for disallowed direction {dir} from coordinate \
             {coord}"
        );
    }

    Ok(())
}

fn grid_direction_iterator<C: Coord + Copy>(
    coord: C,
    dir_type: DirectionType,
) -> Result<(), TestCaseError> {
    // Disallow directions will produce empty iterators.
    for dir in &(Direction::VALUES.difference(coord.allowed_directions(dir_type))) {
        let iter = coord.direction_iterator(dir_type, dir, ..);
        prop_assert!(
            iter.count() == 0,
            "Iterator for disallowed {dir_type} direction {dir} should be empty"
        );
    }

    for dir in &coord.allowed_directions(dir_type) {
        let inclusive_iter = coord.direction_iterator(dir_type, dir, ..=10);
        let inclusive_count = (0..=10).count();
        // Ensure that all elements in the iterator have the correct offset
        // relationship.
        let mut distance = None;
        let mut seen_coords: HashSet<C> = HashSet::new();
        for (index, c) in inclusive_iter.enumerate().skip(1) {
            prop_assert!(
                !seen_coords.contains(&c),
                "All coordinates in the direction iterator should be unique."
            );
            seen_coords.insert(c);
            prop_assert!(index <= inclusive_count, "Iterator has unexpected length");
            let new_distance = coord.distance(&c);
            if let Some(dist) = distance {
                prop_assert!(
                    new_distance > dist,
                    "As we iterate in a direction, we should get further away from the original \
                     coordinate."
                );
            }
            distance = Some(new_distance);
        }
        prop_assert!(
            seen_coords.len() == inclusive_count - 1,
            "Iterator has unexpected length"
        );

        // Ensure that the inclusive iterator has the correct length.
        prop_assert_eq!(
            coord.direction_iterator(dir_type, dir, ..=10).count(),
            inclusive_count
        );

        // Ensure that the exclusive iterator has the correct length.
        let exclusive_iter = coord.direction_iterator(dir_type, dir, ..10);
        prop_assert_eq!(exclusive_iter.count(), (0..10).count());

        // For allowed directions, the first element of the iterator will always be
        // the coordinate itself, even if the range is empty.
        let unbounded_iter = coord.direction_iterator(dir_type, dir, ..);
        prop_assert!(
            unbounded_iter.take(1).collect::<Vec<_>>().as_slice() == [coord],
            "For inclusive range, first element of iterator for {dir_type} direction {dir} should \
             be the coordinate itself."
        );
    }

    Ok(())
}

fn grid_axis_iterator<C: Coord + Copy>(coord: C, axes: &[C::Axes]) -> Result<(), TestCaseError> {
    for axis in axes {
        for sign in [false, true] {
            let axis_coord = coord.move_on_axis(*axis, sign);
            prop_assert!(
                check_adjacent(coord, axis_coord),
                "Moving on axis {} with sign {} should correspond to moving to exactly one \
                 adjacent coordinate.",
                axis,
                sign
            );

            let mut prev_coord = None;
            let inclusive_count = (0..=10).count();
            let mut seen_coords: HashSet<C> = HashSet::new();
            let mut distance = None;
            for axis_coord in coord.axis_iterator(*axis, sign, ..10) {
                if let Some(prev) = prev_coord {
                    prop_assert!(
                        check_adjacent(prev, axis_coord),
                        "All coordinates in the axis iterator should be adjacent."
                    );
                }
                prev_coord = Some(axis_coord);

                prop_assert!(
                    !seen_coords.contains(&axis_coord),
                    "All coordinates in the axis iterator should be unique."
                );
                seen_coords.insert(axis_coord);
                prop_assert!(
                    seen_coords.len() <= inclusive_count,
                    "Iterator has unexpected length"
                );
                let new_distance = coord.distance(&axis_coord);
                if let Some(dist) = distance {
                    prop_assert!(
                        new_distance > dist,
                        "As we iterate along an axis, we should get further away from the \
                         original coordinate."
                    );
                }
                distance = Some(new_distance);
            }

            prop_assert_eq!(
                coord.axis_iterator(*axis, sign, ..=10).count(),
                inclusive_count
            );
        }
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
    let screen_coord = sized_grid.grid_to_screen(&coord);
    prop_assert!(
        sized_grid.coord_contains(&coord, screen_coord),
        "Center of coordinate {coord} in screen space is not contained within the coordinate: \
         {screen_coord}"
    );
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
    let screen_coord = sized_grid.grid_to_screen(&coord);

    for dir in &coord.allowed_directions(dir_type) {
        let opt_moved_coord = coord.move_in_direction(dir_type, dir);
        prop_assert!(
            opt_moved_coord.is_some(),
            "Direction {dir} should be allowed from coordinate {coord}"
        );
        let moved_coord = opt_moved_coord.unwrap();
        let moved_screen_coord = sized_grid.grid_to_screen(&moved_coord);
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
            "Moved from {} (screen {}) via {} direction (angle {}) to {} (screen {}) but got back \
             to {} via {} (angle {}).",
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
    coord: &SG::Coord,
    expected_size: usize,
) -> Result<(), TestCaseError> {
    let vertices = sized_grid.vertices(coord);
    prop_assert_eq!(
        vertices.len(),
        expected_size,
        "Grid coordinate {} should have {} vertices, but got {}.",
        coord,
        expected_size,
        vertices.len()
    );
    let mut current_angle = 0.0;
    let mut prev_vertex = None;
    for (index, vertex) in vertices.iter().enumerate() {
        // Check that the edges are of the correct length.
        if let Some(prev_vertex) = prev_vertex {
            let edge_vector: Vec2 = *vertex - prev_vertex;
            let edge_vector_len = edge_vector.length();
            let diff = (sized_grid.edge_length() - edge_vector_len).abs();
            prop_assert!(
                // TODO allowing 2% error seems a bit high, but maybe that is the
                // best we can expect with f32 and extreme grid sizes?
                (diff / edge_vector_len) < 0.02,
                "The length between the vertices does not match the expected length: {} vs {} \
                 diff {}",
                sized_grid.edge_length(),
                edge_vector_len,
                diff,
            );
        }

        // Need to measure the angles relative to the center of the cell, otherwise
        // the vertices will not appear to be in clockwise order.
        let vertex_angle = (vertex - sized_grid.grid_to_screen(coord))
            .normalize()
            .to_angle()
            .rem_euclid(2.0 * PI);
        prop_assert!(
            vertex_angle >= current_angle,
            "Vertex angles are not in clockwise order: for vertex {index} of {coord}, \
             {vertex_angle} < {current_angle}"
        );
        prev_vertex = Some(vertex.clone());
        current_angle = vertex_angle;
    }

    Ok(())
}

fn grid_shapes(kind: dynamic::Kind) -> Result<(), TestCaseError> {
    for size in 0..10 {
        let ring = dynamic::Coord::ring(kind, size);
        prop_assert!(
            !ring.is_empty(),
            "A ring of size {size} should not be empty."
        );
        if size > 0 {
            let range = dynamic::Coord::range(kind, size - 1);
            prop_assert!(
                !range.is_empty(),
                "A range of size {size} should not be empty.",
            );
            prop_assert!(
                range.is_disjoint(&ring),
                "The ring of size {} should be disjoint from the range of size {}",
                size,
                size - 1
            );
        }
    }

    Ok(())
}

static SIZE_RANGE: std::ops::Range<f32> = 0.001..65535.0f32;

proptest! {
    #[test]
    fn hex_unary_op(coord1 in hexcoord_strategy()) {
        coord_neg(coord1)?;
        coord_add_ident(coord1)?;
        coord_sub_unit(coord1)?;
        coord_mul_unit(coord1)?;

        // Verify that the origin is the same as the default coordinate.
        prop_assert_eq!(dynamic::Coord::Hex(hex::Coord::default()),
        dynamic::Coord::origin(dynamic::Kind::Hex))
    }

    #[test]
    fn square_unary_op(coord1 in squarecoord_strategy()) {
        coord_neg(coord1)?;
        coord_add_ident(coord1)?;
        coord_sub_unit(coord1)?;
        coord_mul_unit(coord1)?;

        // Verify that the origin is the same as the default coordinate.
        prop_assert_eq!(dynamic::Coord::Square(square::Coord::default()),
        dynamic::Coord::origin(dynamic::Kind::Square))
    }

    #[test]
    fn hex_mul_assoc_distrib(coord in hexcoord_strategy(), x in -100..100isize, y in -100..100isize) {
        coord_mul_assoc(coord, x, y)?;
        coord_mul_distributive_ring(coord, x, y)?;
    }

    #[test]
    fn hex_mul_distrib(coord1 in hexcoord_strategy(), coord2 in hexcoord_strategy(), x in -100..100isize) {
        coord_mul_distributive_coord(coord1, coord2, x)?;
    }

    #[test]
    fn square_mul_assoc_distrib(coord in squarecoord_strategy(), x in -100..100isize, y in -100..100isize) {
        coord_mul_assoc(coord, x, y)?;
        coord_mul_distributive_ring(coord, x, y)?;
    }

    #[test]
    fn square_mul_distrib(coord1 in squarecoord_strategy(), coord2 in squarecoord_strategy(), x in -100..100isize) {
        coord_mul_distributive_coord(coord1, coord2, x)?;
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

        // Verify that the origin is the same as the default coordinate.
        prop_assert_eq!(dynamic::Coord::Triangle(triangle::Coord::default()),
        dynamic::Coord::origin(dynamic::Kind::Triangle))
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
    fn hex_assoc(coord1 in hexcoord_strategy(), coord2 in hexcoord_strategy(), coord3 in hexcoord_strategy()) {
        coord_add_assoc(coord1, coord2, coord3)?;
    }

    #[test]
    fn square_assoc(coord1 in squarecoord_strategy(), coord2 in squarecoord_strategy(), coord3 in squarecoord_strategy()) {
        coord_add_assoc(coord1, coord2, coord3)?;
    }

    #[test]
    fn test_grid_color(coord in dynamic_coord_strategy()) {
        grid_color(coord)?
    }

    #[test]
    fn test_grid_rotation(coord in dynamic_coord_strategy()) {
        grid_rotation(coord)?
    }

    #[test]
    fn test_grid_reflection(coord in dynamic_coord_strategy()) {
        grid_reflection(coord,coord.kind().axes().as_slice())?
    }

    #[test]
    fn test_grid_direction(coord in dynamic_coord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_direction(coord, dt)?;
            grid_angle_to_direction(coord, dt)?;
        }
    }

    #[test]
    fn test_grid_direction_iterator(coord in small_dynamic_coord_strategy()) {
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            grid_direction_iterator(coord, dt)?;
        }
    }

    #[test]
    fn test_grid_axis_iterator(coord in small_dynamic_coord_strategy()) {
        grid_axis_iterator(coord, coord.kind().axes().as_slice())?;
    }

    #[test]
    fn hex_grid_to_array_offset(coord in hexcoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, hex::Coord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);

        let dyn_coord = dynamic::Coord::Hex(coord);
        prop_assert_eq!(dyn_coord.grid_to_array_offset(), array_offset);
    }

    #[test]
    fn square_grid_to_array_offset(coord in squarecoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, square::Coord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);

        let dyn_coord = dynamic::Coord::Square(coord);
        prop_assert_eq!(dyn_coord.grid_to_array_offset(), array_offset);
    }

    #[test]
    fn triangle_grid_to_array_offset(coord in trianglecoord_strategy()) {
        let array_offset = coord.grid_to_array_offset();
        prop_assert_eq!(coord, triangle::Coord::array_offset_to_grid(array_offset),
        "With array offset {:?}", array_offset);

        let dyn_coord = dynamic::Coord::Triangle(coord);
        prop_assert_eq!(dyn_coord.grid_to_array_offset(), array_offset);
    }

    #[test]
    fn test_path(coord1 in small_dynamic_coord_strategy(), coord2 in small_dynamic_coord_strategy()) {
        prop_assume!(coord1.kind() == coord2.kind(), "Coordinates should be of the same kind.");
        grid_path(coord1, coord2)?;
    }

    #[test]
    fn test_sized_grid_commutation(size in &SIZE_RANGE,
        coord in dynamic_coord_strategy()) {
        let sized_grid = dynamic::SizedGrid::new(coord.kind(), size);
        prop_assert_eq!(sized_grid.kind(), coord.kind(),
            "Sized grid kind should match coordinate kind.");
        let fudge = match coord {
            dynamic::Coord::Hex(_) => sized_grid.edge_length(),
            _ => 0.0,
        };
        for dt in [DirectionType::Face, DirectionType::Vertex] {
            sized_grid_commutation(sized_grid, coord, dt, fudge)?;
        }
        sized_grid_radius(sized_grid)?;
        sized_grid_identity(sized_grid, coord)?;
    }

    #[test]
    fn sized_vertices(size in &SIZE_RANGE,
        coord in dynamic_coord_strategy()) {
        let kind = coord.kind();
        sized_grid_vertices(dynamic::SizedGrid::new(kind, size), &coord, kind.num_vertices())?;
    }

    #[test]
    fn shapes(kind in kind_strategy()) {
        grid_shapes(kind)?;
    }
}

#[test]
fn triangle_point() {
    assert_ne!(TrianglePoint::Up, !TrianglePoint::Up);
}

#[test]
fn test_color_try_from_usize_success() {
    use endgame_grid::Color::*;
    let cases = [(1, One), (2, Two), (3, Three), (4, Four)];
    for (value, expected) in cases {
        let got = endgame_grid::Color::try_from(value).expect("Expected Ok for value {value}");
        assert_eq!(got, expected, "Color::try_from({value}) mismatch");
    }
}

#[test]
fn test_color_try_from_usize_failure() {
    use endgame_grid::Color;
    // A sampling of invalid inputs, including 0 and values greater than 4.
    let cases = [0, 5, 6, 100];
    for value in cases {
        let err = Color::try_from(value).expect_err("Expected Err for value {value}");
        assert!(
            err.contains("1..=4") && err.contains(&value.to_string()),
            "Error message should mention range and value. Got: {err}"
        );
    }
}

#[test]
fn test_color_display() {
    use endgame_grid::Color::*;
    let cases = [(One, "One"), (Two, "Two"), (Three, "Three"), (Four, "Four")];
    for (color, expected) in cases {
        let s = format!("{color}");
        assert_eq!(s, expected, "Display mismatch for {:?}", color);
    }
}
