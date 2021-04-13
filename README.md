plugin creates poll

"register" method on plugin for sockets
=> builds a token->(arc, arc) map and token->socket_state map

socket_state ist (socket, arc, arc)

allows to build multiple sockets being polled

poll startup system moves poll+arc_map from plugin to poll loop

place socket_state_map as global resource

socket_id is mio::Token

how to send/receive from multiple sockets
=> wanted: one read and one write system per socket, read emit bevy events (socket_id, received_bytes), write expects bevy events (socket_id, bytes_to_send)

manually build system with Local<mio::Token> resource and initialize those resources accordingly

serialize/deserialize systems for (socket_id, type) pairs, those emit actual received data as bevy events

produced by serde_system::<T>::serialize/deserialize ??

every system on socket_id tries deserialization, emits bevy event of own type on success

three plugins:

NetworkedEventCorePlugin

- token_gen resource
- poll resource
- poll loop kick-off
- tokenstatusmap resource

NetworkedEventSocketPlugin

- add configured socket resource

NetworkedEvent<T>Plugin

- add Send<T> and Receive<T> event (resource)
- add send and receive systems
