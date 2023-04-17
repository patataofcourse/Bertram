# Bertram
A helper bot for Megamix modding!

## Symbols
You'll need to add Megamix symbols files in the `sym` folder. An example (example_sym.csv) has been included. Required symbols are:
    - Rhythm Heaven Megamix (US): `sym/rhm.us.csv`
    - Rhythm Paradise Megamix (EU): `sym/rhm.eu.csv`
    - Rhythm Sesang Megamix (KR): `sym/rhm.kr.csv`

Symbols for all Saltwater stable versions will be included in the repository. For debug builds, please store them under `sym/sw._[COMMIT_HASH].csv`

## Rust version
Bertram requires nightly Rust due to let-chains and try_collect not being stable yet. Once let-chains and try_collect are in mainline Rust, I'll look into supporting it again.

As of the time of writing this readme, the latest working nightly is `nightly-2023-04-17`, although newer nightlies will likely work as well. Regarding older nightlies - no clue.

## Running
`cargo run` currently has some miscellaneous tests. To run Bertram, use `cargo run --bin bertram-bot --features bot` with the environment variable `DISCORD_TOKEN` set to the token of your bot. It's recommended to put this command and the environment variable setting in a sh or bat script (example included in run.sh.template)

You can also enable recompiling the bot through an owner-only command by setting environment variable `RECOMPILE` - keep in mind you'll have to rerun the bot in the script though (the exit code for this specific case is 99)

## Library
Feel free to use Bertram's library for your own projects or as code reference!

## rust-analyzer users
For some reason, rust-analyzer fails to evaluate the attribute macros for commands and the `mismatched-arg-count` diagnostic wrongly happens. If you don't disable that diagnostic or turn off rust-analyzer altogether, rust-analyzer will point errors where there are none.

If this stops happening, please let me know so I can take this warning off the README!
