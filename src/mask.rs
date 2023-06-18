use std::{
    fmt::{self, Debug, Formatter},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::mask_row::MaskRow;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mask([MaskRow; 19]);

impl Mask {
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
        out[0] |= self[1] | self[0] << 1 | self[0] >> 1 & stencil[0];
        for i in 1..=17 {
            out[i] |= self[i - 1] | self[i] << 1 | self[i] >> 1 | self[i + 1] & stencil[i];
        }
        out[18] |= self[17] | self[18] << 1 | self[18] >> 1 & stencil[18];
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
    fn displays_in_debug_mode() {}
}
