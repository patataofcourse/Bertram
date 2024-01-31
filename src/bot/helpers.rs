use poise::{serenity_prelude::CreateEmbed, CreateReply};

pub async fn embed(
    ctx: crate::Context<'_>,
    builder: impl for<'b> FnOnce(CreateEmbed) -> CreateEmbed,
) -> Result<poise::ReplyHandle, serenity::Error> {
    ctx.send(CreateReply::default().embed(builder(CreateEmbed::new().color(crate::BERTRAM_COLOR))))
        .await
}
