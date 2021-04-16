use std::sync::Arc;

use bevy::{prelude::Plugin, tasks::IoTaskPool};
use mio::Events;

use crate::core_plugin::{CorePluginState, TokenStatusMap};

const EVENT_CAPACITY: usize = 1024;

#[derive(Debug, Default)]
pub struct PollPlugin;

impl Plugin for PollPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        let mut poll = app
            .world_mut()
            .remove_resource::<CorePluginState>()
            .expect("could not find corepluginstate, add core plugin first")
            .poll;

        let token_status_map = app
            .world_mut()
            .remove_resource::<TokenStatusMap>()
            .expect("could not find corepluginstate, add core plugin first");

        // makes tokenStatusMap read only and shareable across threads
        let token_status_map_arc = Arc::new(token_status_map);
        let token_status_map_arc_clone = token_status_map_arc.clone();

        app.insert_resource(token_status_map_arc);

        let pool = app
            .world()
            .get_resource::<IoTaskPool>()
            .expect("failed to get iotaskpool");

        pool.spawn(async move {
            let mut events = Events::with_capacity(EVENT_CAPACITY);

            loop {
                // wait for os events
                poll.poll(&mut events, None).expect("could not poll");

                // check os events
                for event in &events {
                    if let Some(status) = token_status_map_arc_clone.0.get(&event.token()) {
                        // check if udp is readable
                        status.set_readable(event.is_readable());
                        // check if udp is writable
                        status.set_writable(event.is_writable());
                    }
                }
            }
        })
        .detach();
    }
}
