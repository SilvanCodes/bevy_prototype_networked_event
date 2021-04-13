use std::marker::PhantomData;

use bevy::{app::PluginGroupBuilder, ecs::component::Component, prelude::PluginGroup};
use serde::{de::DeserializeOwned, Serialize};

pub mod core_plugin;
pub mod event_plugin;
pub mod poll_plugin;
pub mod socket_plugin;

pub use core_plugin::CorePlugin;
pub use event_plugin::{Dispatch, EventPlugin, Receive};
pub use poll_plugin::PollPlugin;
pub use socket_plugin::SocketPlugin;

pub struct NetworkedEventPlugins<T> {
    marker: PhantomData<T>,
    addr: String,
    peer_addrs: Vec<String>,
}

impl<T: Default + Component + Serialize + DeserializeOwned + std::fmt::Debug>
    NetworkedEventPlugins<T>
{
    pub fn new(addr: String, peer_addrs: Vec<String>) -> Self {
        Self {
            marker: PhantomData,
            addr,
            peer_addrs,
        }
    }
}

impl<T: Default + Component + Serialize + DeserializeOwned + std::fmt::Debug> PluginGroup
    for NetworkedEventPlugins<T>
{
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(CorePlugin::default())
            .add(SocketPlugin::new(
                self.addr.clone(),
                self.peer_addrs.clone(),
            ))
            .add(EventPlugin::<T>::default())
            .add(PollPlugin::default());
    }
}
