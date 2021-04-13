# Basics

This crate experiments with networking possibilities for bevy.

So far it is only capable of the most basic operation which is serializing -> transmitting -> deserializing, without any guarantees whatsoever.

It does so by given you a `EventWriter<Dispatch<T>>` for sending events over the network and an `EventReader<Receive<T>>` for receiving events.

It looks like this to setup:

```rust
    App::build()
        // ...
        // String is our type to be transmitted
        .add_plugins(NetworkedEventPlugins::<String>::new(
            String::from("127.0.0.0:8000"),       // ip to listen on
            vec![String::from("127.0.0.0:8001")], // ips to send to
        ))
        // ...
        .run();
```

And like this to use:

```rust
    fn dispatch_over_network_system(
        mut my_event_dispatcher: EventWriter<Dispatch<String>>,
    ) {
        my_events.send(Dispatch(String::from("Hello!")));
    }

    fn receive_from_network_system(
        mut my_event_receiver: EventReader<Receive<String>>,
    ) {
        for event in my_event_receiver.iter() {
            println!("got event: {:?}", event);
        }
    }
```

# Obvious caveats

BUFFER_SIZE for deserialization is hardcoded to a small value, adjust to your needs when experimenting.

EVENT_CAPACITY for mio is also arbitrarily hardcoded.

There are no guarantees for order of delivery or delivery at all. (plain UDP + events are skipped when the socket is unavailable)

# Architecture

Sofar the internals are structured in four plugins:

## CorePlugin

- setup stages and basic data structures

## SocketPlugin

- add (UDP) sockets with ip configuration

## EventPlugin

- add Events and systems for transmitting them

## PollPlugin

- kickoff the poll-loop for mio <-> os interaction
- has to be called last ond just once!
