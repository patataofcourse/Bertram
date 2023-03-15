#![feature(let_chains)]

use std::collections::HashSet;
use std::env;

use poise::FrameworkContext;
use poise::{
    event::Event, serenity_prelude as serenity, Framework, FrameworkError, FrameworkOptions,
    PrefixFrameworkOptions,
};

pub mod commands;

#[derive(Debug)]
pub struct Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() {
    let framework = Framework::builder()
        .options(FrameworkOptions {
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".into()),
                mention_as_prefix: true,
                ..Default::default()
            },
            commands: vec![ping(), commands::admin::kill(), commands::crash::luma(), help()],
            on_error: |err| Box::pin(on_error(err)),
            owners: {
                let mut def = HashSet::new();
                //TODO: autodetect owners (default::Default :P)
                def.insert(serenity::UserId(329357113480708106));
                if let Ok(c) = env::var("PRIVATE_ACCOUNT") && let Ok(id) = u64::from_str_radix(&c, 10) {
                    def.insert(serenity::UserId(id));
                }
                def
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
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

#[poise::command(prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.say("Pong!").await?;
    Ok(())
}

async fn on_error(e: FrameworkError<'_, Data, Error>) {
    println!("error {:?}\n---", e);
}

async fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a Event<'a>,
    _framework: FrameworkContext<'a, Data, Error>,
    _data: &'a Data,
) -> Result<()> {
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

// help command copied from an example

/// Shows this menu
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<()> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type !help command for more info on a command.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}