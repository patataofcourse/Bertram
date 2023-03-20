use poise::{serenity_prelude as serenity, FrameworkError};

pub async fn on_error(
    e: FrameworkError<'_, crate::Data, crate::Error>,
) -> std::result::Result<(), serenity::SerenityError> {
    match &e {
        FrameworkError::Setup {
            data_about_bot,
            error,
            ..
        } => {
            println!(
                "Error setting up bot {}#{}:\n{}",
                data_about_bot.user.name, data_about_bot.user.discriminator, error
            )
        }
        FrameworkError::ArgumentParse { error, ctx, .. } => {
            ctx.send(|f| {
                f.content(format!("**Wrong command usage:** {}\nUsage: TODO", error))
                    .reply(true)
            })
            .await?;
        }
        FrameworkError::UnknownCommand {
            msg_content,
            msg,
            ctx,
            prefix,
            ..
        } => {
            msg.reply(
                ctx,
                format!(
                    "**Command not found:** {}\n\
                        See {prefix}help for a list of available commands",
                    msg_content.split_once(' ').unwrap_or((msg_content, "")).0
                ),
            )
            .await?;
        }
        FrameworkError::NotAnOwner { ctx } => {
            ctx.send(|f| {
                f.reply(true)
                    .content("You don't have permission to run this command!")
            })
            .await?;
        }
        FrameworkError::Command { error, ctx } => {
            ctx.send(|f| {
                f.reply(true).content(format!(
                    "An error happened while trying to run the command: {}",
                    error
                ))
            })
            .await?;
        }
        e => println!("{}", e),
    }
    Ok(())
}
