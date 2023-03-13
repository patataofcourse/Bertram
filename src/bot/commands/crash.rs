use serenity::prelude::*;

use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};

#[group("Crash Handler")]
#[commands(luma)]
struct CrashHandler;

#[command]
pub async fn luma(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args_raw = args.raw().collect::<Vec<&str>>();
    msg.reply(ctx, format!("{:?}", args_raw)).await?;
    Ok(())
}
