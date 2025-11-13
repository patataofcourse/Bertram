# Bertram
A helper bot for Rhythm Heaven Megamix modding!

## Symbols
You'll need to add Megamix symbols files in the `sym` folder. An example (example_sym.csv) has been included. Required symbols are:

- Rhythm Tengoku: The Best + (JP, partial support): `sym/rhm.jp.csv`
- Rhythm Heaven Megamix (US): `sym/rhm.us.csv`
- Rhythm Paradise Megamix (EU): `sym/rhm.eu.csv`
- Rhythm Sesang: The Best + (KR): `sym/rhm.kr.csv`

Symbols for all Saltwater stable versions (aside from 0.1.x) will be included in the repository. For debug builds, please store them under `sym/sw._[COMMIT_HASH].csv` and do not force them to enter the repository. Storing symbols for every single Saltwater debug version would not only be a waste of space, but it would most likely not be very useful.

## Rust version
Bertram requires nightly Rust due to `Iterator::try_collect` not being stable yet. Once it is in stable Rust, I'll look into supporting stable again.

As of the time of writing this readme, the latest working nightly is `nightly-2025-08-08`, although newer nightlies will likely work as well. Regarding older nightlies - no clue.

## Running
`cargo run` currently builds `bertram-test` by default, which are some offline tests on Bertram's library. To run Bertram as a bot, use `cargo run --bin bertram-bot --features bot` with the environment variable `DISCORD_TOKEN` set to the token of your bot. It's recommended to put this command and the environment variable setting in a sh or bat script. You can use the provided `run.sh.template` as a starting point.

You can also enable recompiling the bot through an owner-only command by setting environment variable `RECOMPILE` - keep in mind you'll have to rerun the bot in the script though (the exit code for this specific case is 99). Again, this is written in `run.sh.template`, although commented.

## Library
Feel free to use Bertram's library for your own projects or as code reference! Documentation isn't really a thing here for now, but it might still prove useful.

## rust-analyzer users
For some reason, rust-analyzer fails to evaluate the attribute macros for commands and the `mismatched-arg-count` diagnostic wrongly happens. If you don't disable that diagnostic or turn off rust-analyzer altogether, rust-analyzer will point errors where there are none.

If disabling `mismatched-arg-count` doesn't work, try disabling `E0107` instead.

If this stops happening, please let me know so I can take this warning off the README!

## Licensing
Bertram is licensed under the LGPL-3 license. See the [LICENSE](./LICENSE) file for more information.