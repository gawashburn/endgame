use endgame_direction::{Direction, DirectionSet};
use itertools::Itertools;
use std::borrow::Borrow;

#[test]
fn directionset_borrow() {
    for dirs_vec1 in Direction::VALUES.iter().powerset() {
        for dirs_vec2 in Direction::VALUES.iter().powerset() {
            let dirs1 = DirectionSet::from_iter(dirs_vec1.iter().cloned());
            let dirs2 = DirectionSet::from_iter(dirs_vec2.iter().cloned());
            assert!(
                dirs1 != dirs2 || dirs1.borrow() == dirs2.borrow(),
                "Borrow condition not satisfied: {:?} == {:?} but {:?} != {:?}",
                dirs1,
                dirs2,
                dirs1.borrow(),
                dirs2.borrow()
            );
        }
    }
}

#[test]
fn direction_containment() {
    let all_directions = Direction::VALUES;
    // Verify that the cardinal and ordinal directions are subsets of the
    // complete set of directions.
    for (name, dirs) in vec![
        ("Cardinal", Direction::CARDINAL),
        ("Ordinal", Direction::ORDINAL),
    ] {
        assert!(
            !dirs.is_superset(all_directions),
            "The {name} directions ({dirs}) should not be a superset of all directions ({all_directions})."
        );
        assert!(
            all_directions.is_superset(dirs),
            "All directions ({all_directions}) are not a superset the {name} directions ({dirs})"
        );
        assert!(
            dirs.is_subset(all_directions),
            "The {name} directions ({dirs}) are not a subset of all directions ({all_directions})."
        );
        assert!(
            !all_directions.is_subset(dirs),
            "The {name} directions ({dirs}) should not be a subset of all directions ({all_directions})."
        )
    }

    // Verify that the cardinal and ordinal directions are disjoint.
    assert!(
        Direction::CARDINAL
            .intersection(Direction::ORDINAL)
            .is_empty(),
        "Cardinal and Ordinal directions are not disjoint."
    );

    // Check the set differencing over cardinal and ordinal directions behaves
    // as expected.
    assert_eq!(
        Direction::VALUES.difference(Direction::CARDINAL),
        *Direction::ORDINAL,
        "Removing the cardinal directions from all directions should yield the ordinal directions."
    );
    assert_eq!(
        Direction::VALUES.difference(Direction::ORDINAL),
        *Direction::CARDINAL,
        "Removing the ordinal directions from all directions should yield the cardinal directions."
    );

    assert_eq!(
        *Direction::VALUES,
        Direction::CARDINAL.union(Direction::ORDINAL),
        "Union of the cardinal and ordinal directions should be the same as all directions.")
}

#[test]
fn test_is_ordinal() {
    for dir in Direction::ORDINAL {
        assert!(
            dir.is_ordinal(),
            "Ordinal direction {dir} is not in the ordinal set."
        );
        assert!(
            !dir.is_cardinal(),
            "Non-ordinal direction {dir} is incorrectly in the ordinal set."
        );
    }
}

#[test]
fn test_is_cardinal() {
    for dir in Direction::CARDINAL {
        assert!(
            dir.is_cardinal(),
            "Cardinal direction {dir} is not in the cardinal set."
        );
        assert!(
            !dir.is_ordinal(),
            "Non-cardinal direction {dir} is incorrectly in the cardinal set."
        );
    }
}

#[test]
fn test_containment() {
    for (name, dirs) in vec![
        ("Cardinal", Direction::CARDINAL),
        ("Ordinal", Direction::ORDINAL),
    ] {
        for dir in dirs {
            assert!(
                dirs.contains(dir),
                "Direction {dir} in set {name} ({dirs}) via iterator, but not by containment."
            );
        }
    }
}

