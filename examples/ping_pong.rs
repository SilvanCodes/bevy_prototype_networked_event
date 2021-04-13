use bevy::{
    app::ScheduleRunnerPlugin,
    core::CorePlugin,
    prelude::{App, EventReader, EventWriter, IntoSystem, Res, ResMut, Time, Timer},
};
use bevy_prototype_networked_event::event_plugin::{Dispatch, Receive};
use std::env;
use std::time::SystemTime;

fn main() {
    let args: Vec<String> = env::args().collect();

    App::build()
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(CorePlugin::default())
        .add_plugin(bevy_prototype_networked_event::core_plugin::CorePlugin::default())
        .add_plugin(
            bevy_prototype_networked_event::socket_plugin::SocketPlugin::new(
                args[1].clone(),
                vec![args[2].clone()],
            ),
        )
        .add_plugin(bevy_prototype_networked_event::event_plugin::EventPlugin::<
            String,
        >::default())
        .add_plugin(bevy_prototype_networked_event::poll_plugin::PollPlugin::default())
        .add_system(spam_event_system.system())
        .add_system(print_event_system.system())
        .insert_resource(MyTimer(Timer::from_seconds(2.0, true)))
        .run();
}

struct MyTimer(Timer);

fn spam_event_system(
    time: Res<Time>,
    mut timer: ResMut<MyTimer>,
    mut my_events: EventWriter<Dispatch<String>>,
) {
    // update our timer with the time elapsed since the last update

    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // check to see if the timer has finished. if it has, print our message
    if timer.0.tick(time.delta()).finished() {
        // dbg!("timer finsihed, sending event");
        my_events.send(Dispatch(format!("{}", secs)));
    }
}

fn print_event_system(mut my_event_reader: EventReader<Receive<String>>) {
    for event in my_event_reader.iter()
    /* .filter(|e| e.is_remote()) */
    {
        println!("got event: {:?}", event.0);
    }
}
