extern crate endgame_direction;

#[cfg(test)]
mod tests {
    use endgame_direction::Direction;

    #[test]
    fn trivial() {
        Direction::North.is_cardinal();
    }
}
