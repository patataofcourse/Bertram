# Bertram
A helper bot for Megamix modding!

Uses [patatbot](https://github.com/patataofcourse/patatbot/)

## Setting up
- This bot currently has some hardcoded private constants in `private.py`, which is not included in this repo. If you want to use this bot, wait until the need for `private.py` is removed, or figure it out yourself
- The token for the bot is in a file called `tokens.py` at the root of the folder. All it needs is a single constant named `bot`, which will be the token of your bot
- You'll need:
    - A CSV list of symbols for US RHM in `sym/rhm.us.csv` (not included due to the sheer size of it), which can be exported from Ghidra's Symbol Table window. See `sym/example_sym.csv` for more information.
    - A US Megamix code.bin in `sym/rhm.us.code`