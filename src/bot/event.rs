use poise::{serenity_prelude as serenity, Event, FrameworkContext};

pub async fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a Event<'a>,
    _framework: FrameworkContext<'a, crate::Data, crate::Error>,
    _data: &'a crate::Data,
) -> crate::Result<()> {
    match event {
        Event::Ready { .. } | Event::Resume { .. } => {
            println!("Bot ready!");
            ctx.set_presence(
                Some(serenity::Activity::playing(
                    "Rhythm Heaven Megamix tm for the Nintendo 3DS tm",
                )),
                serenity::OnlineStatus::Online,
            )
            .await;
        }
        _ => {}
    }
    Ok(())
}
