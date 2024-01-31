use poise::{
    serenity_prelude::{self as serenity, FullEvent as Event},
    FrameworkContext,
};

pub async fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a Event,
    _framework: FrameworkContext<'a, crate::Data, crate::Error>,
    _data: &'a crate::Data,
) -> crate::Result<()> {
    match event {
        Event::Ready { .. } | Event::Resume { .. } => {
            println!("Bot ready!");
            ctx.set_presence(
                Some(serenity::ActivityData::playing(
                    "Rhythm Heaven Megamix tm for the Nintendo 3DS tm",
                )),
                serenity::OnlineStatus::Online,
            );
        }
        _ => {}
    }
    Ok(())
}
