#![feature(let_chains)]

use std::collections::HashSet;
use std::env;

use poise::{
    serenity_prelude as serenity, Framework, FrameworkError, FrameworkOptions,
    PrefixFrameworkOptions,
};

pub mod commands;
pub mod error;
pub mod event;

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
            commands: vec![
                // misc / generic
                ping(),
                help(),

                // admin
                commands::admin::kill(),
                commands::admin::recompile(),

                // crash
                commands::crash::luma(),
                commands::crash::saltwater(),
                commands::crash::ctru(),
            ],
            on_error: |err| Box::pin(on_error(err)),
            owners: {
                let mut def = HashSet::new();
                //TODO: autodetect owners (default::Default :P)
                def.insert(serenity::UserId(329357113480708106));
                if let Ok(c) = env::var("PRIVATE_ACCOUNT") && let Ok(id) = c.parse() {
                    def.insert(serenity::UserId(id));
                }
                def
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event::event_handler(ctx, event, framework, data))
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
    match error::on_error(e).await {
        Ok(_) => {}
        Err(e) => println!("Failed to send error diagnostic: {:?}\n\n", e),
    }
}

// help command copied from an example

/// Shows this menu
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<()> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Type !help command for more info on a command.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}
