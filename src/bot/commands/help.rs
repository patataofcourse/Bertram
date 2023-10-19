use crate::{helpers::embed, Command, Context, Result};

/// Show this help menu
#[poise::command(prefix_command, check = "crate::op_check")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command you want to get help for"]
    #[rest]
    command: Option<String>,
) -> Result<()> {
    match command {
        Some(c) => command_help(ctx, c).await,
        None => display_commands(ctx).await,
    }
}

async fn command_help(ctx: Context<'_>, cmdname: String) -> Result<()> {
    //TODO: subcommands
    match ctx
        .framework()
        .options
        .commands
        .iter()
        .find(|c| c.name == cmdname)
    {
        None => {
            ctx.say(format!("No command \"{cmdname}\" found")).await?;
        }
        Some(cmd) => {
            embed(ctx, |embed| {
                //TODO: subcommands
                embed
                    .title(format!("{}{}", ctx.prefix(), cmdname))
                    .description(cmd.description.as_deref().unwrap_or(""))
                    .footer(|c| c.text("Bertram help"))
                    .field(
                        "Usage",
                        format!("{}{} {}", ctx.prefix(), cmdname, command_usage(cmd, false)),
                        false,
                    );
                if !cmd.parameters.is_empty() {
                    embed.field(
                        "Arguments",
                        cmd.parameters
                            .iter()
                            .map(|c| {
                                format!(
                                    "- `{}`{} - {}",
                                    c.name,
                                    if c.required { " (required)" } else { "" },
                                    c.description.as_deref().unwrap_or("[Missing description]")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                        false,
                    );
                }
                embed
            })
            .await?;
        }
    }
    Ok(())
}

pub async fn display_commands(ctx: Context<'_>) -> Result<()> {
    let mut categories = vec![(None, vec![])];
    'outer: for cmd in &ctx.framework().options.commands {
        // If command checks aren't successful, don't display
        for check in &cmd.checks {
            if !check(ctx).await? {
                continue 'outer;
            }
        }
        if cmd.owners_only && !ctx.framework().options.owners.contains(&ctx.author().id) {
            continue;
        }

        match categories.iter_mut().find(|cat| cat.0 == cmd.category) {
            Some(c) => c.1.push(cmd),
            None => categories.push((cmd.category, vec![cmd])),
        }
    }
    embed(ctx, |embed| {
        embed
            .title("Bertram help")
            .description(format!(
                "Use {}help <command> for more info on a command",
                ctx.prefix()
            ))
            .fields(categories.iter().map(|category| {
                (
                    category.0.unwrap_or("No category"),
                    category
                        .1
                        .iter()
                        .map(|c| {
                            format!(
                                "\\- **{}{}** {:}",
                                ctx.prefix(),
                                c.name,
                                command_usage(c, true)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                    false,
                )
            }))
    })
    .await?;
    Ok(())
}

fn command_usage(cmd: &Command, as_supercommand: bool) -> String {
    if cmd.subcommands.is_empty() || !as_supercommand {
        cmd.parameters
            .iter()
            .map(|p| {
                if p.required {
                    format!("<{}>", p.name)
                } else {
                    format!("[{}]", p.name)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        "...".to_string()
    }
}
