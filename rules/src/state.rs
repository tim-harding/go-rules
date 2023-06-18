use crate::{color::Color, mask::Mask};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct State {
    pub(crate) black: Mask,
    pub(crate) white: Mask,
}

impl State {
    pub fn new(black: Mask, white: Mask) -> Self {
        Self { black, white }
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
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n")?;
        for y in 0..19 {
            for x in 0..19 {
                let c = match self.get(x, y) {
                    Some(Color::Black) => 'b',
                    Some(Color::White) => 'w',
                    None => '.',
                };
                write!(f, "{c}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
