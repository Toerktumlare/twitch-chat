## twitch-chat

this is a tiny TUI twitch chat listener that runs in the terminal and prints the chat messages.

### How to run
If you want to run this there are some steps that needs to be taken.

- Clone or fork the repository
- Checkout the master branch
- find the main.rs file and edit it and change the `NICK` value to your nickname on twitch
- Build the project using cargo build
- Generate a twitch api token by going to https://twitchapps.com/tmi/ and connect
- set this token as an environmental variable named `TWITCH_API_TOKEN`
- go to the target folder, and either debug or release depending on what type of build you did
- run:

```
twitch-chat <channel_name>
```

where `<channel_name>` is the name of the streamers chat you want to connect to.

> At the moment there is no parameter to set the name of who you are logged in as. So you need to change the NICK parameter in main.rs to the name of your account manually before you compile.


## TODO:
- make `NICK` parameter an input argument
- breakout application from main method
- return `Result<T>` from both twitch lister and event listener
- add `--debug` flag so we can read debug messages in chat, including broken chat messages
- fix parsing of twitch reply chat messages
- add unicode stuff for moderators
- identify smileys and replace with unicode
- add a unicode symbol that represents Kappa
