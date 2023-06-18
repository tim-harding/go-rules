use std::{
    fmt::{self, Debug, Formatter},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut, Index, IndexMut, Not, Shl,
        ShlAssign, Shr, ShrAssign,
    },
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
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
            write!(f, "{:019b}\n", row.0)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaskRow(u32);

impl MaskRow {
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
