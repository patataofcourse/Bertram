use std::collections::HashSet;
use std::env;

use poise::serenity_prelude::CacheHttp;
use poise::{
    serenity_prelude as serenity, Framework, FrameworkError, FrameworkOptions,
    PrefixFrameworkOptions,
};

use poise::macros::command;

//pub mod commands;

#[derive(Debug)]
struct Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Result = std::result::Result<(), Error>;

#[tokio::main]
async fn main() {
    let framework = Framework::builder()
        .options(FrameworkOptions {
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".into()),
                mention_as_prefix: true,
                ..Default::default()
            },
            commands: vec![ping()],
            on_error: |err| Box::pin(on_error(err)),
            owners: {
                let mut def = HashSet::new();
                def.insert(serenity::UserId(329357113480708106));
                if let Ok(c) = env::var("PRIVATE_ACCOUNT") {
                    if let Ok(id) = u64::from_str_radix(&c, 10) {
                        def.insert(serenity::UserId(id));
                    }
                }
                def
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|_ctx, _ready, _framework| Box::pin(async move { Ok(Data) }));

    framework.run().await.unwrap();
}

#[command(prefix_command)]
async fn ping(ctx: Context<'_>) -> Result {
    ctx.say("Pong!").await?;
    ctx.channel_id()
        .say(ctx.http(), format!("{:?}", ctx.framework().options.owners))
        .await?;
    Ok(())
}

async fn on_error(e: FrameworkError<'_, Data, Error>) {
    println!("error {:?}\n---", e);
}
