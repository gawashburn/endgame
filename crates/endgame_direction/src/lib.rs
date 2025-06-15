//! A simple implementation of cardinal and ordinal directions, as the
//! canonical 'direction' crate bakes in some notions of coordinates that
//! seem better separated to allow for different grid systems.
use bitset_core::BitSet;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

//////////////////////////////////////////////////////////////////////////////

/// An enumeration of compass directions.  The traditional "cardinal" directions,
/// along the "ordinal" ones as well.
#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    // Use a counter-clockwise ordering starting at East, so that
    // it aligns well with radian angles.
    East = 0,
    NorthEast = 1,
    North = 2,
    NorthWest = 3,
    West = 4,
    SouthWest = 5,
    South = 6,
    SouthEast = 7,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Direction::*;
        let name = match self {
            East => "East",
            NorthEast => "NorthEast",
            North => "North",
            NorthWest => "NorthWest",
            West => "West",
            SouthWest => "SouthWest",
            South => "South",
            SouthEast => "SouthEast",
        };
        write!(f, "{}", name)
    }
}

impl Direction {
    /// A reference to the set of all possible directions.
    pub const VALUES: DirectionSet  = DirectionSet(0b11111111);

    /// A reference to the set of cardinal directions.
    pub const CARDINAL: DirectionSet = DirectionSet(0b01010101);

    /// A reference to the set of ordinal directions.
    pub const ORDINAL: DirectionSet = DirectionSet(0b10101010);

    /// Is this a cardinal direction?
    pub fn is_cardinal(self) -> bool {
        Direction::CARDINAL.0.bit_test(self as usize)
    }

    /// Is this an ordinal direction?
    pub fn is_ordinal(self) -> bool {
        Direction::ORDINAL.0.bit_test(self as usize)
    }

    fn from_u8(value: u8) -> Direction {
        use Direction::*;
        match value {
            0 => East,
            1 => NorthEast,
            2 => North,
            3 => NorthWest,
            4 => West,
            5 => SouthWest,
            6 => South,
            7 => SouthEast,
            _ => panic!("Invalid direction value: {}", value),
        }
    }

    /// Produce the Direction clockwise from this Direction.
    pub fn clockwise(self) -> Direction {
        Direction::from_u8((self as u8).overflowing_sub(1).0 % 8)
    }

    /// Produce the Direction counter-clockwise from this Direction.
    pub fn counter_clockwise(self) -> Direction {
        Direction::from_u8((self as u8).overflowing_add(1).0 % 8)
    }

    /// The opposite Direction from this Direction.
    pub fn opposite(self) -> Direction {
        Direction::from_u8(((self as u8) + 4) % 8)
    }

    /// The angle of the direction in radians.
    pub fn angle(self) -> f32 {
        (self as u8 as f32) * (std::f32::consts::PI / 4.0)
    }
}

impl std::ops::Not for Direction {
    type Output = Direction;
    fn not(self) -> Direction {
        self.opposite()
    }
}

//////////////////////////////////////////////////////////////////////////////

/// A structure representing a set of directions.  Conveniently, this `Direction`
/// only models eight possible directions, so it is possible to efficiently
/// represent a set of directions as bitset with just a single byte.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectionSet(u8);

impl Display for DirectionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        // TODO It seems like this should be a common enough
        // case for there to be something in the standard library?
        for (index, dir) in self.iter().enumerate() {
            if index != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", dir)?;
        }
        write!(f, "}}")
    }
}

pub struct DirectionSetIter<'a> {
    set: &'a DirectionSet,
    index: u8,
}

impl<'a> DirectionSetIter<'a> {
    fn new(set: &'a DirectionSet) -> Self {
        DirectionSetIter { set, index: 0 }
    }

    /// Position the iterator at the next set bit.
    fn position(&mut self) {
        while self.index < 8 && !self.set.0.bit_test(self.index as usize) {
            self.index += 1;
        }
    }
}

impl Iterator for DirectionSetIter<'_> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.index <= 8);
        self.position();
        if self.index == 8 {
            return None;
        }
        let dir = Direction::from_u8(self.index);
        self.index += 1;
        Some(dir)
    }
}

// TODO IntoIterator from a DirectionSet value?
impl<'a> IntoIterator for &'a DirectionSet {
    type Item = Direction;
    type IntoIter = DirectionSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DirectionSetIter::new(&self)
    }
}

impl DirectionSet {
    /// Create a `DirectionSet` from a slice of `Direction`s.
    pub const fn from_slice(dirs: &[Direction]) -> Self {
        // Annoyingly to make this a constant function it was necessary
        // to write the bit operations directly instead of using BitSet
        // implementations
        let mut v = 0u8;
        let mut index = 0usize;
        while index < dirs.len() {
            v |= 1 << dirs[index] as usize;
            index += 1;
        }
        DirectionSet(v)
    }
    
    /// Create an empty `DirectionSet`.
    pub const fn empty() -> DirectionSet {
        DirectionSet(0)
    }
    
    /// Is this `DirectionSet` a superset of the other?
    pub fn superset(&self, other: DirectionSet) -> bool {
        self.0.bit_superset(&other.0)
    }
    
    /// Is this 1DirectionSet1 a subset of the other?
    pub fn subset(&self, other: DirectionSet) -> bool {
        self.0.bit_subset(&other.0)
    }
    
    /// Return the intersection of this `DirectionSet` with another.
    pub fn intersection(&self, other: DirectionSet) -> DirectionSet {
        let mut v = self.0;
        DirectionSet(*v.bit_and(&other.0))
    }
    
    /// Return the union of this `DirectionSet` with another.
    pub fn union(&self, other: DirectionSet) -> DirectionSet {
        let mut v = self.0;
        DirectionSet(*v.bit_or(&other.0))
    }

    /// Return the number of `Direction`s in the set.
    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns true if the set of `Direction`s is empty.
    pub fn is_empty(&self) -> bool {
        self.0.bit_none()
    }

    /// Returns an iterator for visiting all directions in the set.
    /// The iteration order may be implementation dependent.
    pub fn iter(&self) -> DirectionSetIter<'_> {
        DirectionSetIter::new(self)
    }

    /// Returns true if the `DirectionSet` contains the given `Direction`,
    /// false otherwise.
    pub fn contains(&self, dir: Direction) -> bool {
        self.0.bit_test(dir as usize)
    }
}