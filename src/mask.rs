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

    pub fn new() -> Self {
        Self::default()
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
        let mut out = Mask::new();
        out[0] |= self[1] | self[0].expand() & stencil[0];
        for i in 1..=17 {
            out[i] |= self[i - 1] | self[i].expand() | self[i + 1] & stencil[i];
        }
        out[18] |= self[17] | self[18].expand() & stencil[18];
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
        let mut mask = Mask::new();
        mask.set(1, 2);
        mask.set(2, 1);
        mask.set(2, 2);
        mask.set(3, 2);
        mask.set(2, 3);
        let expected = "\
            0000000000000000000\n\
            0010000000000000000\n\
            0111000000000000000\n\
            0010000000000000000\n\
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
        let mut actual = Mask::new();
        actual.set(10, 10);
        actual = actual.expand(&Mask::FILLED);

        let mut expected = Mask::new();
        expected.set(10, 10);
        expected.set(9, 10);
        expected.set(11, 10);
        expected.set(10, 9);
        expected.set(10, 11);

        assert_eq!(actual, expected);
    }
}
