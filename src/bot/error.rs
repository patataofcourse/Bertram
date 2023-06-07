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
            ctx.say(format!("**Wrong command usage:** {}\nUsage: TODO", error))
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
            ctx.say(format!(
                "An error happened while trying to run the command:```\n{}```",
                error
            ))
            .await?;
        }
        FrameworkError::CommandCheckFailed {
            error: None,
            ctx: _,
        } => {}
        e => {
            let Some(ctx) = e.ctx() else {
            println!("{}", e); return Ok(())
        };
            ctx.say(format!("Error happened:```\n{}```", e)).await?;
        }
    }
    Ok(())
}
