#![feature(let_chains, iterator_try_collect)]

use std::collections::HashSet;
use std::env;

use poise::{
    serenity_prelude::{self as serenity, UserId},
    Framework, FrameworkError, FrameworkOptions, PrefixFrameworkOptions,
};

pub mod commands;
pub mod error;
pub mod event;
pub mod helpers;

#[derive(Debug)]
pub struct Data {
    pub ops: Vec<UserId>,
    pub prefix_override: Option<String>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Command = poise::Command<Data, Error>;
type Context<'a> = poise::Context<'a, Data, Error>;
type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
type Result<T> = std::result::Result<T, Error>;

const BERTRAM_COLOR: i32 = 0xbbf89b;
const TAG_COLOR: i32 = 0x00a3ff;

pub static RUSTC_AT_BUILD: &str = env!("RUSTC_VER");
pub static COMMIT_AT_BUILD: &str = env!("GIT_HASH");

async fn prefix(ctx: PartialContext<'_>) -> Result<Option<String>> {
    if ctx.guild_id == Some(serenity::GuildId(277545487375007744)) {
        Ok(Some("-".to_string()))
    } else if let Some(c) = &ctx.data.prefix_override {
        Ok(Some(c.to_string()))
    } else {
        Ok(Some("!".to_string()))
    }
}

fn alpha_check_inner(channel: serenity::ChannelId, guild: Option<serenity::GuildId>) -> bool {
    [1088507265759314020, 856358616469864489, 1112147857596760124].contains(&channel.0)
        || [1012766391897698394].contains(&guild.map(|c| c.0).unwrap_or(0))
}

async fn alpha_check(ctx: Context<'_>) -> Result<bool> {
    Ok(alpha_check_inner(ctx.channel_id(), ctx.guild_id()))
}

async fn op_check(ctx: Context<'_>) -> Result<bool> {
    Ok(ctx.data().ops.contains(&ctx.author().id)
        || ctx.framework().options.owners.contains(&ctx.author().id))
}

#[tokio::main]
async fn main() {
    let prefix_override = std::env::var("BERTRAM_PREFIX").ok();
    let framework = Framework::builder()
        .options(FrameworkOptions {
            prefix_options: PrefixFrameworkOptions {
                mention_as_prefix: false,
                dynamic_prefix: Some(|ctx| Box::pin(prefix(ctx))),
                ..Default::default()
            },
            commands: vec![
                // misc / generic
                ping(),
                commands::help::help(),
                // admin
                commands::admin::kill(),
                commands::admin::recompile(),
                commands::admin::info(),
                commands::crash::symbolgen(),
                // crash helpers
                commands::crash::ctru(),
                commands::crash::symbol(),
                commands::crash::solve(),
                // crash - for coders
                commands::crash::luma(),
                commands::crash::saltwater(),
                commands::crash::analyze(),
                // tags / FAQs
                commands::tags::docs(),
                commands::tags::faq(),
            ],
            on_error: |err| Box::pin(on_error(err)),
            owners: {
                let mut def = HashSet::new();
                //TODO: autodetect owners (default::Default :P)
                def.insert(serenity::UserId(329357113480708106));
                if let Ok(c) = env::var("PRIVATE_ACCOUNT")
                    && let Ok(id) = c.parse()
                {
                    def.insert(serenity::UserId(id));
                }
                def
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event::event_handler(ctx, event, framework, data))
            },
            // remove this after SpiceRack alpha is over
            command_check: Some(|c| Box::pin(alpha_check(c))),
            reply_callback: Some(|_, reply| {
                reply.reply(true);
                reply.allowed_mentions(|c| c.empty_parse());
            }),
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    ops: vec![UserId(231520589511262209)],
                    prefix_override,
                })
            })
        });

    framework.run().await.unwrap();
}

#[poise::command(prefix_command)]
/// PONG.
async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.say("Pong!").await?;
    Ok(())
}

async fn on_error(err: FrameworkError<'_, Data, Error>) {
    error::on_error(err)
        .await
        .unwrap_or_else(|e| println!("Failed to send error diagnostic: {:?}\n\n", e))
}
