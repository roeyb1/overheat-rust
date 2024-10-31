pub mod life {
    use core::ops::{Div, Mul};
    use std::{fmt::Display, time::Duration};

    use bevy::prelude::Component;
    use derive_more::derive::{Add, AddAssign, Sub, SubAssign};
    use serde::{Deserialize, Serialize};

    use crate::ability::pool::{MaxPoolLessThanMin, Pool, RegeneratingPool};

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Add, Sub, AddAssign, SubAssign, Serialize, Deserialize)]
    pub struct Life(pub f32);

    #[derive(Component, Serialize, Deserialize)]
    pub struct LifePool {
        current: Life,
        max: Life,
        pub regen_per_second: Life,
    }

    impl LifePool {
        pub fn new(current: Life, max: Life, regen_per_second: Life) -> Self {
            assert!(current <= max);
            assert!(current >= LifePool::MIN);
            assert!(max >= LifePool::MIN);

            Self {
                current,
                max,
                regen_per_second,
            }
        }
    }

    impl Pool for LifePool {
        type Quantity = Life;
    
        const MIN: Life = Life(0.);
    
        fn current(&self) -> Self::Quantity {
            self.current
        }
    
        fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
            let actual = Life(new_quantity.0.clamp(0., self.max.0));
            self.current = actual;
            self.current
        }
    
        fn max(&self) -> Self::Quantity {
            self.max
        }

        fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin> {
            if new_max < Self::MIN {
                Err(MaxPoolLessThanMin)
            } else {
                self.max = new_max;
                self.set_current(self.current);
                Ok(())
            }
        }
    }
        

    impl RegeneratingPool for LifePool {
        fn regen_per_second(&self) -> Self::Quantity {
            self.regen_per_second
        }
    
        fn set_regen_per_second(&mut self, new_regen_per_second: Self::Quantity) {
            self.regen_per_second = new_regen_per_second;
        }
    
        fn regenerate(&mut self, delta_time: Duration) {
            self.set_current(self.current + self.regen_per_second * delta_time.as_secs_f32());
        }
    }




    impl Mul<f32> for Life {
        type Output = Life;

        fn mul(self, rhs: f32) -> Life {
            Life(self.0 * rhs)
        }
    }

    impl Mul<Life> for f32 {
        type Output = Life;

        fn mul(self, rhs: Life) -> Life {
            Life(self * rhs.0)
        }
    }

    impl Div<f32> for Life {
        type Output = Life;

        fn div(self, rhs: f32) -> Life {
            Life(self.0 / rhs)
        }
    }

    impl Div<Life> for Life {
        type Output = f32;

        fn div(self, rhs: Life) -> f32 {
            self.0 / rhs.0
        }
    }

    impl Display for Life {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Display for LifePool {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} / {}", self.current, self.max)
        }
    }
}