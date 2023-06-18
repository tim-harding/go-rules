use std::{
    fmt::{self, Debug, Formatter},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::mask_row::MaskRow;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mask([MaskRow; 19]);

impl Mask {
    pub const EMPTY: Self = Self([MaskRow::EMPTY; 19]);
    pub const FILLED: Self = Self([MaskRow::FILLED; 19]);

    pub fn new<const N: usize>(rows: [u32; N]) -> Self {
        assert!(
            N <= 19,
            "Cannot initialize a mask with more than nineteen rows"
        );
        let mut mask = Self::default();
        for (i, row) in rows.into_iter().enumerate() {
            mask.0[i] = MaskRow::new(row);
        }
        mask
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        assert!(x <= 18);
        assert!(y <= 18);
        self[y].get(x)
    }

    pub fn set(&mut self, x: usize, y: usize) {
        assert!(x <= 18);
        assert!(y <= 18);
        self[y].set(x);
    }

    pub fn unset(&mut self, x: usize, y: usize) {
        assert!(x <= 18);
        assert!(y <= 18);
        self[y].unset(x)
    }

    pub fn expand(&self, stencil: &Mask) -> Self {
        let mut out = Mask::EMPTY;
        out[0] = (self[1] | self[0].expand()) & stencil[0];
        for i in 1..=17 {
            out[i] = (self[i - 1] | self[i].expand() | self[i + 1]) & stencil[i];
        }
        out[18] = (self[17] | self[18].expand()) & stencil[18];
        out
    }

    pub fn rows(&self) -> impl Iterator<Item = &MaskRow> {
        self.0.iter()
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut MaskRow> {
        self.0.iter_mut()
    }
}

impl Deref for Mask {
    type Target = [MaskRow; 19];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Mask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<usize> for Mask {
    type Output = MaskRow;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Mask {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Debug for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in self.rows() {
            write!(f, "{:?}\n", row)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_format() {
        // Right part
        #[rustfmt::skip]
        let mut mask = Mask::new([
            0b0000, 
            0b0100, 
            0b1110, 
            0b0100,
        ]);

        // Left part
        mask.set(15, 2);
        mask.set(16, 1);
        mask.set(16, 2);
        mask.set(17, 2);
        mask.set(16, 3);

        let expected = "\
            0000000000000000000\n\
            0010000000000000100\n\
            0111000000000001110\n\
            0010000000000000100\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n\
            0000000000000000000\n";
        assert_eq!(format!("{mask:?}"), expected);
    }

    #[test]
    fn expands_single_stone() {
        #[rustfmt::skip]
        let mut actual = Mask::new([
            0b000,
            0b010,
            0b000,
        ]);
        actual = actual.expand(&Mask::FILLED);

        #[rustfmt::skip]
        let mut expected = Mask::new([
            0b010,
            0b111,
            0b010,
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn expands_from_border() {
        #[rustfmt::skip]
        let mut actual = Mask::new([
            0b001,
            0b000,
            0b000,
        ]);
        actual = actual.expand(&Mask::FILLED);
        actual = actual.expand(&Mask::FILLED);

        #[rustfmt::skip]
        let mut expected = Mask::new([
            0b111,
            0b011,
            0b001,
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn expands_curvy_shape() {
        #[rustfmt::skip]
        let mut actual = Mask::new([
            0b00000,
            0b01110,
            0b01010,
        ]);
        actual = actual.expand(&Mask::FILLED);

        #[rustfmt::skip]
        let mut expected = Mask::new([
            0b01110,
            0b11111,
            0b11111,
            0b01010,
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn keeps_stencil() {
        #[rustfmt::skip]
        let mut actual = Mask::new([
            0b00000,
            0b01000,
            0b00000,
            0b00000,
        ]);

        #[rustfmt::skip]
        let mut stencil = Mask::new([
            0b00000,
            0b01110,
            0b01110,
            0b01110,
        ]);

        #[rustfmt::skip]
        let mut expected = Mask::new([
            0b00000,
            0b01100,
            0b01000,
            0b00000,
        ]);
        actual = actual.expand(&stencil);
        assert_eq!(actual, expected);

        #[rustfmt::skip]
        let mut expected = Mask::new([
            0b00000,
            0b01110,
            0b01100,
            0b01000,
        ]);
        actual = actual.expand(&stencil);
        assert_eq!(actual, expected);
    }
}
