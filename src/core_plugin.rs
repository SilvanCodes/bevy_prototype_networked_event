use std::{
    ops::RangeFrom,
    sync::atomic::{AtomicBool, Ordering},
};

use bevy::{
    prelude::{AppBuilder, CoreStage, Plugin, StageLabel, SystemStage},
    utils::HashMap,
};
use mio::{Poll, Token};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum NetworkStage {
    Receive,
    Dispatch,
}

#[derive(Debug)]
pub struct TokenStatus {
    readable: AtomicBool,
    writable: AtomicBool,
    ordering: Ordering,
}

impl Default for TokenStatus {
    fn default() -> Self {
        Self {
            readable: AtomicBool::new(false),
            writable: AtomicBool::new(false),
            ordering: Ordering::Relaxed,
        }
    }
}

impl TokenStatus {
    #[inline]
    pub fn is_readable(&self) -> bool {
        self.readable.load(self.ordering)
    }

    #[inline]
    pub fn set_readable(&self, val: bool) {
        self.readable.store(val, self.ordering);
    }

    #[inline]
    pub fn is_writable(&self) -> bool {
        self.writable.load(self.ordering)
    }

    #[inline]
    pub fn set_writable(&self, val: bool) {
        self.writable.store(val, self.ordering);
    }
}

#[derive(Debug, Default)]
pub struct TokenStatusMap(pub HashMap<Token, TokenStatus>);

pub struct CorePluginState {
    pub token_gen: RangeFrom<usize>,
    pub poll: Poll,
}

impl Default for CorePluginState {
    fn default() -> Self {
        Self {
            token_gen: 0..,
            poll: Poll::new().expect("could not create poll"),
        }
    }
}

#[derive(Debug, Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CorePluginState>()
            .init_resource::<TokenStatusMap>()
            .add_stage_before(
                CoreStage::Update,
                NetworkStage::Receive,
                SystemStage::parallel(),
            )
            .add_stage_after(
                CoreStage::Update,
                NetworkStage::Dispatch,
                SystemStage::parallel(),
            );
    }
}
