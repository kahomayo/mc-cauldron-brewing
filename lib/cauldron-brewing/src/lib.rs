use crate::fungal::FungalAutomaton;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum BasicPotionIngredient {
    Sugar,
    GhastTear,
    SpiderEye,
    FermentedSpiderEye,
    BlazePowder,
    MagmaCream,
}

impl BasicPotionIngredient {
    pub fn added_bits(self) -> &'static [u8] {
        match self {
            BasicPotionIngredient::Sugar => &[0u8],
            BasicPotionIngredient::GhastTear => &[11u8],
            BasicPotionIngredient::SpiderEye => &[5u8, 7u8, 10u8],
            BasicPotionIngredient::FermentedSpiderEye => &[9u8, 14u8],
            BasicPotionIngredient::BlazePowder => &[14u8],
            BasicPotionIngredient::MagmaCream => &[1u8, 6u8, 14u8],
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct LiquidData(pub u32);

impl LiquidData {
    pub fn apply_ingredient(self, ingredient: BasicPotionIngredient) -> Self {
        let mut result = self;
        for bit in ingredient.added_bits() {
            result.0 |= 1 << bit;
        }
        result.0 &= u16::MAX as u32;
        result
    }

    pub fn dilute(self) -> Self {
        Self(
            self.0
                & !((1 << 1) | (1 << 3) | (1 << 5) | (1 << 7) | (1 << 9) | (1 << 11) | (1 << 13)),
        )
    }

    pub fn apply_wart(self) -> Self {
        self.apply_stage_1().apply_automaton()
    }

    fn apply_stage_1(self) -> Self {
        if self.0 & 1 == 0 {
            return self;
        }
        let first_clear = self.first_clear();
        if first_clear < 2 || (self.0 & (1 << first_clear - 1)) != 0 {
            return self;
        }
        let mut res = self.0 & !(1 << first_clear);
        res <<= 1;
        res |= 0b11 << first_clear - 1;
        Self(res & u16::MAX as u32)
    }

    fn first_clear(self) -> i32 {
        math::first_clear(0xffff_8000 | self.0)
    }

    fn apply_automaton(self) -> Self {
        let first_clear = self.first_clear();
        let without_leading_bits = if first_clear >= 0 {
            self.0 & !(1 << first_clear)
        } else {
            self.0
        };
        let evolved: u32 = {
            let mut curr = FungalAutomaton::new(without_leading_bits);
            let mut next = FungalAutomaton::default();
            while curr != next {
                let new = curr.next();
                curr = next;
                next = new;
            }
            curr.into()
        };
        let result = if first_clear >= 0 {
            evolved | 1 << first_clear
        } else {
            evolved
        };
        Self(result & u16::MAX as u32)
    }
}

mod fungal {
    use std::ops::{Index, IndexMut};

    #[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
    pub struct FungalAutomaton(WrappingArr<bool, 15>);

    #[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Hash)]
    struct WrappingArr<T, const I: usize>(pub [T; I]);

    impl<T, const I: usize> Index<usize> for WrappingArr<T, I> {
        type Output = T;

        fn index(&self, index: usize) -> &Self::Output {
            &self.0[index % I]
        }
    }

    impl<T, const I: usize> IndexMut<usize> for WrappingArr<T, I> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.0[index % I]
        }
    }

    impl<T, const I: usize> Default for WrappingArr<T, I>
    where
        [T; I]: Default,
    {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    impl FungalAutomaton {
        pub fn new(v: u32) -> Self {
            let mut res = Self::default();
            for i in 0..15 {
                res[i] = v & (1 << i) == 1;
            }
            res
        }

        pub fn next(&self) -> Self {
            let mut next_gen = Self::default();
            for i in 0..15 {
                let should_set = if self[i] {
                    (self[i + 1] || !self[i + 2]) && (self[i - 1] || !self[i - 2])
                } else {
                    self[i - 1] && self[i + 1]
                };
                next_gen[i] = should_set;
            }
            next_gen
        }
    }

    impl Into<u32> for FungalAutomaton {
        fn into(self) -> u32 {
            let mut res = 0;
            for i in 0..15 {
                if self[i] {
                    res |= 1 << i;
                }
            }
            res
        }
    }

    impl Index<usize> for FungalAutomaton {
        type Output = bool;

        fn index(&self, index: usize) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl IndexMut<usize> for FungalAutomaton {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            self.0.index_mut(index)
        }
    }
}

pub(crate) mod math {
    pub fn first_clear(v: u32) -> i32 {
        31 - (v.leading_ones() as i32)
    }

    #[cfg(test)]
    mod tests {
        use crate::math::first_clear;

        #[test]
        fn first_clear_0_is_31() {
            assert_eq!(first_clear(0), 31);
        }

        #[test]
        fn first_clear_pot_is_14() {
            assert_eq!(first_clear(0xffff_8000), 14);
        }

        #[test]
        fn first_clear_max_is_m1() {
            assert_eq!(first_clear(0xffff_ffff), -1);
        }
    }
}
