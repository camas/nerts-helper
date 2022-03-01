# NERTS! Online Specification

Not really a specification.

## NERTS! Online

NERTS! Online is a multiplayer card game that runs solely using the steamworks API. There are no other servers that it connects to. All games are P2P. All connections handled by Steam. Since steam handles everything all packets arrive intact and in sequence and can be treated as packets and not as a stream.

## Host/Client Setup

The host of the game controls the entire game, storing the state of every card face up or down, sending all server messages and receiving and handling all client messages.

Clients on the other hand send messages only to the host and know only what the host tells them, which is basically the minimum needed to render the game. The client does not know the values of face down cards and is not even sent ownership information for most of the cards, held cards being the only exception.

Both host and clients send each other messages at regular intervals, or when an action like drawing or clicking happens.

## Game phases

The game currently has 4 phases, we only really care about the play phase and I might have named the other ones wrong so don't take them at face value.

## Gameplay

Game is free so

## Messages

There is one message type for each direction. `ClientMessage` from client to server, `ServerMessage` from server to client. Both are serialized as simple binary data. A string is a 32-bit length followed by `length` bytes for example. See [reader.rs](/nerts-bot/src/messages/io/reader.rs) and [writer.rs](/nerts-bot/src/messages/io/writer.rs).

### Structure

See [client.rs](/nerts-bot/src/messages/client.rs) and [server.rs](/nerts-bot/src/messages/server.rs).

### Compression

Client messages are small enough (11 bytes) that they are sent as-is. Server messages on the other hand can be fairly big, so are compressed. Compression is indicated by the first byte in the raw packet being sent.

* `0` for a keyframe, which means no compression. This is sent on request from the client, or if the message size changes. Most of the time it doesn't.

* `1` for a regular, compressed frame.

To compress first the entire packet minus the compression flag is serialized. The size is checked against the previous message sent, if they are not the same compression isn't used. Next a byte-by-byte difference between the new and old packets are computed. This means that if the two are very similar most of the bytes will be `0`. Finally zlib is used to compress the bytes. If most of them are zero this will reduce size drastically ans saves having to think about sending different packets for updating different parts of the state.

Decompression follows the above steps in reverse and needs the previous, uncompressed, frame.
