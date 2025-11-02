use crate::{Coord, ModuleCoord};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HashShape<C: Coord> {
    set: HashSet<C>,
}

impl<C: Coord> From<&[C]> for HashShape<C> {
    fn from(slice: &[C]) -> Self {
        Self {
            set: slice.to_owned().into_iter().collect(),
        }
    }
}
impl<C: Coord, const N: usize> From<[C; N]> for HashShape<C> {
    fn from(slice: [C; N]) -> Self {
        Self {
            set: slice.to_owned().into_iter().collect(),
        }
    }
}

impl<C: Coord> FromIterator<C> for HashShape<C> {
    fn from_iter<I: IntoIterator<Item=C>>(iter: I) -> Self {
        Self {
            set: iter.into_iter().collect(),
        }
    }
}

impl<C: Coord> Hash for HashShape<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Order-independent hashing for the set.
        let mut hashes: Vec<u64> = self
            .set
            .iter()
            .map(|item| {
                let mut hasher = std::hash::DefaultHasher::new();
                item.hash(&mut hasher);
                hasher.finish()
            })
            .collect();
        hashes.sort_unstable();
        for h in hashes {
            h.hash(state);
        }
    }
}

impl<C: Coord> IntoIterator for HashShape<C> {
    type Item = C;
    type IntoIter = std::collections::hash_set::IntoIter<C>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter()
    }
}

impl<C: Coord> std::ops::Sub for HashShape<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        HashShape {
            set: self.set.difference(&rhs.set).cloned().collect(),
        }
    }
}

impl<C: Coord> std::ops::Sub<&HashShape<C>> for HashShape<C> {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        HashShape {
            set: self.set.difference(&rhs.set).cloned().collect(),
        }
    }
}

impl<'a, C: Coord> std::ops::Sub<HashShape<C>> for &'a HashShape<C> {
    type Output = HashShape<C>;

    fn sub(self, rhs: HashShape<C>) -> Self::Output {
        HashShape {
            set: self.set.difference(&rhs.set).cloned().collect(),
        }
    }
}

impl<'a, 'b, C: Coord> std::ops::Sub<&'b HashShape<C>> for &'a HashShape<C> {
    type Output = HashShape<C>;

    fn sub(self, rhs: &'b HashShape<C>) -> Self::Output {
        HashShape {
            set: self.set.difference(&rhs.set).cloned().collect(),
        }
    }
}

impl<C: Coord> crate::Shape<C> for HashShape<C> {
    type Iterator<'a>
    = HashShapeIterator<'a, C>
    where
        Self: 'a,
        C: 'a;

    fn new() -> Self {
        Self {
            set: HashSet::new(),
        }
    }

    fn contains(&self, coord: &C) -> bool {
        self.set.contains(coord)
    }

    fn is_subshape(&self, other: &Self) -> bool {
        self.set.is_subset(&other.set)
    }

    fn is_supershape(&self, other: &Self) -> bool {
        self.set.is_superset(&other.set)
    }

    fn is_disjoint(&self, other: &Self) -> bool {
        self.set.is_disjoint(&other.set)
    }

    fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    fn union<'a>(&'a self, other: &'a Self) -> Self
    where
        C: 'a,
    {
        HashShape {
            set: self.set.union(&other.set).cloned().collect(),
        }
    }

    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        C: 'a,
    {
        HashShapeIterator {
            inner: self.set.iter(),
        }
    }
}

impl<MC: ModuleCoord> crate::ModuleShape<MC> for HashShape<MC>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    fn translate(&self, offset: &MC) -> Self {
        let new_set = self
            .set
            .iter()
            .map(|coord| coord + offset)
            .collect::<HashSet<_>>();
        HashShape { set: new_set }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct HashShapeIterator<'a, C: Coord + 'a> {
    inner: std::collections::hash_set::Iter<'a, C>,
}

impl<'a, C: Coord + 'a> HashShapeIterator<'a, C> {
    pub fn empty() -> Self {
        HashShapeIterator {
            inner: std::collections::hash_set::Iter::<'a, C>::default(),
        }
    }
}

impl<'a, C: Coord + 'a> Iterator for HashShapeIterator<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, C: Coord + 'a> crate::ShapeIterator<'a, C> for HashShapeIterator<'a, C> {}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashShapeContainer<C: Coord, V>
where
    V: Debug + Clone + PartialEq + Eq + Hash,
{
    map: HashMap<C, V>,
}

impl<'a, 'b, C: Coord, V> std::ops::Sub<&'b HashShapeContainer<C, V>>
for &'a HashShapeContainer<C, V>
where
    V: Debug + Clone + PartialEq + Eq + Hash,
{
    type Output = HashShapeContainer<C, V>;

    fn sub(self, rhs: &'b HashShapeContainer<C, V>) -> Self::Output {
        let mut map = self.map.clone();
        map.retain(|c, _| !rhs.map.keys().contains(c));
        HashShapeContainer { map }
    }
}