#[test]
fn test_direction_rotation() {
    for dir in Direction::VALUES {
        assert_eq!(
            dir,
            dir.clockwise().counter_clockwise(),
            "clockwise and counter-clockwise on {} are not inverses.",
            dir
        );

        assert_eq!(
            dir.rotate(1),
            dir.clockwise(),
            "Rotating by a single step should be equivalent to rotating clockwise.",
        );

        assert_eq!(
            dir.rotate(-1),
            dir.counter_clockwise(),
            "Rotating by negative step should be equivalent to rotating counter-clockwise.",
        );

        assert_eq!(
            dir.rotate(8),
            dir,
            "Rotating by eight steps should be an identity.",
        );

        assert_eq!(
            dir.rotate(-8),
            dir,
            "Rotating by negative eight steps should be an identity.",
        );
    }
}

#[test]
fn test_direction_invertibility() {
    for dir in Direction::VALUES {
        assert_eq!(
            dir, !!dir,
            "opposite operation on {} is not invertible.",
            dir
        );
    }
}

#[test]
fn test_from_slice() {
    let slice = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
    let dir_set = DirectionSet::from_slice(&slice);
    assert!(
        !dir_set.is_empty(),
        "The set {dir_set} should not be empty."
    );
    assert!(dir_set.contains(Direction::North));
    assert!(dir_set.contains(Direction::East));
    assert!(dir_set.contains(Direction::South));
    assert!(dir_set.contains(Direction::West));
    assert!(!dir_set.contains(Direction::NorthEast));

    let dir_set = dir_set.difference(Direction::CARDINAL);
    assert!(
        dir_set.is_empty(),
        "The set {dir_set} should be empty after removing all cardinal directions."
    );

    let slice = [Direction::West, Direction::West];
    let dir_set = DirectionSet::from_slice(&slice);
    assert!(dir_set.contains(Direction::West));
    assert_eq!(
        dir_set.len(),
        1,
        "The set {dir_set} should just contain West."
    );

    for dir in Direction::VALUES {
        let dir_set = DirectionSet::from_slice(&[dir]);
        assert!(
            dir_set.contains(dir),
            "Direction {dir} should be in the set."
        );
    }

    assert!(
        DirectionSet::from_slice(&[]).is_empty(),
        "DirectionSet from empty slice should be empty."
    );
}

#[test]
fn test_insert_and_remove() {
    let mut dir_set = DirectionSet::new();
    assert!(dir_set.is_empty(), "New direction set should be empty.");

    for dir in Direction::VALUES {
        assert!(
            !dir_set.contains(dir),
            "Direction {dir} should not be in the set: {dir_set}."
        );
        assert!(
            dir_set.insert(dir),
            "Direction {dir} should not be in the set: {dir_set}."
        );
        assert!(
            dir_set.contains(dir),
            "Direction {dir} should now be in the set: {dir_set}."
        );
    }

    for dir in Direction::VALUES {
        assert!(
            dir_set.contains(dir),
            "Direction {dir} should be in the set before removal: {dir_set}."
        );
        assert!(
            dir_set.remove(dir),
            "Direction {dir} should be in the set before removal: {dir_set}."
        );
        assert!(
            !dir_set.contains(dir),
            "Direction {dir} should no longer be in the set: {dir_set}."
        );
    }

    assert!(
        dir_set.is_empty(),
        "Direction set should be empty after removing all directions."
    );
}

#[test]
fn test_iteration() {
    for (name, dirs) in vec![
        ("all", Direction::VALUES),
        ("cardinal", Direction::CARDINAL),
        ("ordinal", Direction::ORDINAL),
    ] {
        let mut dir_set = DirectionSet::new();

        let mut iter = dirs.iter();
        let mut count = 0;
        for dir in dirs {
            assert_eq!(iter.next(), Some(dir));
            count += 1;
            assert!(
                dir_set.insert(dir),
                "Direction {dir} should not have already been in the set: {dir_set}."
            );
            assert!(
                count <= dirs.len(),
                "Iterated over too many directions in {} directions.",
                name
            );
        }
        assert_eq!(
            *dirs, dir_set,
            "Iterator did not iterate over all {} directions.",
            name
        );
        assert_eq!(
            iter.next(),
            None,
            "Iterator did not end after all {} directions.",
            name
        );
    }
}

