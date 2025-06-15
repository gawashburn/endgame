use endgame_direction::{Direction, DirectionSet};

#[test]
fn direction_containment() {
    let all_directions = Direction::VALUES;
    // Verify that the cardinal and ordinal directions are subsets of the
    // complete set of directions.
    for (name, dirs) in vec![
        ("Cardinal", Direction::CARDINAL),
        ("Ordinal", Direction::ORDINAL),
    ] {
        assert!(!dirs.superset(all_directions),
                "The {name} directions ({dirs}) should not be a superset of all directions ({all_directions}).");
        assert!(all_directions.superset(dirs),
             "All directions ({all_directions}) are not a superset the {name} directions ({dirs})");
        assert!(dirs.subset(all_directions),
                "The {name} directions ({dirs}) are not a subset of all directions ({all_directions}).");
        assert!(!all_directions.subset(dirs),
                "The {name} directions ({dirs}) should not be a subset of all directions ({all_directions}).")
    }

    // Verify that the cardinal and ordinal directions are disjoint.
    assert!(
        Direction::CARDINAL.intersection(Direction::ORDINAL).is_empty(),
        "Cardinal and Ordinal directions are not disjoint."
    );
}

#[test]
fn test_is_ordinal() {
    for dir in &Direction::ORDINAL {
            assert!(dir.is_ordinal(),
                "Ordinal direction {dir} is not in the ordinal set.");
            assert!(!dir.is_cardinal(),
                "Non-ordinal direction {dir} is incorrectly in the ordinal set.");
    }
}

#[test]
fn test_is_cardinal() {
    for dir in &Direction::CARDINAL {
        assert!(dir.is_cardinal(),
            "Cardinal direction {dir} is not in the cardinal set.");
        assert!(!dir.is_ordinal(),
            "Non-cardinal direction {dir} is incorrectly in the cardinal set.");
    }
}

#[test]
fn test_containment() {
    for (name, dirs) in vec![
        ("Cardinal", Direction::CARDINAL),
        ("Ordinal", Direction::ORDINAL),
    ] {
        for dir in &dirs {
            assert!(
                dirs.contains(dir),
                "Direction {dir} in set {name} ({dirs}) via iterator, but not by containment.");
        }
    }
}
    
#[test]
fn test_direction_rotation() {
    for dir in &Direction::VALUES {
        assert_eq!(
            dir,
            dir.clockwise().counter_clockwise(),
            "clockwise and counter-clockwise on {} are not inverses.",
            dir
        );
    }
}

#[test]
fn test_direction_invertibility()  {  
    for dir in &Direction::VALUES {
        assert_eq!(
            dir, !!dir,
            "opposite operation on {} is not invertible.",
            dir
        );
    }
}

#[test]
fn test_from_slice() {
    let slice = [Direction::North, Direction::East, Direction::South, Direction::West];
    let dir_set = DirectionSet::from_slice(&slice);
    assert!(!dir_set.is_empty(),
            "The set {dir_set} should not be empty.");
    assert!(dir_set.contains(Direction::North));
    assert!(dir_set.contains(Direction::East));
    assert!(dir_set.contains(Direction::South));
    assert!(dir_set.contains(Direction::West));
    assert!(!dir_set.contains(Direction::NorthEast));

    let slice = [Direction::West, Direction::West];
    let dir_set = DirectionSet::from_slice(&slice);
    assert!(dir_set.contains(Direction::West));
    assert_eq!(dir_set.len(), 1, "The set {dir_set} should just contain West.");
    
    for dir in &Direction::VALUES {
        let dir_set = DirectionSet::from_slice(&[dir]);
        assert!(dir_set.contains(dir), "Direction {dir} should be in the set.");
    }
    
    assert!(DirectionSet::from_slice(&[]).is_empty(),
        "DirectionSet from empty slice should be empty.");
}

#[test]
fn test_iteration() {
    let dir_set = Direction::VALUES;
    let mut iter = dir_set.iter();
    let mut count = 0;
    for dir in &Direction::VALUES {
        assert_eq!(iter.next(), Some(dir));
        count += 1;
        assert!(count <= dir_set.len(), "Iterated over too many directions.");
    }
    assert_eq!(count, dir_set.len(), "Iterator did not iterate over all directions.");
    assert_eq!(iter.next(), None, "Iterator did not end after all directions.");
}

#[test]
fn test_direction_set_formatting() {
    let dir_set = Direction::VALUES;
    let formatted = format!("{dir_set}");
    assert_eq!(formatted, "{East, NorthEast, North, NorthWest, West, SouthWest, South, SouthEast}",
               "Formatted direction set does not match expected output.");
    let dir_set = DirectionSet::empty();
    let formatted = format!("{dir_set}");
    assert_eq!(formatted, "{}", 
               "Formatted direction set does not match expected output.");
}

#[test]
fn test_direction_angles() {
    use std::f32::consts::PI;

    let mut angle = 0.0;
    for dir in &Direction::VALUES {
        // TODO What is really an acceptable error here?
        assert!((dir.angle() - angle).abs() < (8.0 * f32::EPSILON),
            "Direction {dir} has unexpected angle: difference between \
            {} vs {angle} is greater than {}",
                dir.angle(), 8.0 * f32::EPSILON);
        angle += PI / 4.0;
    }
}