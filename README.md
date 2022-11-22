# patatbot
Ready to use base for discord bots

## What is this?
The result of me fucking with Discord bots since the year 2019 :D

## How to use
- Install discord.py with `python -m pip install discord.py`
- Fill in the values at constants.py
    - `BOT_NAME` (string) - name of your bot
    - `BOT_VERSION` (string) - current version of your bot
    - `BOT_PREFIX` (string) - default command prefix for the bot (for example `!` if you want your commands to be like this: `!command`)
    - `BOT_ACTIVITY` (string) - text to be displayed in the "Playing _" status.
    - `BOT_OWNERS` (tuple) - Discord IDs of users to give owner permissions to. They will be able to use every command.
    - `BOT_ERROR_CHANNEL` (int) - Discord ID of the channel to send errors and debug infos to
    - `BOT_HELPERS` (tuple) - Discord IDs of users to give "helper" (mantainer) permissions to. They will be able to use a wider variety of commands to manage the bot. Helpful if you have your bot running on someone else's server.
    - `BOT_COLOR` (int) - RGB color to use for builtin embeds.
    - The other fields allow you to customize text in the builtin modules
- Create a file named tokens.py with the following contents[1]:
```py
bot = "YOUR_BOT_TOKEN_GOES_HERE"
```
- Create files inside the module folder with your commands, etc. (module/test.py is available as an example)
- Run the bot with `python main.py`

[^1] NEVER publish your Discord bot token! `tokens.py` will not be automatically added to the git repo by design

## License
This project is licensed under the WTFPL (Do What The Fuck You Want To Public License).