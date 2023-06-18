mod color;
mod mask;
mod mask_row;
mod state;

use color::Color;
use mask::Mask;
use state::State;
use std::fmt::Debug;

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

        let node = self.nodes[self.current];
        let mut state = node.state.clone();

        if state.black.get(x, y) || state.white.get(x, y) {
            return Err(PlaceStoneError::AlreadyExists);
        }

        state.set(x, y, Some(self.to_play));

        let mut capture = Capture::new(&state, self.to_play);
        let left = x > 0 && capture.is_capture(x - 1, y);
        let right = x < 18 && capture.is_capture(x + 1, y);
        let down = y > 0 && capture.is_capture(x, y - 1);
        let up = y < 18 && capture.is_capture(x, y + 1);
        let did_capture = left || right || down || up;
        if did_capture {
            if left {
                state.remove_group(x.wrapping_sub(1), y);
            }
            if right {
                state.remove_group(x.wrapping_add(1), y);
            }
            if up {
                state.remove_group(x, y.wrapping_add(1));
            }
            if down {
                state.remove_group(x, y.wrapping_sub(1));
            }
        } else {
            let defender = self.to_play.opposite();
            let left = x == 0 || state.get(x - 1, y) == Some(defender);
            let right = x == 18 || state.get(x + 1, y) == Some(defender);
            let down = y == 0 || state.get(x, y - 1) == Some(defender);
            let up = y == 18 || state.get(x, y + 1) == Some(defender);
            let is_self_capture = left && right && down && up;
            if is_self_capture {
                return Err(PlaceStoneError::SelfCapture);
            }
        }

        if let Some(parent) = self.nodes.get(node.parent) {
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

struct Capture<'a> {
    state: &'a State,
    visited: Mask,
    capturer: Color,
}

impl<'a> Capture<'a> {
    pub fn new(state: &'a State, capturer: Color) -> Self {
        Self {
            state,
            visited: Mask::default(),
            capturer,
        }
    }

    fn is_capture(&mut self, x: usize, y: usize) -> bool {
        assert!(x <= 18);
        assert!(y <= 18);

        if self.visited.get(x, y) {
            return true;
        }

        self.visited.set(x, y);

        let attacker = match self.capturer {
            Color::Black => &self.state.black,
            Color::White => &self.state.white,
        };

        let defender = match self.capturer {
            Color::Black => &self.state.white,
            Color::White => &self.state.black,
        };

        attacker.get(x, y)
            || (defender.get(x, y)
                && (x == 0 || self.is_capture(x - 1, y))
                && (x >= 18 || self.is_capture(x + 1, y))
                && (y == 0 || self.is_capture(x, y - 1))
                && (y >= 18 || self.is_capture(x, y + 1)))
    }
}

#[derive(Debug, thiserror::Error)]
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
    use super::*;

    #[test]
    fn captures_one_stone() {
        #[rustfmt::skip]
        let black = Mask::new([
            0b000,
            0b101,
            0b010,
        ]);

        #[rustfmt::skip]
        let white = Mask::new([
            0b000,
            0b010,
            0b000,
        ]);

        let state = State::new(black, white);
        let mut tree = Tree::new(state, Color::Black);
        tree.place_stone(2, 0)
            .expect("Should be able to capture one stone");

        #[rustfmt::skip]
        let expected_black = Mask::new([
            0b010,
            0b101,
            0b010,
        ]);
        let expected = State::new(expected_black, Mask::EMPTY);

        assert_eq!(tree.current(), &expected);
    }
}
