use std::process;

use serenity::model::user::OnlineStatus;
use serenity::prelude::*;

use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::channel::Message,
};

#[group]
#[commands(kill)]
struct Admin;

#[command]
#[owners_only]
/// Kills the bot
async fn kill(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "*poofs into smoke*").await?;
    ctx.set_presence(None, OnlineStatus::Invisible).await;
    process::exit(0);
}
