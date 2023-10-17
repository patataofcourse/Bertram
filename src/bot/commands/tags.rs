use bertram::faq::RESOURCES_TAG;
use poise::serenity_prelude::{self as serenity};

use crate::helpers::embed;

/// Tag that leads to resources-and-guides
#[poise::command(prefix_command, category = "Admin", owners_only)]
pub async fn docs(ctx: crate::Context<'_>) -> crate::Result<()> {
    embed(ctx, |c| {
        c.title("idk what to make the title of this")
            .description(RESOURCES_TAG)
    })
    .await?;
    Ok(())
}
