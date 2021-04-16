use std::{marker::PhantomData, net::SocketAddr, sync::Arc};

use bevy::{
    ecs::component::Component,
    prelude::{AppBuilder, IntoSystem, Plugin, Res},
    utils::HashSet,
};
use crossbeam_channel::{unbounded, Receiver};
use mio::{net::UdpSocket, Interest, Token};

use crate::core_plugin::{
    CorePluginState, Dispatcher, NetworkStage, NetworkedEvent, ReceiverMap, TokenStatus,
    TokenStatusMap,
};

const BUFFER_SIZE: usize = 64;

#[derive(Debug)]
pub struct SocketHandle<T = ()> {
    marker: PhantomData<T>,
    pub token: Token,
    pub socket: UdpSocket,
    pub queue: Receiver<NetworkedEvent>,
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

impl<T: Component> SocketPlugin<T> {
    fn dispatch_system(socket_handle: Res<SocketHandle<T>>, status_arc: Res<Arc<TokenStatusMap>>) {
        let status = status_arc
            .0
            .get(&socket_handle.token)
            .expect("could not find token status");

        for event in socket_handle.queue.try_iter() {
            let buf = bincode::serialize(&event).expect("could not serialize");

            for &target in &socket_handle.peer_addrs {
                if status.is_writable() {
                    match socket_handle.socket.send_to(&buf, target) {
                        Ok(bytes_sent) => {
                            // println!("sent {:?} -> {:?} bytes", event, bytes_sent);
                        }
                        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            status.set_writable(false);
                            break;
                        }
                        // TODO
                        Err(err) => panic!("{}", err),
                    }
                }
            }
        }
    }

    fn receive_system(
        event_receivers: Res<ReceiverMap>,
        socket_handle: Res<SocketHandle<T>>,
        status_arc: Res<Arc<TokenStatusMap>>,
    ) {
        let status = status_arc
            .0
            .get(&socket_handle.token)
            .expect("could not find token status");

        let mut buffer = [0_u8; BUFFER_SIZE];

        while status.is_readable() {
            // TODO: research error cases of receive
            match socket_handle.socket.recv_from(&mut buffer) {
                Ok((num_recv, _)) => {
                    // deserialize bytes
                    let event: NetworkedEvent =
                        bincode::deserialize(&buffer[0..num_recv]).expect(" could not deserialize");

                    if let Some(sender) = event_receivers.0.get(&event.id) {
                        sender
                            .send(event)
                            .expect("could not add event to receiver queue");
                    } else {
                        panic!("missing receiver from receiver map")
                    }
                }
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    status.set_readable(false);
                }
                Err(err) => panic!("{}", err),
            }
        }
    }
}

impl<T: Component> Plugin for SocketPlugin<T> {
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

        let (queue_sender, queue) = unbounded();

        // create socket
        let socket_handle: SocketHandle<T> = SocketHandle {
            marker: PhantomData,
            token,
            socket,
            queue,
            peer_addrs: self
                .peer_addrs
                .iter()
                .flat_map(|addr| addr.parse())
                .collect(),
        };

        app.insert_resource(socket_handle)
            .insert_resource(Dispatcher::<T>::new(queue_sender))
            .add_system_to_stage(NetworkStage::Receive, Self::receive_system.system())
            .add_system_to_stage(NetworkStage::Dispatch, Self::dispatch_system.system());
    }
}
