struct Mask([u32; 19]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    states: Vec<State>,
    to_play: Color,
}

impl Sequence {
    pub fn empty() -> Self {
        Self {
            states: vec![State::new()],
            to_play: Color::Black,
        }
    }

    pub fn place_stone(&mut self, x: usize, y: usize) -> Result<(), PlaceStoneError> {
        assert!(x <= 18);
        assert!(y <= 18);

        let mut state = self
            .states
            .last()
            .expect("Sequence should have at least one state")
            .clone();

        if col(state.white[y] | state.black[y as usize], x) {
            return Err(PlaceStoneError::AlreadyExists);
        }

        let attacker = if self.states.len() % 2 == 1 {
            self.to_play
        } else {
            self.to_play.opposite()
        };
        state.set(x, y, attacker);

        let mut capture = Capture::new(&state, attacker);
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
            let defender = attacker.opposite();
            let left = x == 0 || state.get(x - 1, y) == Some(defender);
            let right = x == 18 || state.get(x + 1, y) == Some(defender);
            let down = y == 0 || state.get(x, y - 1) == Some(defender);
            let up = y == 18 || state.get(x, y + 1) == Some(defender);
            let is_self_capture = left && right && down && up;
            if is_self_capture {
                return Err(PlaceStoneError::SelfCapture);
            }
        }

        if let Some(ko_test) = self.states.get(self.states.len() - 2) {
            if ko_test == &state {
                return Err(PlaceStoneError::Ko);
            }
        }

        self.states.push(state);

        Ok(())
    }

    pub fn go_back(&mut self) {
        if self.states.len() > 1 {
            self.states.pop();
        }
    }

    pub fn reset(&mut self) {
        self.states.drain(1..);
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
            visited: [0u32; 19],
            capturer,
        }
    }

    fn is_capture(&mut self, x: usize, y: usize) -> bool {
        assert!(x <= 18);
        assert!(y <= 18);

        if col(self.visited[y], x) {
            return true;
        }

        set_col(&mut self.visited[y], x);

        let attacker = match self.capturer {
            Color::Black => &self.state.black,
            Color::White => &self.state.white,
        };

        let defender = match self.capturer {
            Color::Black => &self.state.white,
            Color::White => &self.state.black,
        };

        col(attacker[y], x)
            || (col(defender[y], x)
                && (x == 0 || self.is_capture(x - 1, y))
                && (x >= 18 || self.is_capture(x + 1, y))
                && (y == 0 || self.is_capture(x, y - 1))
                && (y >= 18 || self.is_capture(x, y + 1)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct State {
    black: Mask,
    white: Mask,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        assert!(x <= 18);
        assert!(y <= 18);

        let to_set = match color {
            Color::Black => &mut self.black,
            Color::White => &mut self.white,
        };
        set_col(&mut to_set[y], x);
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        assert!(x <= 18);
        assert!(y <= 18);

        if col(self.black[y], x) {
            Some(Color::Black)
        } else if col(self.white[y], x) {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn mask_group(&self, x: usize, y: usize, color: Color) -> Mask {
        let mut mask = Mask::default();
    }

    pub fn remove_group(&mut self, x: usize, y: usize) {
        let target = if col(self.black[y], x) {
            &mut self.black
        } else if col(self.white[y], x) {
            &mut self.white
        } else {
            return;
        };

        let mut stack = vec![];
        stack.push((x, y));

        self.remove_group_inner(x, y, target);
    }

    fn remove_group_inner(&mut self, x: usize, y: usize, target: &mut Mask) {
        assert!(x <= 18);
        assert!(y <= 18);

        let row = &mut target[y];
        if col(*row, x) {
            unset_col(row, x);
            self.remove_group_inner(x.saturating_sub(1), y, target);
            self.remove_group_inner(x, y.saturating_sub(1), target);
            self.remove_group_inner(x.wrapping_add(1).min(18), y, target);
            self.remove_group_inner(x, y.wrapping_add(1).min(18), target);
        }
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

fn col(row: u32, i: usize) -> bool {
    row >> i & 1 == 1
}

fn set_col(row: &mut u32, i: usize) {
    *row |= 1 << i;
}

fn unset_col(row: &mut u32, i: usize) {
    *row &= !(1 << i);
}
