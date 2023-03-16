use std::{env, process};

use poise::serenity_prelude::{self as serenity};

#[poise::command(prefix_command, category = "Admin", owners_only)]
/// Kills the bot
pub async fn kill(ctx: crate::Context<'_>) -> crate::Result<()> {
    ctx.say("*poofs into smoke*").await?;
    ctx.serenity_context()
        .set_presence(None, serenity::OnlineStatus::Invisible)
        .await;
    process::exit(0);
}

#[poise::command(prefix_command, category = "Admin", owners_only)]
/// Recompiles and reboots the bot
pub async fn recompile(ctx: crate::Context<'_>) -> crate::Result<()> {
    if let Ok(_) = env::var("RECOMPILE") {
        let m = ctx.say("Recompiling bot...").await?;
        process::Command::new("cargo")
            .arg("build")
            .arg("--features")
            .arg("bot")
            .output()?;
        m.into_message()
            .await?
            .channel_id
            .say(ctx, format!("Done! Resetting... <@{}>", ctx.author().id))
            .await?;
        process::exit(99)
    } else {
        ctx.say("Recompiling is not available right now!").await?;
        Ok(())
    }
}
