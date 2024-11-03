use std::{fmt::Display, time::Duration};

use bevy::{prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

use super::CannotUseAbility;

#[derive(Component, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, Reflect)]
pub struct Cooldown {
    cd: Duration,
    /// Time elapsed since the cooldown was last triggered
    elapsed: Duration,
}

#[allow(unused)]
impl Cooldown {
    pub fn new(cd: Duration) -> Self {
        assert!(cd != Duration::ZERO);

        Self {
            cd,
            elapsed: cd
        }
    }

    pub fn from_secs(cd: f32) -> Self {
        Self::new(Duration::from_secs_f32(cd))
    }

    pub fn tick(&mut self, delta_time: Duration) {
        if self.elapsed == self.cd {
            return;
        }

        self.elapsed = self.elapsed.saturating_add(delta_time).min(self.cd);
    }

    /// Returns true if the action is ready to be used.
    pub fn ready(&self) -> Result<(), CannotUseAbility> {
        if self.elapsed >= self.cd {
            Ok(())
        } else {
            Err(CannotUseAbility::OnCooldown)
        }
    }

    /// Reset the cooldown. The ability will be ready immediately.
    #[inline]
    pub fn refresh(&mut self) {
        self.elapsed = self.cd;
    }

    /// Use the cooldown if and only if it is ready, then begins the cooldown
    #[inline]
    pub fn trigger(&mut self) -> Result<(), CannotUseAbility> {
        self.ready()?;
        self.elapsed = Duration::ZERO;

        Ok(())
    }

    pub fn remaining(&self) -> Duration {
        self.cd.saturating_sub(self.elapsed)
    }
}

impl Display for Cooldown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} / {:?}", self.elapsed, self.cd)
    }
}