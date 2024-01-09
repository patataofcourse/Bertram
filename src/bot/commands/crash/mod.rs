pub mod analyze;
pub mod luma;
pub mod saltwater;

pub use analyze::{analyze, symbol, symbolgen};
pub use luma::{luma, stack};
pub use saltwater::saltwater;

use std::io::Cursor;

use poise::Context;

use bertram::{
    crash::{luma::CrashLuma, saltwater::CrashSWD, solve::SolveDiagnosis},
    ctru::CtruError,
};

use crate::helpers::embed;

async fn fetch_luma_dump(ctx: &crate::Context<'_>, link: Option<&str>) -> crate::Result<CrashLuma> {
    let file = if let crate::Context::Prefix(c) = ctx
        && !c.msg.attachments.is_empty()
    {
        c.msg.attachments[0].download().await?
    } else {
        reqwest::get(link.ok_or("No file given")?)
            .await?
            .bytes()
            .await?
            .into()
    };

    Ok(CrashLuma::from_file(&mut Cursor::new(file.as_slice()))?)
}

async fn fetch_saltwater_dump(
    ctx: &crate::Context<'_>,
    link: Option<&str>,
) -> crate::Result<CrashSWD> {
    let file = if let Context::Prefix(c) = ctx
        && !c.msg.attachments.is_empty()
    {
        c.msg.attachments[0].download().await?
    } else {
        reqwest::get(link.ok_or("No file given")?)
            .await?
            .bytes()
            .await?
            .into()
    };

    Ok(CrashSWD::from_file(&mut Cursor::new(file.as_slice()))?)
}

/// Analyzes an ErrDisp / ctru error code
#[poise::command(prefix_command, category = "Helpers")]
pub async fn ctru(
    ctx: crate::Context<'_>,
    #[description = "Error code to interpret"] code: String,
) -> crate::Result<()> {
    let Ok(code) = u32::from_str_radix(code.trim_start_matches("0x"), 16) else {
        Err("Not a valid hex number")?
    };
    ctx.say(format!("```{}```", CtruError::from_code(code)))
        .await?;
    Ok(())
}

/// Gives possible errors that could cause a crash
#[poise::command(prefix_command, category = "Helpers")]
pub async fn solve(
    ctx: crate::Context<'_>,
    #[description = "Link to the crash dump. If not provided, it expects the dump to be sent as an attachment"]
    link: Option<String>,
) -> crate::Result<()> {
    let dump = match fetch_luma_dump(&ctx, link.as_deref()).await {
        Ok(c) => c.as_generic(Some(5))?,
        Err(_) => fetch_saltwater_dump(&ctx, link.as_deref())
            .await?
            .as_generic(),
    };
    let diagnoses = SolveDiagnosis::find_matches(&dump)?;
    let mut output = diagnoses
        .iter()
        .map(|c| match c {
            SolveDiagnosis::InvalidTickflowAddress(addr) => (
                format!(
                    "Tried to run Tickflow at an invalid address{}",
                    addr.map(|c| format!(" (`{c:08x}`)")).unwrap_or_default()
                ),
                "__100% chance__\n\
                Things that might cause this:\n\
                - `call`ing a sub by number\n\
                - `sub`ing a sub by label\n\
                - `return`ing on an async thread\n\
                - Weird Tickompiler behavior (goto loc, or loc shares name with a string)"
                    .to_string(),
            ),
            SolveDiagnosis::NoEffectMemory => (
                "Ran out of effect file memory (using Karate Man's effect file?)".to_string(),
                "__~80%__ chance\n\
                Use Bunny Hop's effect file instead!"
                    .to_string(),
            ),
            //TODO
            SolveDiagnosis::SceneLoadingError(_err) => (
                "Error in the scene loading process".to_string(),
                "Due to call stack size, this diagnosis is unlikely to appear anymore.\n\
                __~90%__ chance\n\
                Might be one of the following:\n\
                - No cellanim/effect/layout loaded\n\
                - Layout loaded in a slot lesser or equal than 3\n\
                If this diagnosis gets fixed, more data will later be shown"
                    .to_string(),
            ),
            SolveDiagnosis::NonExecRegion(addr) => (
                format!("Running code in a non-executable region (`{addr:08x}`)",),
                "__100%__ chance\n\
                Something went really wrong with a code patch!"
                    .to_string(),
            ),
            SolveDiagnosis::NullRead => (
                "Tried to read from null".to_string(),
                "__100% chance__\n\
                There's many reasons why this error could be happening, too many to list.\n\
                However, the most relevant are:\n\
                - Cellanim/effect/layout not loaded\n\
                - Layout loaded in a slot lesser or equal than 3\n\
                - Other scene loading mishaps\n\
                - A misbehaving code patch\n\
                - Ran out of memory"
                    .to_string(),
            ),
        })
        .collect::<Vec<_>>();

    if output.is_empty() {
        output.push((
            "No matches found!".to_string(),
            "Something really wack is going on - reverse-engineering might be needed.".to_string(),
        ))
    }
    embed(ctx, |e| {
        let mut e = e
            .title("Bertram solver")
            .description("Here's some possible diagnoses for your error!");
        for field in output {
            e = e.field(field.0, field.1, false)
        }
        e
    })
    .await?;
    Ok(())
}
