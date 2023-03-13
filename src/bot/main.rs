use std::collections::HashSet;
use std::env;

use serenity::prelude::*;

use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::Message,
        prelude::{Activity, UserId},
        user::OnlineStatus,
    },
};

pub mod commands;

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: serenity::model::prelude::Ready) {
        ctx.set_presence(
            Some(Activity::playing(
                "Rhythm Heaven Megamix tm for the Nintendo 3DS tm",
            )),
            OnlineStatus::Online,
        )
        .await;
        println!("Bot ready!")
    }

    async fn resume(&self, ctx: Context, _: serenity::model::prelude::ResumedEvent) {
        ctx.set_presence(
            Some(Activity::playing(
                "Rhythm Heaven Megamix tm for the Nintendo 3DS tm",
            )),
            OnlineStatus::Online,
        )
        .await;
        println!("Bot back up!")
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| {
            let mut owners = HashSet::new();
            owners.insert(UserId(329357113480708106));
            if let Ok(c) = env::var("PRIVATE_ACCOUNT") {
                if let Ok(id) = u64::from_str_radix(&c, 10) {
                    owners.insert(UserId(id));
                }
            }
            c.prefix("!").owners(owners)
        })
        .group(&GENERAL_GROUP)
        .group(&commands::admin::ADMIN_GROUP)
        .help(&MY_HELP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
#[description("helo")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[help]
#[embed_success_colour("#BBF89B")]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await?;
    Ok(())
}
