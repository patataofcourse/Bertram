use bertram::faq::{QUESTIONS, RESOURCES_TAG};

use crate::helpers::embed;

/// Tag that leads to resources-and-guides
#[poise::command(prefix_command, category = "Tags")]
pub async fn docs(ctx: crate::Context<'_>) -> crate::Result<()> {
    embed(ctx, |c| {
        c.color(crate::TAG_COLOR)
            .title("idk what to make the title of this")
            .description(RESOURCES_TAG)
    })
    .await?;
    Ok(())
}

/// Displays FAQ tags. Use with no tag to see a list of tags.
#[poise::command(prefix_command, category = "Tags")]
pub async fn faq(
    ctx: crate::Context<'_>,
    #[description = "Name of the tag to use"] name: Option<String>,
) -> crate::Result<()> {
    match name {
        Some(c) => {
            let Some(question) = QUESTIONS.iter().find(|q| q.name == c) else {
                ctx.say(format!(
                    "FAQ tag `{c}` does not exist. Use {}faq for a list of tags",
                    ctx.prefix()
                ))
                .await?;
                return Ok(());
            };
            embed(ctx, |c| {
                c.color(crate::TAG_COLOR)
                    .title(question.question)
                    .description(question.answer)
            })
            .await?;
        }
        None => {
            embed(ctx, |c| {
                c.color(crate::TAG_COLOR)
                    .title("Available FAQ tags")
                    .description(
                        QUESTIONS
                            .iter()
                            .map(|q| format!("- `{}` - {}", q.name, q.question))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
            })
            .await?;
        }
    }
    Ok(())
}
