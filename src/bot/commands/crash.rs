use std::{fs::File, io::Cursor};

use anyhow::anyhow;
use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};
use poise::Context;

use bertram::{
    crash::{
        analyze::{self, CrashAnalysis, Symbols},
        luma::{CrashLuma, LumaProcessor},
        saltwater::{CrashSWD, Region, SWDType},
        solve::SolveDiagnosis,
        ExcType, FAULT_STATUS_SOURCES,
    },
    ctru::CtruError,
};

use crate::helpers::embed;

pub async fn fetch_luma_dump(
    ctx: &crate::Context<'_>,
    link: Option<&str>,
) -> crate::Result<CrashLuma> {
    let file = if let Context::Prefix(c) = ctx && !c.msg.attachments.is_empty() {
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

pub async fn fetch_saltwater_dump(
    ctx: &crate::Context<'_>,
    link: Option<&str>,
) -> crate::Result<CrashSWD> {
    let file = if let Context::Prefix(c) = ctx && !c.msg.attachments.is_empty() {
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

/// Gives a report on a Luma3DS crash dump (.dmp)
#[poise::command(prefix_command, subcommands("stack"), category = "For code modders")]
pub async fn luma(
    ctx: crate::Context<'_>,
    #[description = "Link to the crash dump. If not provided, it expects the dump to be sent as an attachment"]
    link: Option<String>,
) -> crate::Result<()> {
    let dump = fetch_luma_dump(&ctx, link.as_deref()).await?;

    //TODO: move formatting to main crate
    ctx.say(format!(
        concat!(
            "**Luma3DS crash dump:**\n",
            "```Processor: {}\n",
            "Exception type: {}\n",
            "{}",
            "{}",
            "\n",
            "Register dump:\n",
            "r0      {:08x}    r1      {:08x}\n",
            "r2      {:08x}    r3      {:08x}\n",
            "r4      {:08x}    r5      {:08x}\n",
            "r6      {:08x}    r7      {:08x}\n",
            "r8      {:08x}    r9      {:08x}\n",
            "r10     {:08x}    r11     {:08x}\n",
            "r12     {:08x}    sp      {:08x}\n",
            "lr      {:08x}    pc      {:08x}\n",
            "{}    {}\n",
            "{}    {}\n",
            "{}    {}\n",
            "{}\n",
            "```"
        ),
        dump.processor,
        dump.exception_type,
        if let ExcType::DataAbort | ExcType::PrefetchAbort = dump.exception_type {
            let fault = if dump.exception_type == ExcType::DataAbort {
                let dfsr = dump.registers.get(17).unwrap_or(&0);
                (dfsr & 0xf) + ((dfsr >> 10) & 1)
            } else {
                let ifsr = dump.registers.get(18).unwrap_or(&0);
                ifsr & 0xf
            };
            format!(
                "Fault status: {}\n",
                FAULT_STATUS_SOURCES
                    .iter()
                    .find(|(k, _)| *k == fault)
                    .unwrap_or(&(0, "Invalid"))
                    .1
            )
        } else {
            String::new()
        },
        if !dump.extra.is_empty() {
            if let LumaProcessor::Arm11(_) = dump.processor {
                let info = dump.get_title_info().unwrap();
                format!("Current process: {} ({:016X})\n", info.0, info.1)
            } else {
                "<ARM9 memory embedded in the crash>\n".to_string()
            }
        } else {
            String::new()
        },
        //TODO: for loop instead???
        dump.registers[0],
        dump.registers[1],
        dump.registers[2],
        dump.registers[3],
        dump.registers[4],
        dump.registers[5],
        dump.registers[6],
        dump.registers[7],
        dump.registers[8],
        dump.registers[9],
        dump.registers[10],
        dump.registers[11],
        dump.registers[12],
        dump.registers[13],
        dump.registers[14],
        dump.registers[15],
        if let Some(c) = dump.registers.get(16) {
            format!("cpsr    {c:08x}")
        } else {
            String::new()
        },
        if let Some(c) = dump.registers.get(17) {
            format!("dfsr    {c:08x}")
        } else {
            String::new()
        },
        if let Some(c) = dump.registers.get(18) {
            format!("ifsr    {c:08x}")
        } else {
            String::new()
        },
        if let Some(c) = dump.registers.get(19) {
            format!("far     {c:08x}")
        } else {
            String::new()
        },
        if let Some(c) = dump.registers.get(20) {
            format!("fpexc   {c:08x}")
        } else {
            String::new()
        },
        if let Some(c) = dump.registers.get(21) {
            format!("fpinst  {c:08x}")
        } else {
            String::new()
        },
        if let Some(c) = dump.registers.get(22) {
            format!("fpinst2 {c:08x}")
        } else {
            String::new()
        },
    ))
    .await?;
    Ok(())
}

/// Shows the stack of a Luma3DS crash dump (.dmp)
#[poise::command(prefix_command)]
pub async fn stack(
    ctx: crate::Context<'_>,
    #[description = "Link to the crash dump. If not provided, it expects the dump to be sent as an attachment"]
    link: Option<String>,
    #[description = "Amount of lines to display. Currently requires `link` due to limitations."]
    size: Option<usize>,
) -> crate::Result<()> {
    //TODO: maybe make it possible for size to be given on its own? how though?
    let dump = fetch_luma_dump(&ctx, link.as_deref()).await?;

    let mut formatted_stack = String::new();

    let mut stack_slice: &[u8] = &dump.stack;
    for i in 0..(dump.stack.len() / 4) {
        formatted_stack += &format!(
            "{:08x}{}",
            u32::read_from(&mut stack_slice, LE).unwrap(),
            if i % 4 == 3 { "\n" } else { " " }
        );
        if (formatted_stack.clone() + ".").lines().count() > size.unwrap_or(16) {
            break;
        }
    }
    formatted_stack = formatted_stack.trim_end().to_string();

    ctx.say(format!(
        "Stack dump (w/endian) (sp = `{:08x}`):```{}```",
        dump.registers[13], &formatted_stack
    ))
    .await
    .unwrap();
    Ok(())
}

/// Gives a report on a Saltwater crash dump (.swd)
#[poise::command(prefix_command, category = "For code modders")]
pub async fn saltwater(
    ctx: crate::Context<'_>,
    #[description = "Link to the crash dump. If not provided, it expects the dump to be sent as an attachment"]
    link: Option<String>,
) -> crate::Result<()> {
    let dump = fetch_saltwater_dump(&ctx, link.as_deref()).await?;

    //TODO: move formatting to main crate
    ctx.say(format!(
        concat!(
            "**Saltwater crash dump:**\n",
            "```Region: {}\n",
            "Version: {}\n",
            "Exception type: {}\n",
            "{}",
            "\nRegister dump:\n",
            "{}",
            "lr      {:08x}    pc      {:08x}\n{}",
            "\nCall stack (wip):\n",
            // TODO: fetch symbols
            // TODO: make this dependent on CALL_STACK_SIZE if possible
            " - {:08x}\n",
            " - {:08x}\n",
            " - {:08x}\n",
            " - {:08x}\n",
            " - {:08x}\n",
            "```"
        ),
        dump.region,
        dump.version,
        dump.exception_type,
        if let ExcType::DataAbort | ExcType::PrefetchAbort = dump.exception_type {
            let fault = if dump.exception_type == ExcType::DataAbort {
                let dfsr = dump.status_a;
                (dfsr & 0xf) + ((dfsr >> 10) & 1)
            } else {
                let ifsr = dump.status_b;
                ifsr & 0xf
            };
            format!(
                "Fault status: {}\n",
                FAULT_STATUS_SOURCES
                    .iter()
                    .find(|(k, _)| *k == fault)
                    .unwrap_or(&(0, "Invalid"))
                    .1
            )
        } else {
            String::new()
        },
        if dump.crash_type == SWDType::Extended {
            let regs = dump.registers.unwrap();
            format!(
                concat!(
                    "r0      {:08x}    r1      {:08x}\n",
                    "r2      {:08x}    r3      {:08x}\n",
                    "r4      {:08x}    r5      {:08x}\n",
                    "r6      {:08x}    r7      {:08x}\n",
                    "r8      {:08x}    r9      {:08x}\n",
                    "r10     {:08x}    r11     {:08x}\n",
                    "r12     {:08x}    sp      {:08x}\n",
                ),
                //TODO: again, this might be better with a for loop?
                regs[0],
                regs[1],
                regs[2],
                regs[3],
                regs[4],
                regs[5],
                regs[6],
                regs[7],
                regs[8],
                regs[9],
                regs[10],
                regs[11],
                regs[12],
                regs[13],
            )
        } else {
            "".to_string()
        },
        dump.lr,
        dump.pc,
        match dump.exception_type.status_reg_names() {
            [None, _] => "".to_string(),
            [Some(c), None] => format!("{:08}{:08x}", c, dump.status_a),
            [Some(c), Some(d)] => format!(
                "{:08}{:08x}    {:08}{:08x}",
                c, dump.status_a, d, dump.status_b
            ),
        },
        dump.call_stack[0],
        dump.call_stack[1],
        dump.call_stack[2],
        dump.call_stack[3],
        dump.call_stack[4],
    ))
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
    let _3gx = if let Context::Prefix(c) = ctx && !c.msg.attachments.is_empty() {
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
