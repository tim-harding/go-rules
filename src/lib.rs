use std::{
    fmt::{self, Debug, Formatter},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut, Not, Shl, ShlAssign, Shr,
        ShrAssign,
    },
};

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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mask([MaskRow; 19]);

impl Mask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        assert!(x <= 18);
        assert!(y <= 18);
        self.0[y].get(x)
    }

    pub fn set(&mut self, x: usize, y: usize) {
        assert!(x <= 18);
        assert!(y <= 18);
        self.0[y].set(x);
    }

    pub fn unset(&mut self, x: usize, y: usize) {
        assert!(x <= 18);
        assert!(y <= 18);
        self.0[y].unset(x)
    }

    pub fn row(&self, y: usize) -> &MaskRow {
        assert!(y <= 18);
        &self.0[y]
    }

    pub fn row_mut(&mut self, y: usize) -> &mut MaskRow {
        assert!(y <= 18);
        &mut self.0[y]
    }

    pub fn expand(&self, stencil: &Mask) -> Self {
        let mut out = Mask::new();
        out.0[0] |= self.0[1] | self.0[0] << 1 | self.0[0] >> 1 & stencil.0[0];
        for i in 1..=17 {
            out.0[i] |=
                self.0[i - 1] | self.0[i] << 1 | self.0[i] >> 1 | self.0[i + 1] & stencil.0[i];
        }
        out.0[18] |= self.0[17] | self.0[18] << 1 | self.0[18] >> 1 & stencil.0[18];
        out
    }

    pub fn rows(&self) -> impl Iterator<Item = &MaskRow> {
        self.0.iter()
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut MaskRow> {
        self.0.iter_mut()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaskRow(u32);

impl MaskRow {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, i: usize) -> bool {
        assert!(i <= 18);
        self.0 >> i & 1 == 1
    }

    fn set(&mut self, i: usize) {
        assert!(i <= 18);
        self.0 |= 1 << i;
    }

    fn unset(&mut self, i: usize) {
        assert!(i <= 18);
        self.0 &= !(1 << i);
    }
}

impl Deref for MaskRow {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaskRow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitAnd for MaskRow {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for MaskRow {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for MaskRow {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for MaskRow {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl Shl<usize> for MaskRow {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ShlAssign for MaskRow {
    fn shl_assign(&mut self, rhs: Self) {
        self.0 <<= rhs.0
    }
}

impl Shr<usize> for MaskRow {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl ShrAssign for MaskRow {
    fn shr_assign(&mut self, rhs: Self) {
        self.0 >>= rhs.0
    }
}

impl Not for MaskRow {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
