use crate::{color::Color, mask::Mask, state::State};

pub struct Capture<'a> {
    attacker: &'a mut Mask,
    defender: &'a mut Mask,
}

impl<'a> Capture<'a> {
    pub fn new(state: &'a mut State, color: Color) -> Self {
        let State { black, white } = state;
        let attacker;
        let defender;

        match color {
            Color::Black => {
                attacker = black;
                defender = white;
            }
            Color::White => {
                attacker = white;
                defender = black;
            }
        };

        Self { attacker, defender }
    }

    pub fn try_capture(&mut self, x: usize, y: usize) -> bool {
        let group = self.defender.flood(x, y);
        let is_capture = !group.has_a_liberty(self.attacker);
        if is_capture {
            for (row, &mask) in self.defender.rows_mut().zip(group.rows()) {
                *row &= !mask;
            }
        }
        is_capture
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mask::Mask;

    #[test]
    fn captures_one_stone() {
        #[rustfmt::skip]
        let black = Mask::new([
            0b010,
            0b101,
            0b010,
        ]);

        #[rustfmt::skip]
        let white = Mask::new([
            0b000,
            0b010,
            0b000,
        ]);

        let mut state = State::new(black, white);
        let mut capture = Capture::new(&mut state, Color::Black);
        assert!(capture.try_capture(1, 1));

        #[rustfmt::skip]
        let expected_black = Mask::new([
            0b010,
            0b101,
            0b010,
        ]);
        assert_eq!(state.black, expected_black);
        assert_eq!(state.white, Mask::EMPTY);
    }
}
