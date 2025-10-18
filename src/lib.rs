extern crate endgame_direction;
extern crate endgame_egui;
extern crate endgame_explorer;
extern crate endgame_generation;
extern crate endgame_grid;
extern crate endgame_ludic;

#[cfg(test)]
mod tests {
    use endgame_direction::Direction;

    #[test]
    fn trivial() {
        Direction::North.is_cardinal();
    }
}
