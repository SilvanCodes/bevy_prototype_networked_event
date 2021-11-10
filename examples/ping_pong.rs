use bevy::{
    app::ScheduleRunnerPlugin,
    core::CorePlugin,
    prelude::{App, EventReader, EventWriter, IntoSystem, Res, ResMut, Time, Timer},
};
use bevy_prototype_networked_event::{
    Dispatch, Networked, NetworkedEventPlugins, Receive, SetupNetworkedEvent,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::SystemTime;

struct MySocketIdentifier;

fn main() {
    let args: Vec<String> = env::args().collect();

    App::build()
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(CorePlugin::default())
        .add_networked_core()
        .add_networked_socket::<()>(args[1].clone().to_string(), vec![args[2].clone().to_string()])
        .add_networked_event::<Ping, ()>()
        .add_networked_event::<Pong, ()>()
        .add_networked_loop()
        /* .add_plugins(NetworkedEventPlugins::<MyMessage>::new(
            args[1].clone(),
            vec![args[2].clone()],
        )) */
        .add_system(spam_event_system.system())
        .add_system(print_event_system.system())
        .insert_resource(MyTimer(Timer::from_seconds(2.0, true)))
        .run();
}

struct MyTimer(Timer);

#[derive(Debug, Deserialize, Serialize)]
struct Ping;

#[typetag::serde]
impl Networked for Ping {}

#[derive(Debug, Deserialize, Serialize)]
struct Pong;

#[typetag::serde]
impl Networked for Pong {}

fn spam_event_system(
    time: Res<Time>,
    mut timer: ResMut<MyTimer>,
    mut ping_dispatcher: EventWriter<Dispatch<Ping>>,
) {
    // check to see if the timer has finished. if it has, print our message
    if timer.0.tick(time.delta()).finished() {
        dbg!("sending ping");
        ping_dispatcher.send(Dispatch(Ping));
    }
}

fn print_event_system(
    mut ping_receiver: EventReader<Receive<Ping>>,
    mut pong_receiver: EventReader<Receive<Pong>>,
    mut pong_dispatcher: EventWriter<Dispatch<Pong>>,
) {
    for _ in ping_receiver.iter() {
        dbg!("got ping, sending pong");
        pong_dispatcher.send(Dispatch(Pong));
    }
    for _ in pong_receiver.iter() {
        dbg!("got pong!");
    }
}
