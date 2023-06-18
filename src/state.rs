use std::fmt::{self, Debug, Formatter};

use crate::{color::Color, mask::Mask};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct State {
    pub(crate) black: Mask,
    pub(crate) white: Mask,
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
        let mut mask = Mask::EMPTY;
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