#[test]
fn test_direction_set_formatting() {
    let dir_set = Direction::VALUES;
    let formatted = format!("{dir_set}");
    assert_eq!(
        formatted, "{East, NorthEast, North, NorthWest, West, SouthWest, South, SouthEast}",
        "Formatted direction set does not match expected output."
    );
    let dir_set = DirectionSet::new();
    let formatted = format!("{dir_set}");
    assert_eq!(
        formatted, "{}",
        "Formatted direction set does not match expected output."
    );
}

#[test]
fn test_direction_angles() {
    use std::f32::consts::PI;

    let mut angle = 0.0;
    for dir in Direction::VALUES {
        // TODO What is really an acceptable error here?
        assert!(
            (dir.angle() - angle).abs() < (8.0 * f32::EPSILON),
            "Direction {dir} has unexpected angle: difference between \
            {} vs {angle} is greater than {}",
            dir.angle(),
            8.0 * f32::EPSILON
        );
        angle += PI / 4.0;
    }
}


#[test]
fn test_direction_short_name_mappings() {
    use Direction::*;
    let cases = [
        (East, "E"),
        (NorthEast, "NE"),
        (North, "N"),
        (NorthWest, "NW"),
        (West, "W"),
        (SouthWest, "SW"),
        (South, "S"),
        (SouthEast, "SE"),
    ];
    for (dir, expected) in cases {
        assert_eq!(dir.short_name(), expected, "short_name mismatch for {dir}");
    }
}

#[test]
fn test_direction_parse_success() {
    use Direction::*;

    let success_cases: &[(&str, Direction)] = &[
        // Single-letter cardinal abbreviations
        ("e", East),
        ("n", North),
        ("w", West),
        ("s", South),
        // Two-letter ordinal abbreviations (mixed case)
        ("ne", NorthEast),
        ("NW", NorthWest),
        ("Se", SouthEast),
        ("sW", SouthWest),
        // Full names
        ("east", East),
        ("north", North),
        ("west", West),
        ("south", South),
        // Ordinals with separators 
        ("north-east", NorthEast),
        ("north_east", NorthEast),
        ("south-west", SouthWest),
        ("south_west", SouthWest),
        ("south west", SouthWest),
        ("north east", NorthEast),
    ];

    for (input, expected) in success_cases {
        let parsed = Direction::parse(input);
        assert_eq!(parsed, Some(*expected), "parse failed for input '{input}'.");
    }
}

#[test]
fn test_direction_parse_failure() {
    // Choose inputs that should not match any direction according to current regexes
    let failure_cases = [
        // Empty and ASCII whitespace-only.
        "",
        " ",
        "\t\t",
        // Non-ASCII/Unicode whitespace-only (should trim to empty or not match)
        "\u{00A0}", // NO-BREAK SPACE
        "\u{2002}", // EN SPACE
        "\u{2003}", // EM SPACE
        "\u{2009}", // THIN SPACE
        "\u{200B}", // ZERO WIDTH SPACE 
        "\u{3000}", // IDEOGRAPHIC SPACE
        // Clearly invalid strings.
        "123",
        "foo",
        // Invalid separator usage with spaces around hyphen (ASCII and Unicode)
        "east- west",         // ASCII space after hyphen is not allowed by the regex
        "north-\u{00A0}east", // NBSP after hyphen should not match
        "north\u{00A0}-east", // NBSP before hyphen should not match
        // Zero width joiners/spaces inserted between words should not match
        "north\u{200B}east",
        "south\u{200B}west",
        // Leading/trailing zero width space around otherwise valid tokens
        "north-east\u{200B}",
        "\u{200B}north-east",
        // Double underscore
        "north__east",
        "-ne",
        "se-",
        "_sw",
    ];

    for input in failure_cases {
        let parsed = Direction::parse(input);
        assert!(parsed.is_none(), "Unexpected parse success for input '{input}': {:?}.", parsed);
    }
}
