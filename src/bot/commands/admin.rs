use std::process;

use poise::serenity_prelude as serenity;

#[poise::command(prefix_command)]
/// Kills the bot
pub async fn kill(ctx: crate::Context<'_>) -> crate::Result<()> {
    ctx.say("*poofs into smoke*").await?;
    ctx.serenity_context()
        .set_presence(None, serenity::OnlineStatus::Invisible)
        .await;
    process::exit(0);
}
