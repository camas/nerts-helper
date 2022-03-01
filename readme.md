# Nerts Helper

This is a '''''''''helper'''''''' for the game [NERTS! Online](https://store.steampowered.com/app/1131190/NERTS_Online/).

Currently it's a bit dumb but since reaction time is limited only by ping it still has a bit of an advantage.

## Media

Most of the time the bot can't play so just imagine a cursor flicking around the screen, the deck being constantly shuffled and the cards randomly changing color.

## Build/run

To run steam also needs to be running as NERTS! Online uses the steam apis to run everything.

Building requires the `steamworks-sdk` crate with the two changes I've made in [ee3840f3](https://github.com/camas/steamworks-rs/commit/ee3840f3eac2ecdc80e529303ce26ddc08f2e8a4) and [de336efd](https://github.com/camas/steamworks-rs/commit/de336efd0dcfac2dcd30b0200633525f514268ce).

```shell
cargo build
cargo run
```

## Technical Information

NERTS! Online is a unity game, no il2cpp so to decompile yourself just open `GameAssembly.dll` with dnSpy.

[bot-specs.md](/bot-specs.md) - Quick overview of how the bot works internally.

[specs.md](/specs.md) - Overview of how NERTS! Online works.
