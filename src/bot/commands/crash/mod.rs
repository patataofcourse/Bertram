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
#[poise::command(prefix_command, category = "Helpers", check = "crate::op_check")]
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
    let diagnoses = SolveDiagnosis::find_matches(&dump);
    ctx.say(format!("{diagnoses:?}")).await?;
    Ok(())
}
