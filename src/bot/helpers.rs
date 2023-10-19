use poise::serenity_prelude::CreateEmbed;

pub async fn embed(
    ctx: crate::Context<'_>,
    builder: impl for<'b> FnOnce(&'b mut CreateEmbed) -> &'b mut CreateEmbed,
) -> Result<poise::ReplyHandle, serenity::Error> {
    ctx.send(|c| c.embed(|c| builder(c.color(crate::BERTRAM_COLOR))))
        .await
}
