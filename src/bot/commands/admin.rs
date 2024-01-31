use std::{env, process};

use poise::serenity_prelude::{self as serenity};

use crate::helpers::embed;

/// Kills the bot
#[poise::command(prefix_command, category = "Admin", check = "crate::op_check")]
pub async fn kill(ctx: crate::Context<'_>) -> crate::Result<()> {
    ctx.say("*poofs into smoke*").await?;
    ctx.serenity_context()
        .set_presence(None, serenity::OnlineStatus::Invisible);
    process::exit(0);
}

/// Recompiles and reboots the bot
#[poise::command(prefix_command, category = "Admin", check = "crate::op_check")]
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

#[poise::command(prefix_command, category = "Admin", check = "crate::op_check")]
pub async fn info(ctx: crate::Context<'_>) -> crate::Result<()> {
    //TODO: parse nightlies to use format "nightly-YYYY-MM-DD (rust 1.XX)""
    let rustc_ver = (|| {
        String::from_utf8(
            process::Command::new("rustc")
                .arg("-V")
                .output()
                .ok()?
                .stdout,
        )
        .ok()
    })();
    let commit = (|| {
        String::from_utf8(
            process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()?
                .stdout,
        )
        .ok()
    })();
    let hostname =
        (|| String::from_utf8(process::Command::new("hostname").output().ok()?.stdout).ok())();
    embed(ctx, |e| {
        e.title("Bertram info")
            .field(
                "Running on:",
                hostname
                    .map(|c| format!("`{c}`"))
                    .unwrap_or("unavailable".to_string()),
                false,
            )
            .field(
                "rustc version",
                format!(
                    "on build: {}\ncurrent: {}",
                    crate::RUSTC_AT_BUILD,
                    if let Some(c) = rustc_ver {
                        c
                    } else {
                        "unavailable".to_string()
                    }
                ),
                false,
            )
            .field(
                "Bertram commit",
                format!(
                    "on build: {}\ncurrent: {}",
                    crate::COMMIT_AT_BUILD,
                    if let Some(c) = commit {
                        c
                    } else {
                        "unavailable".to_string()
                    }
                ),
                false,
            )
    })
    .await?;
    Ok(())
}
