use crate::fungal::FungalAutomaton;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum PotionIngredient {
    Sugar,
    GhastTear,
    SpiderEye,
    FermentedSpiderEye,
    BlazePowder,
    MagmaCream,
}

impl PotionIngredient {
    /// Lists the bits that are set by this ingredient
    pub fn added_bits(self) -> &'static [u8] {
        match self {
            PotionIngredient::Sugar => &[0u8],
            PotionIngredient::GhastTear => &[11u8],
            PotionIngredient::SpiderEye => &[5u8, 7u8, 10u8],
            PotionIngredient::FermentedSpiderEye => &[9u8, 14u8],
            PotionIngredient::BlazePowder => &[14u8],
            PotionIngredient::MagmaCream => &[1u8, 6u8, 14u8],
        }
    }
}

/// Represents the liquidData of a Cauldron tile entity or the damage value of a potion item.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct LiquidData(pub u16);

impl LiquidData {
    /// Calculates the result of adding an ingredient
    pub fn apply_ingredient(self, ingredient: PotionIngredient) -> Self {
        let mut result = self;
        for bit in ingredient.added_bits() {
            result.0 |= 1 << bit;
        }
        result
    }

    /// Calculates the result of adding a water bucket
    ///
    /// To do this in-game, you have to first remove a layer using an empty bottle.
    pub fn dilute(self) -> Self {
        Self(
            self.0
                & !((1 << 1) | (1 << 3) | (1 << 5) | (1 << 7) | (1 << 9) | (1 << 11) | (1 << 13)),
        )
    }

    /// Calculates the result of adding a nether wart.
    pub fn apply_wart(self) -> Self {
        self.apply_wart_stage_1().apply_automaton()
    }

    /// The first step of wart handling.
    fn apply_wart_stage_1(self) -> Self {
        // If the lowest bit isn't set, return.
        // lowest bit can be set by adding sugar, maybe also using warts?
        if self.0 & 1 == 0 {
            return self;
        }
        // find the first occurrence of '10' (from the right) in the data.
        let first_set = self.first_set();
        if first_set < 2 || (self.0 & (1 << first_set - 1)) != 0 {
            return self;
        }
        // clear every bit that is left of that.
        // e.g. '0011_1001' becomes '0000_1001'
        let mut res = self.0 & !(1 << first_set);
        res <<= 1;
        res |= 0b11 << first_set - 1;
        Self(res)
    }

    /// Finds the position of the first bit that is set
    fn first_set(self) -> i32 {
        math::first_set(self.0)
    }

    /// Applies the nether wart automaton
    fn apply_automaton(self) -> Self {
        // Remove the first bit that is set
        let first_set = self.first_set();
        let without_leading_bits = if first_set >= 0 {
            self.0 & !(1 << first_set)
        } else {
            self.0
        };

        // Run the fungal automaton until its output stops changing
        let evolved: u16 = {
            let mut next = FungalAutomaton::new(without_leading_bits);
            let mut current = FungalAutomaton::default();
            while current != next {
                current = next;
                next = next.next();
            }
            current.into()
        };

        // Add the bit that was removed above
        let result = if first_set >= 0 {
            evolved | 1 << first_set
        } else {
            evolved
        };

        Self(result)
    }
}

mod fungal {
    /// Represents the cellular automaton used for nether warts.
    #[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
    pub struct FungalAutomaton(pub u16);

    impl FungalAutomaton {
        /// Calculates the next generation.
        pub fn next(&self) -> Self {
            let mut next_gen = Self::default();
            for i in 0..15isize {
                // The indices here wrap around
                let bit = if self.at(i) {
                    (self.at(i + 1) || !self.at(i + 2)) && (self.at(i - 1) || !self.at(i - 2))
                } else {
                    self.at(i - 1) && self.at(i + 1)
                };
                next_gen.set(i, bit);
            }
            next_gen
        }

        /// Creates a fungal automaton from the bits in an integer.
        pub fn new(v: u16) -> Self {
            Self(v)
        }

        pub fn as_u16(self) -> u16 {
            self.0
        }

        fn at(&self, index: isize) -> bool {
            let shift = (index % 15) & 0x1f;
            if shift < 16 {
                self.0 & (1 << shift) != 0
            } else {
                false
            }
        }

        fn set(&mut self, index: isize, v: bool) {
            if v {
                self.0 |= (v as u16) << index;
            }
        }
    }

    impl Into<u16> for FungalAutomaton {
        fn into(self) -> u16 {
            self.as_u16()
        }
    }
    #[cfg(test)]
    mod tests {
        use crate::fungal::FungalAutomaton;

        #[test]
        fn negative_overflow_is_correct() {
            assert_eq!(FungalAutomaton::new(14627).at(-1), false, "index -1");
            assert_eq!(FungalAutomaton::new(14627).at(-2), false, "index -2");
        }

        #[test]
        fn positive_overflow_is_correct() {
            assert_eq!(FungalAutomaton::new(14627).at(13 + 1), false, "index 13+1");
            assert_eq!(FungalAutomaton::new(14627).at(13 + 2), true, "index 13+2");
        }
    }
}

mod math {
    pub fn first_set(v: u16) -> i32 {
        15 - (v.leading_zeros() as i32)
    }
}

#[cfg(test)]
mod tests {
    use crate::LiquidData;
    use crate::PotionIngredient::{BlazePowder, FermentedSpiderEye, MagmaCream, SpiderEye, Sugar};

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

    #[test]
    fn wart_to_31011_is_correct() {
        assert_eq!(LiquidData(31011).apply_wart().0, 16675);
    }
}