impl<C: Coord, V> HashShapeContainer<C, V>
where
    V: Debug + Clone + PartialEq + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn from_shape_value<S: crate::Shape<C>>(shape: S, v: V) -> Self
    where
    // TODO Should this really be needed? Shouldn't it already be implied
    //  by S satisfying crate::Shape<C>?
        S: std::ops::Sub<Output=S>,
        for<'a> S: std::ops::Sub<&'a S, Output=S>,
        for<'b> S: std::ops::Sub<&'b S, Output=S>,
        for<'a, 'b> &'a S: std::ops::Sub<&'b S, Output=S>,
    {
        Self {
            map: shape.iter().cloned().zip(std::iter::repeat(v)).collect(),
        }
    }

    pub fn from_iter_value<I: IntoIterator<Item=C>>(iter: I, v: V) -> Self {
        Self {
            map: iter.into_iter().zip(std::iter::repeat(v)).collect(),
        }
    }
}

impl<C: Coord, V> FromIterator<(C, V)> for HashShapeContainer<C, V>
where
    V: Debug + Clone + PartialEq + Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item=(C, V)>>(iter: I) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}

impl<C: Coord, V: Debug + Clone + PartialEq + Eq + Hash> Hash for HashShapeContainer<C, V>
where
    V: Debug + Clone,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Order-independent hashing for the map.
        let mut hashes: Vec<u64> = self
            .map
            .iter()
            .map(|(key, value)| {
                let mut hasher = std::hash::DefaultHasher::new();
                key.hash(&mut hasher);
                value.hash(&mut hasher);
                hasher.finish()
            })
            .collect();
        hashes.sort_unstable();
        for h in hashes {
            h.hash(state);
        }
    }
}

impl<C: Coord, V: Debug + Clone + PartialEq + Eq + Hash> IntoIterator for HashShapeContainer<C, V>
where
    V: Debug + Clone,
{
    type Item = (C, V);
    type IntoIter = std::collections::hash_map::IntoIter<C, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<C: Coord, V: Debug + Clone + PartialEq + Eq + Hash> crate::ShapeContainer<C, V>
for HashShapeContainer<C, V>
where
    V: Debug + Clone,
{
    type Iterator<'a>
    = HashShapeContainerIterator<'a, C, V>
    where
        Self: 'a,
        C: 'a,
        V: 'a;

    type Shape = HashShape<C>;

    fn contains(&self, coord: &C) -> bool {
        self.map.contains_key(coord)
    }

    fn get(&self, coord: &C) -> Option<&V> {
        self.map.get(coord)
    }

    fn get_mut(&mut self, coord: &C) -> Option<&mut V> {
        self.map.get_mut(coord)
    }

    fn insert(&mut self, coord: C, value: V) -> Option<V> {
        self.map.insert(coord, value)
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn as_shape(&self) -> Self::Shape {
        HashShape {
            set: self.map.keys().cloned().collect(),
        }
    }

    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        C: 'a,
        V: 'a,
    {
        HashShapeContainerIterator {
            inner: self.map.iter(),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct HashShapeContainerIterator<'a, C: Coord + 'a, V: Debug + Clone + PartialEq + Eq + Hash>
where
    V: Debug + Clone,
{
    inner: std::collections::hash_map::Iter<'a, C, V>,
}

impl<'a, C: Coord + 'a, V: Debug + Clone + PartialEq + Eq + Hash>
HashShapeContainerIterator<'a, C, V>
where
    V: Debug + Clone,
{
    pub fn empty() -> Self {
        HashShapeContainerIterator {
            inner: std::collections::hash_map::Iter::<'a, C, V>::default(),
        }
    }
}

impl<'a, C: Coord + 'a, V: Debug + Clone + PartialEq + Eq + Hash> Iterator
for HashShapeContainerIterator<'a, C, V>
where
    V: Debug + Clone,
{
    type Item = (&'a C, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, C: Coord + 'a, V: Debug + Clone + PartialEq + Eq + Hash>
crate::ShapeContainerIterator<'a, C, V> for HashShapeContainerIterator<'a, C, V>
where
    V: Debug + Clone,
{}

//////////////////////////////////////////////////////////////////////////////////////////////////

impl<MC: ModuleCoord, V: Debug + Clone + PartialEq + Eq + Hash> crate::ModuleShapeContainer<MC, V>
for HashShapeContainer<MC, V>
where
        for<'a, 'b> &'a MC: std::ops::Add<&'b MC, Output=MC>,
        for<'a, 'b> &'a MC: std::ops::Sub<&'b MC, Output=MC>,
{
    fn translate(&self, offset: &MC) -> Self {
        let new_map = self
            .map
            .iter()
            .map(|(coord, value)| (coord + offset, value.clone()))
            .collect::<HashMap<_, _>>();
        HashShapeContainer { map: new_map }
    }
}