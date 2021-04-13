use std::{any::Any, marker::PhantomData, net::SocketAddr};

use bevy::{
    prelude::{AppBuilder, Plugin},
    utils::HashSet,
};
use mio::{net::UdpSocket, Interest, Token};

use crate::core_plugin::{CorePluginState, TokenStatus, TokenStatusMap};

#[derive(Debug)]
pub struct SocketHandle<T: std::fmt::Debug = ()> {
    marker: PhantomData<T>,
    pub token: Token,
    pub socket: UdpSocket,
    pub peer_addrs: HashSet<SocketAddr>,
}

pub struct SocketPlugin<T = ()> {
    marker: PhantomData<T>,
    addr: String,
    peer_addrs: Vec<String>,
}

impl SocketPlugin {
    pub fn new(addr: String, peer_addrs: Vec<String>) -> Self {
        Self {
            marker: PhantomData,
            addr,
            peer_addrs,
        }
    }
}

impl<T: Any + Send + Sync + std::fmt::Debug> Plugin for SocketPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        let mut socket =
            UdpSocket::bind(self.addr.parse().expect("could not parse socket address"))
                .expect("could not bind socket");

        let mut core_plugin_state = app
            .world_mut()
            .get_resource_mut::<CorePluginState>()
            .expect("can't find corepluginstate, add core plugin first");

        let token = core_plugin_state.token_gen.next().map(Token).unwrap();

        core_plugin_state
            .poll
            .registry()
            .register(&mut socket, token, Interest::READABLE | Interest::WRITABLE)
            .expect("could not register socket for poll");

        let mut token_status_map = app
            .world_mut()
            .get_resource_mut::<TokenStatusMap>()
            .expect("can't find tokenstatusmap, add core plugin first");

        token_status_map.0.insert(token, TokenStatus::default());

        // create socket
        let socket_handle: SocketHandle<T> = SocketHandle {
            marker: PhantomData,
            token,
            socket,
            peer_addrs: self
                .peer_addrs
                .iter()
                .flat_map(|addr| addr.parse())
                .collect(),
        };

        app.insert_resource(socket_handle);
    }
}
