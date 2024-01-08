use std::{fs::File, io::Cursor};

use anyhow::anyhow;

use bertram::crash::{
    analyze::{self, CrashAnalysis, Symbols},
    saltwater::Region,
};

use crate::helpers::embed;

use super::{fetch_luma_dump, fetch_saltwater_dump};

/// Gets the name of a specific symbol in RHM for the specified region
#[poise::command(prefix_command, category = "For code modders")]
pub async fn symbol(
    ctx: crate::Context<'_>,
    #[description = "Location of the symbol (hexadecimal)."] sym: String,
    #[description = "Region to lookup (US/EU/JP/KR). Defaults to US."] region: Option<String>,
) -> crate::Result<()> {
    let region = match region
        .map(|c| c.to_lowercase())
        .as_ref()
        .map(String::as_ref)
    {
        Some("us") | None => Region::US,
        Some("eu") => Region::EU,
        Some("jp") => Region::JP,
        Some("kr") => Region::KR,
        _ => Err(anyhow!("invalid region"))?,
    };

    let mut symbols = Symbols::from_paths(
        format!("sym/rhm.{}.csv", format!("{region:?}").to_lowercase()),
        "",
    )?;

    symbols.init_bounds(region)?;

    let symbol = symbols.find_symbol(u32::from_str_radix(&sym, 16)?)?;

    ctx.say(match symbol {
        Some(c) => format!("Symbol found: {} ({:08x})", c.symbol, c.func_pos),
        None => "Symbol couldn't be found".to_string(),
    })
    .await?;

    Ok(())
}

#[poise::command(prefix_command, category = "For code modders")]
pub async fn analyze(
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
    let analysis = CrashAnalysis::from(&dump)?;
    embed(ctx, |e| analysis.as_serenity_embed(e)).await?;
    Ok(())
}

/// Generate Saltwater symbols for debug builds
#[poise::command(prefix_command, category = "Admin", owners_only)]
pub async fn symbolgen(
    ctx: crate::Context<'_>,
    #[description = "Link to the Saltwater 3GX file. If not provided, it expects the plugin to be sent as an attachment"]
    link: Option<String>,
) -> crate::Result<()> {
    let _3gx = if let crate::Context::Prefix(c) = ctx
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
    let hash = analyze::get_3gx_commit_hash(&mut Cursor::new(_3gx.as_slice()))?.unwrap();
    let mut out = File::create(format!("sym/sw._{hash}.csv",))?;

    Symbols::ctrplugin_symbols_to_csv(&mut Cursor::new(_3gx.as_slice()), &mut out, true)?;
    ctx.say(format!("Wrote symbols for commit {hash}!",))
        .await?;
    Ok(())
}
