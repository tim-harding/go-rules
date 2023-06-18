mod mask;

use std::fmt::{self, Debug, Formatter};

use mask::Mask;

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
    states: Vec<Node>,
    current: usize,
    to_play: Color,
    placement_mode: PlacementMode,
}

impl Tree {
    pub fn empty() -> Self {
        Self {
            states: vec![Node::new(State::new(), usize::MAX)],
            current: 0,
            to_play: Color::Black,
            placement_mode: PlacementMode::Toggle,
        }
    }

    pub fn place_stone(&mut self, x: usize, y: usize) -> Result<(), PlaceStoneError> {
        assert!(x <= 18);
        assert!(y <= 18);

        let node = self.states[self.current];
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

        if let Some(parent) = self.states.get(node.parent) {
            if parent.state == state {
                return Err(PlaceStoneError::Ko);
            }
        }

        self.states.push(Node::new(state, self.current));
        self.current = self.states.len() - 1;
        if self.placement_mode == PlacementMode::Toggle {
            self.to_play = self.to_play.opposite();
        }

        Ok(())
    }

    pub fn set_placement_mode(&mut self, mode: PlacementMode) {
        self.placement_mode = mode;
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct State {
    black: Mask,
    white: Mask,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, x: usize, y: usize, color: Option<Color>) {
        assert!(x <= 18);
        assert!(y <= 18);

        match color {
            Some(Color::Black) => self.black.set(x, y),
            Some(Color::White) => self.white.set(x, y),
            None => {
                self.black.unset(x, y);
                self.white.unset(x, y);
            }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        assert!(x <= 18);
        assert!(y <= 18);

        if self.black.get(x, y) {
            Some(Color::Black)
        } else if self.white.get(x, y) {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn mask_group(&self, x: usize, y: usize, color: Color) -> Mask {
        let mut mask = Mask::new();
        let stencil = match color {
            Color::Black => &self.black,
            Color::White => &self.white,
        };
        mask.set(x, y);
        loop {
            let next = mask.expand(stencil);
            if next == mask {
                break;
            }
            mask = next;
        }
        mask
    }

    pub fn remove_group(&mut self, x: usize, y: usize) {
        if let Some(color) = self.get(x, y) {
            let mask = self.mask_group(x, y, color);
            let target = match color {
                Color::Black => &mut self.black,
                Color::White => &mut self.white,
            };
            for (row, &mask) in target.rows_mut().zip(mask.rows()) {
                *row &= !mask;
            }
        }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..19 {
            for x in 0..19 {
                let c = match self.get(x, y) {
                    Some(Color::Black) => 'b',
                    Some(Color::White) => 'w',
                    None => ' ',
                };
                write!(f, "{c}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}
