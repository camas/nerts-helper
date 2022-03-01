# Bot Specs

Since the protocol is quite simple, so is the bot. The bot so far can only act as a client as that requires the least effort. Most game logic is left to the server to handle.

When the bot is created it starts a few threads:

* Steam callback loop - Constantly polls the steam api so that callbacks work.

* Send loop - Sends client messages at regular intervals or when requested.

* Receive loop - Receives messages from the server and processes them.

The bot then returns a handle instead of it's own struct when created for cross-thread access. The send and receive loops also use these handles. Careful when using this handle as it's quite easy to cause a deadlock.

To parse the messages from the server the bot uses hardcoded offsets to sort the cards based on their position as ownership data is only sent when a card is being held. These offsets haven't changed in a while, but could. If they do the bot should crash.

After parsing the bot exposes the current state of the game in a vaguely usable form through the `state` field. Here the state is layed out fairly intuitively, with a set of players who each own their own cards, plus the shared spaces in the center.

To perform actions the client reads a few variables from the state and sends them back to the server in a ClientMessage either at intervals or when `send_client_message` is called. See:`Bot::create_client_message`. This could probably be made more user friendly.
