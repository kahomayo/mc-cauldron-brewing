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
pub struct LiquidData(pub u16);

impl LiquidData {
    pub fn apply_ingredient(self, ingredient: BasicPotionIngredient) -> Self {
        let mut result = self;
        for bit in ingredient.added_bits() {
            result.0 |= 1 << bit;
        }
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
        let first_set = self.first_set();
        if first_set < 2 || (self.0 & (1 << first_set - 1)) != 0 {
            return self;
        }
        let mut res = self.0 & !(1 << first_set);
        res <<= 1;
        res |= 0b11 << first_set - 1;
        Self(res)
    }

    fn first_set(self) -> i32 {
        math::first_set(self.0)
    }

    fn apply_automaton(self) -> Self {
        let first_set = self.first_set();
        let without_leading_bits = if first_set >= 0 {
            self.0 & !(1 << first_set)
        } else {
            self.0
        };
        let evolved: u16 = {
            let mut next = FungalAutomaton::new(without_leading_bits);
            let mut current = FungalAutomaton::default();
            while current != next {
                let new_next = next.next();
                current = next;
                next = new_next;
            }
            current.into()
        };
        let result = if first_set >= 0 {
            evolved | 1 << first_set
        } else {
            evolved
        };
        Self(result)
    }
}

mod fungal {
    use std::ops::{Index, IndexMut};

    #[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
    pub struct FungalAutomaton(WrappingArr<bool, 15>);

    #[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Hash)]
    struct WrappingArr<T, const I: usize>(pub [T; I]);

    impl<T, const I: usize> Index<isize> for WrappingArr<T, I> {
        type Output = T;

        fn index(&self, index: isize) -> &Self::Output {
            &self.0[index.rem_euclid(I as isize) as usize]
        }
    }

    impl<T, const I: usize> IndexMut<isize> for WrappingArr<T, I> {
        fn index_mut(&mut self, index: isize) -> &mut Self::Output {
            &mut self.0[index.rem_euclid(I as isize) as usize]
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
        pub fn new(v: u16) -> Self {
            let mut res = Self::default();
            for i in 0..15 {
                res[i] = (v & (1 << i)) != 0;
            }
            res
        }

        pub fn next(&self) -> Self {
            let mut next_gen = Self::default();
            for i in 0..15isize {
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

    impl Into<u16> for FungalAutomaton {
        fn into(self) -> u16 {
            let mut res = 0;
            for i in 0..15 {
                if self[i] {
                    res |= 1 << i;
                }
            }
            res
        }
    }

    impl Index<isize> for FungalAutomaton {
        type Output = bool;

        fn index(&self, index: isize) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl IndexMut<isize> for FungalAutomaton {
        fn index_mut(&mut self, index: isize) -> &mut Self::Output {
            self.0.index_mut(index)
        }
    }
}

pub(crate) mod math {
    pub fn first_set(v: u16) -> i32 {
        15 - (v.leading_zeros() as i32)
    }
}

#[cfg(test)]
mod tests {
    use crate::BasicPotionIngredient::{
        BlazePowder, FermentedSpiderEye, MagmaCream, SpiderEye, Sugar,
    };
    use crate::LiquidData;

    #[test]
    fn potion_w_is_correct() {
        assert_eq!(LiquidData::default().dilute().0, 0);
    }

    #[test]
    fn potion_water_fermented_is_correct() {
        assert_eq!(
            LiquidData::default()
                .dilute()
                .apply_ingredient(FermentedSpiderEye)
                .0,
            16896
        );
    }

    #[test]
    fn potion_water_fermented_water_is_correct() {
        assert_eq!(
            LiquidData::default()
                .dilute()
                .apply_ingredient(FermentedSpiderEye)
                .dilute()
                .0,
            16384
        );
    }

    #[test]
    fn potion_water_eye_is_correct() {
        assert_eq!(
            LiquidData::default().dilute().apply_ingredient(SpiderEye).0,
            1184
        );
    }

    #[test]
    fn potion_water_eye_wart_is_correct() {
        assert_eq!(
            LiquidData::default()
                .dilute()
                .apply_ingredient(SpiderEye)
                .apply_wart()
                .0,
            1088
        );
    }

    #[test]
    fn potion_water_eye_fermented_blaze_magma_sugar_wart_is_correct() {
        assert_eq!(
            LiquidData::default()
                .dilute()
                .apply_ingredient(SpiderEye)
                .apply_ingredient(FermentedSpiderEye)
                .apply_ingredient(BlazePowder)
                .apply_ingredient(MagmaCream)
                .apply_ingredient(Sugar)
                .apply_wart()
                .0,
            20614
        );
    }

    #[test]
    fn potion_water_eye_fermented_blaze_magma_sugar_wart_water_is_correct() {
        assert_eq!(
            LiquidData::default()
                .dilute()
                .apply_ingredient(SpiderEye)
                .apply_ingredient(FermentedSpiderEye)
                .apply_ingredient(BlazePowder)
                .apply_ingredient(MagmaCream)
                .apply_ingredient(Sugar)
                .apply_wart()
                .dilute()
                .0,
            20484
        );
    }

    #[test]
    fn potion_water_eye_fermented_blaze_magma_sugar_wart_water_sugar_is_correct() {
        assert_eq!(
            LiquidData::default()
                .dilute()
                .apply_ingredient(SpiderEye)
                .apply_ingredient(FermentedSpiderEye)
                .apply_ingredient(BlazePowder)
                .apply_ingredient(MagmaCream)
                .apply_ingredient(Sugar)
                .apply_wart()
                .dilute()
                .apply_ingredient(Sugar)
                .0,
            20485
        );
    }
}
