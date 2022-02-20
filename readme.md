## twitch-chat

this is a tiny TUI twitch chat listener that runs in the terminal and prints the chat messages.

### How to run
If you want to run this there are some steps that needs to be taken.

- Clone or fork the repository
- Checkout the master branch
- Build the project using cargo build
- Generate a twitch api token by going to https://twitchapps.com/tmi/ and connect
- set this token as an environmental variable named `TWITCH_API_TOKEN`
- go to the target folder, and either debug or release depending on what type of build you did
- run:

```
twitch-chat --nick=<nick> --channel=<channel>
```

where `<nick>` is the nickname of your account the token is issued for and `<channel>` is the name of the streamers chat you want to connect to.

## TODO:
- ~~make `NICK` parameter an input argument~~
- ~~breakout application from main method~~
- ~~add a unicode symbol that represents Kappa~~
- ~~add file logger, to be able to tail logs~~
- remove tungstenite-rs in favor of ws-rs
- ability to set log levels in logger
- ability to include some context while logging
- Implement `Drop` for `EventHandler`
- identify smileys and replace with unicode
- add `--debug` flag so we can read debug messages in chat, including broken chat messages
- fix parsing of twitch reply chat messages
- add unicode stuff for moderators
- some day solve consistent name color (twitch not sending default colors)
- remember to never implement log rotation
