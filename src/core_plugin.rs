use std::{
    marker::PhantomData,
    ops::RangeFrom,
    sync::atomic::{AtomicBool, Ordering},
};

use bevy::{
    ecs::component::Component,
    prelude::{AppBuilder, CoreStage, Plugin, StageLabel, SystemStage},
    utils::HashMap,
};
use crossbeam_channel::{Receiver, Sender};
use downcast_rs::{impl_downcast, Downcast};
use mio::{Poll, Token};
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum NetworkStage {
    Receive,
    PostReceive,
    PreDispatch,
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

#[typetag::serde]
pub trait Networked: Component + Downcast + std::fmt::Debug {}
impl_downcast!(Networked);

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EventId {
    t: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkedEvent {
    pub id: EventId,
    pub data: Box<dyn Networked>,
}

#[derive(Debug, Default)]
pub struct ReceiverMap(pub HashMap<EventId, Sender<NetworkedEvent>>);

pub struct ReceiverQueue<T> {
    marker: PhantomData<T>,
    pub queue: Receiver<NetworkedEvent>,
}

impl<T> ReceiverQueue<T> {
    pub fn new(queue: Receiver<NetworkedEvent>) -> Self {
        Self {
            marker: PhantomData,
            queue,
        }
    }
}

pub struct Dispatcher<S> {
    marker: PhantomData<S>,
    pub sender: Sender<NetworkedEvent>,
}

impl<S> Dispatcher<S> {
    pub fn new(sender: Sender<NetworkedEvent>) -> Self {
        Self {
            marker: PhantomData,
            sender,
        }
    }
    pub fn dispatch(&self, msg: NetworkedEvent) {
        self.sender.try_send(msg).expect("could not dispatch")
    }
}

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
            .init_resource::<ReceiverMap>()
            .add_stage_before(
                CoreStage::Update,
                NetworkStage::PostReceive,
                SystemStage::parallel(),
            )
            .add_stage_before(
                NetworkStage::PostReceive,
                NetworkStage::Receive,
                SystemStage::parallel(),
            )
            .add_stage_after(
                CoreStage::Update,
                NetworkStage::PreDispatch,
                SystemStage::parallel(),
            )
            .add_stage_after(
                NetworkStage::PreDispatch,
                NetworkStage::Dispatch,
                SystemStage::parallel(),
            );
    }
}
