use std::{marker::PhantomData, sync::Arc};

use bevy::{
    ecs::component::Component,
    prelude::{EventReader, EventWriter, IntoSystem, Plugin, Res},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    core_plugin::{NetworkStage, TokenStatusMap},
    socket_plugin::SocketHandle,
};

const BUFFER_SIZE: usize = 64;

#[derive(Debug)]
pub struct Dispatch<T>(pub T);

#[derive(Debug)]
pub struct Receive<T>(pub T);

#[derive(Debug, Default)]
pub struct EventPlugin<T>(PhantomData<T>);

impl<T: Component + Serialize + DeserializeOwned + std::fmt::Debug> EventPlugin<T> {
    fn dispatch_system(
        mut event_queue: EventReader<Dispatch<T>>,
        socket_handle: Res<SocketHandle>,
        status_arc: Res<Arc<TokenStatusMap>>,
    ) {
        let status = status_arc
            .0
            .get(&socket_handle.token)
            .expect("could not find token status");

        for event in event_queue.iter() {
            let buf = bincode::serialize(&event.0).expect("could not serialize");

            for &target in &socket_handle.peer_addrs {
                if status.is_writable() {
                    match socket_handle.socket.send_to(&buf, target) {
                        Ok(bytes_sent) => {
                            println!("sent {:?} -> {:?} bytes", event.0, bytes_sent);
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
        mut event_queue: EventWriter<Receive<T>>,
        socket_handle: Res<SocketHandle>,
        status_arc: Res<Arc<TokenStatusMap>>,
    ) {
        let status = status_arc
            .0
            .get(&socket_handle.token)
            .expect("could not find token status");

        let mut buffer = [0_u8; BUFFER_SIZE];

        loop {
            if status.is_readable() {
                // dbg!("socket is readable");
                // TODO: research error cases of receive
                match socket_handle.socket.recv_from(&mut buffer) {
                    Ok((num_recv, from_addr)) => {
                        println!(
                            "received {:?} -> {:?} from {:?}",
                            buffer, num_recv, from_addr
                        );

                        // deserialize bytes
                        let event: T = bincode::deserialize(&buffer[0..num_recv])
                            .expect(" could not deserialize");

                        event_queue.send(Receive(event));
                    }
                    Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        status.set_readable(false);
                        break;
                    }
                    Err(err) => panic!("{}", err),
                }
            } else {
                break;
            }
        }
    }
}

impl<T: Component + Serialize + DeserializeOwned + std::fmt::Debug> Plugin for EventPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_event::<Receive<T>>()
            .add_event::<Dispatch<T>>()
            .add_system_to_stage(NetworkStage::Receive, Self::receive_system.system())
            .add_system_to_stage(NetworkStage::Dispatch, Self::dispatch_system.system());
    }
}
