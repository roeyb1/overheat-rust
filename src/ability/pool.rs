use core::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

use bevy::{prelude::{Component, Query, Res}, time::Time};
use serde::{Deserialize, Serialize};

use super::CannotUseAbility;

pub struct MaxPoolLessThanMin;

pub trait Pool: Sized {
    type Quantity: Add<Output = Self::Quantity>
        + Sub<Output = Self::Quantity>
        + AddAssign
        + SubAssign
        + PartialEq
        + PartialOrd
        + Clone
        + Copy
        + Send
        + Sync
        +'static;

    const MIN: Self::Quantity;

    fn current(&self) -> Self::Quantity;

    fn available(&self, amount: Self::Quantity) -> Result<(), CannotUseAbility> {
        if self.current() >= amount {
            Ok(())
        } else {
            Err(CannotUseAbility::ResourceMissing)
        }
    }

    fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity;

    fn max(&self) -> Self::Quantity;

    fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin>;

    fn expend(&mut self, amount: Self::Quantity) -> Result<(), CannotUseAbility> {
        self.available(amount)?;

        let new = self.current() - amount;
        self.set_current(new);
        Ok(())
    }

    fn replenish(&mut self, amount: Self::Quantity) {
        let new = self.current() + amount;
        self.set_current(new);
    }
}

pub trait RegeneratingPool: Pool {
    fn regen_per_second(&self) -> Self::Quantity;

    fn set_regen_per_second(&mut self, new_regen_per_second: Self::Quantity);

    fn regenerate(&mut self, delta_time: Duration);
}

pub struct AbilityCost<P: Pool>(pub P::Quantity);

#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct NullPool;

impl Pool for NullPool {
    type Quantity = f32;

    const MIN: f32 = 0.;

    fn current(&self) -> Self::Quantity {
        Self::MIN
    }

    fn set_current(&mut self, _new_quantity: Self::Quantity) -> Self::Quantity {
        Self::MIN
    }

    fn max(&self) -> Self::Quantity {
        Self::MIN
    }

    fn set_max(&mut self, _new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin> {
        Ok(())
    }
}

pub fn tick_pools_regen<P: RegeneratingPool + Component>(
    mut query: Query<&mut P>,
    time: Res<Time>
) {
    let delta_time = time.delta();

    for mut pool in query.iter_mut() {
        pool.regenerate(delta_time);
    }
}