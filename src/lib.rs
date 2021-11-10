use std::marker::PhantomData;

use bevy::{app::PluginGroupBuilder, prelude::{AppBuilder, PluginGroup}};

pub mod core_plugin;
pub mod event_plugin;
pub mod poll_plugin;
pub mod socket_plugin;

pub use core_plugin::{CorePlugin, Networked};
pub use event_plugin::{Dispatch, EventPlugin, Receive};
pub use poll_plugin::PollPlugin;
use socket_plugin::Socket;
pub use socket_plugin::SocketPlugin;

pub struct NetworkedEventPlugins<T> {
    marker: PhantomData<T>,
    addr: String,
    peer_addrs: Vec<String>,
}

pub trait SetupNetworkedEvent {
    fn add_networked_socket<S: Socket>(
        &mut self,
        addr: String,
        peer_addrs: Vec<String>,
    ) -> &mut Self;
    fn add_networked_event<T: Networked, S: Socket>(&mut self) -> &mut Self;
    fn add_networked_core(&mut self) -> &mut Self;
    fn add_networked_loop(&mut self) -> &mut Self;
}

impl SetupNetworkedEvent for AppBuilder {
    fn add_networked_event<T: Networked, S: Socket>(&mut self) -> &mut Self {
        self.add_plugin(EventPlugin::<T, S>::default())
    }

    fn add_networked_socket<S: Socket>(
        &mut self,
        addr: String,
        peer_addrs: Vec<String>,
    ) -> &mut Self {
        self.add_plugin(SocketPlugin::<S>::new(
            addr.into(),
            peer_addrs.into_iter().map(|t| t.into()).collect(),
        ))
    }

    fn add_networked_core(&mut self) -> &mut Self {
        self.add_plugin(CorePlugin::default())
    }

    fn add_networked_loop(&mut self) -> &mut Self {
        self.add_plugin(PollPlugin::default())
    }
}

impl<T: Networked> NetworkedEventPlugins<T> {
    pub fn new(addr: String, peer_addrs: Vec<String>) -> Self {
        Self {
            marker: PhantomData,
            addr,
            peer_addrs,
        }
    }
}

impl<T: Networked> PluginGroup for NetworkedEventPlugins<T> {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(CorePlugin::default())
            .add(SocketPlugin::<()>::new(
                self.addr.clone(),
                self.peer_addrs.clone(),
            ))
            .add(EventPlugin::<T, ()>::default())
            .add(PollPlugin::default());
    }
}
