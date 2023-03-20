use std::{env, process};

use poise::serenity_prelude::{self as serenity};

/// Kills the bot
#[poise::command(prefix_command, category = "Admin", owners_only)]
pub async fn kill(ctx: crate::Context<'_>) -> crate::Result<()> {
    ctx.say("*poofs into smoke*").await?;
    ctx.serenity_context()
        .set_presence(None, serenity::OnlineStatus::Invisible)
        .await;
    process::exit(0);
}

/// Recompiles and reboots the bot
#[poise::command(prefix_command, category = "Admin", owners_only)]
pub async fn recompile(ctx: crate::Context<'_>) -> crate::Result<()> {
    if env::var("RECOMPILE").is_ok() {
        let m = ctx.say("Recompiling bot...").await?;
        process::Command::new("cargo")
            .arg("build")
            .arg("--features")
            .arg("bot")
            .stderr(process::Stdio::inherit())
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
