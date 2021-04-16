use std::{any::TypeId, marker::PhantomData};

use bevy::{
    app::Events,
    ecs::component::Component,
    prelude::{EventWriter, IntoSystem, Plugin, Res, ResMut},
};
use crossbeam_channel::unbounded;

use crate::core_plugin::{
    Dispatcher, EventId, NetworkStage, Networked, NetworkedEvent, ReceiverMap, ReceiverQueue,
};

#[derive(Debug)]
pub struct Dispatch<T: Networked>(pub T);

#[derive(Debug)]
pub struct Receive<T: Networked>(pub T);

#[derive(Debug)]
pub struct EventPlugin<T, S = ()>(PhantomData<T>, PhantomData<S>);

impl<T, S> Default for EventPlugin<T, S> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<T: Component + Networked, S: Component> EventPlugin<T, S> {
    fn receive_system(
        mut event_queue: EventWriter<Receive<T>>,
        receiver_queue: Res<ReceiverQueue<T>>,
    ) {
        for event in receiver_queue.queue.try_iter() {
            event_queue.send(Receive(*event.data.downcast().expect("downcast failed")))
        }
    }

    fn dispatch_system(
        mut event_queue: ResMut<Events<Dispatch<T>>>,
        dispatcher: Res<Dispatcher<S>>,
    ) {
        let type_id = TypeId::of::<T>();

        let event_type: EventId = unsafe { std::mem::transmute(type_id) };

        for event in event_queue.drain() {
            let networked_event = NetworkedEvent {
                id: event_type,
                data: Box::new(event.0),
            };

            dispatcher.dispatch(networked_event);
        }
    }
}

impl<T: Networked> Plugin for EventPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        let type_id = TypeId::of::<T>();

        let event_type: EventId = unsafe { std::mem::transmute(type_id) };

        let (sender, queue) = unbounded();

        let mut receiver_map = app
            .world_mut()
            .get_resource_mut::<ReceiverMap>()
            .expect("can't find receivermap, add core plugin first");

        receiver_map.0.insert(event_type, sender);

        app.add_event::<Receive<T>>()
            .add_event::<Dispatch<T>>()
            .insert_resource(ReceiverQueue::<T>::new(queue))
            .add_system_to_stage(NetworkStage::PostReceive, Self::receive_system.system())
            .add_system_to_stage(NetworkStage::PreDispatch, Self::dispatch_system.system());
    }
}
