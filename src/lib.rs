mod capture;
mod color;
mod mask;
mod mask_row;
mod state;

use color::Color;
use state::State;
use std::fmt::Debug;

use crate::capture::Capture;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Node {
    state: State,
    parent: usize,
}

impl Node {
    pub fn new(state: State, parent: usize) -> Self {
        Self { state, parent }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlacementMode {
    Black,
    White,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    nodes: Vec<Node>,
    current: usize,
    to_play: Color,
    pub placement_mode: PlacementMode,
}

impl Tree {
    pub fn new(state: State, to_play: Color) -> Self {
        Self {
            nodes: vec![Node::new(state, usize::MAX)],
            current: 0,
            to_play,
            placement_mode: PlacementMode::Toggle,
        }
    }

    pub fn empty() -> Self {
        Self {
            nodes: vec![Node::new(State::default(), usize::MAX)],
            current: 0,
            to_play: Color::Black,
            placement_mode: PlacementMode::Toggle,
        }
    }

    pub fn place_stone(&mut self, x: usize, y: usize) -> Result<(), PlaceStoneError> {
        assert!(x <= 18);
        assert!(y <= 18);

        let current_node = self.nodes[self.current];
        let mut state = current_node.state.clone();

        if state.black.get(x, y) || state.white.get(x, y) {
            return Err(PlaceStoneError::AlreadyExists);
        }

        state.set(x, y, Some(self.to_play));

        let mut capture = Capture::new(&mut state, self.to_play);
        let mut is_capture = false;
        is_capture |= x > 0 && capture.try_capture(x - 1, y);
        is_capture |= x < 18 && capture.try_capture(x + 1, y);
        is_capture |= y > 0 && capture.try_capture(x, y - 1);
        is_capture |= y < 18 && capture.try_capture(x, y + 1);

        if !is_capture {
            let defender = Some(self.to_play.opposite());
            let left = x == 0 || state.get(x - 1, y) == defender;
            let right = x == 18 || state.get(x + 1, y) == defender;
            let down = y == 0 || state.get(x, y - 1) == defender;
            let up = y == 18 || state.get(x, y + 1) == defender;
            let is_self_capture = left && right && down && up;
            if is_self_capture {
                return Err(PlaceStoneError::SelfCapture);
            }
        }

        if let Some(parent) = self.nodes.get(current_node.parent) {
            if parent.state == state {
                return Err(PlaceStoneError::Ko);
            }
        }

        self.nodes.push(Node::new(state, self.current));
        self.current = self.nodes.len() - 1;
        if self.placement_mode == PlacementMode::Toggle {
            self.to_play = self.to_play.opposite();
        }

        Ok(())
    }

    pub fn current(&self) -> &State {
        &self.nodes[self.current].state
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlaceStoneError {
    #[error("The stone placement violates ko rules")]
    Ko,
    #[error("The stone placement results in self-capture")]
    SelfCapture,
    #[error("Attempting to place a stone in an occupied intersection")]
    AlreadyExists,
}

#[cfg(test)]
mod tests {
    use crate::mask::Mask;

    use super::*;

    #[test]
    fn already_exists() {
        let state = State::new(Mask::new([0b1]), Mask::EMPTY);
        let mut tree = Tree::new(state, Color::White);
        assert_eq!(tree.place_stone(0, 0), Err(PlaceStoneError::AlreadyExists));
    }

    #[test]
    fn self_capture() {
        #[rustfmt::skip]
        let white = Mask::new([
            0b010,
            0b101,
            0b010,
        ]);
        let state = State::new(Mask::EMPTY, white);
        let mut tree = Tree::new(state, Color::Black);
        assert_eq!(tree.place_stone(1, 1), Err(PlaceStoneError::SelfCapture));
    }

    #[test]
    fn ko() {
        #[rustfmt::skip]
        let black = Mask::new([
            0b0010,
            0b0101,
            0b0010,
        ]);

        #[rustfmt::skip]
        let white = Mask::new([
            0b0100,
            0b1000,
            0b0100,
        ]);

        #[rustfmt::skip]
        let black_expected = Mask::new([
            0b0010,
            0b0001,
            0b0010,
        ]);

        #[rustfmt::skip]
        let white_expected = Mask::new([
            0b0100,
            0b1010,
            0b0100,
        ]);

        let expected = State::new(black_expected, white_expected);
        let state = State::new(black, white);
        let mut tree = Tree::new(state, Color::White);
        assert_eq!(tree.place_stone(1, 1), Ok(()));
        assert_eq!(tree.current(), &expected);
        assert_eq!(tree.place_stone(2, 1), Err(PlaceStoneError::Ko));
    }

    #[test]
    fn capture_multiple() {
        #[rustfmt::skip]
        let black = Mask::new([
            0b111111111,
            0b100000001,
            0b111101111,
        ]);

        #[rustfmt::skip]
        let white = Mask::new([
            0b000000000,
            0b011101110,
            0b000010000,
            0b000010000,
        ]);

        #[rustfmt::skip]
        let black_expected = Mask::new([
            0b111111111,
            0b100010001,
            0b111101111,
        ]);

        #[rustfmt::skip]
        let white_expected = Mask::new([
            0b000000000,
            0b000000000,
            0b000010000,
            0b000010000,
        ]);

        let state = State::new(black, white);
        let expected = State::new(black_expected, white_expected);
        let mut tree = Tree::new(state, Color::Black);
        assert_eq!(tree.place_stone(4, 1), Ok(()));
        assert_eq!(tree.current(), &expected);
    }
}
