use std::{
    fmt::{self, Debug, Formatter},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut, Not, Shl, ShlAssign, Shr,
        ShrAssign,
    },
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaskRow(u32);

impl MaskRow {
    pub const EMPTY: Self = Self(0);
    pub const FILLED: Self = Self(0b1111111111111111111);

    pub fn new(value: u32) -> Self {
        assert!(
            &value < &Self::FILLED,
            "Cannot have a row with more than 19 stones"
        );
        Self(value)
    }

    pub fn get(&self, i: usize) -> bool {
        assert!(i <= 18);
        self.0 >> i & 1 == 1
    }

    pub fn set(&mut self, i: usize) {
        assert!(i <= 18);
        self.0 |= 1 << i;
    }

    pub fn unset(&mut self, i: usize) {
        assert!(i <= 18);
        self.0 &= !(1 << i);
    }

    pub fn expand(self) -> Self {
        self << 1 | self | self >> 1
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

impl Debug for MaskRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:019b}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_format() {
        let mut row = MaskRow::EMPTY;
        row.set(10);
        row.set(3);
        row.set(13);
        assert_eq!(format!("{row:?}"), "0000010010000001000")
    }
}
