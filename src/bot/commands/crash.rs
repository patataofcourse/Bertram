use std::io::Cursor;

use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};
use poise::Context;

use bertram::{
    crash::{
        luma::{CrashLuma, LumaProcessor},
        saltwater::CrashSWD,
    },
    ctru::CtruError,
};

pub async fn fetch_luma_dump(
    ctx: &crate::Context<'_>,
    link: Option<&str>,
) -> crate::Result<CrashLuma> {
    let file = if let Context::Prefix(c) = ctx && c.msg.attachments.len() > 0{
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
    let file = if let Context::Prefix(c) = ctx && c.msg.attachments.len() > 0{
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
pub async fn ctru(ctx: crate::Context<'_>, code: String) -> crate::Result<()> {
    let Ok(code) = u32::from_str_radix(code.trim_start_matches("0x"), 16) else {Err("Not a valid hex number")?};
    ctx.send(|f| {
        f.content(format!("```{}```", CtruError::from_code(code)))
            .reply(true)
    })
    .await?;
    Ok(())
}

/// Gives a report on a Luma3DS crash dump (.dmp)
#[poise::command(prefix_command, subcommands("stack"), category = "For code modders")]
pub async fn luma(ctx: crate::Context<'_>, link: Option<String>) -> crate::Result<()> {
    let dump = fetch_luma_dump(&ctx, link.as_deref()).await?;

    ctx.send(|f| {
        f.content(format!(
            concat!(
                "**Luma3DS crash dump:**\n",
                "```Processor: {}\n",
                "Exception type: {}\n",
                "Fault status: ...\n", //TODO
                "{}\n",
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
            if dump.extra.len() > 0 {
                if let LumaProcessor::Arm11(_) = dump.processor {
                    let info = dump.get_title_info().unwrap();
                    format!("Current process: {} ({:016X})", info.0, info.1)
                } else {
                    "<ARM9 memory embedded in the crash>".to_string()
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
        .reply(true)
    })
    .await?;
    Ok(())
}

/// Shows the stack of a Luma3DS crash dump (.dmp)
#[poise::command(prefix_command)]
pub async fn stack(ctx: crate::Context<'_>, link: Option<String>) -> crate::Result<()> {
    let dump = fetch_luma_dump(&ctx, link.as_deref()).await?;

    let mut formatted_stack = String::new();

    let mut stack_slice: &[u8] = &dump.stack;
    for i in 0..(dump.stack.len() / 4) {
        formatted_stack += &format!(
            "{:08x}{}",
            u32::read_from(&mut stack_slice, LE).unwrap(),
            if i % 4 == 3 { "\n" } else { " " }
        );
        //TODO: customizable stack size for printing
        if (formatted_stack.clone() + ".").lines().count() >= 16 {
            break;
        }
    }
    formatted_stack = formatted_stack.trim_end().to_string();

    ctx.send(|f| {
        f.content(format!(
            "Stack dump (w/endian) (sp = `{:08x}`):```{}```",
            dump.registers[13], &formatted_stack
        ))
        .reply(true)
    })
    .await
    .unwrap();
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn saltwater(ctx: crate::Context<'_>, link: Option<String>) -> crate::Result<()> {
    let dump = fetch_saltwater_dump(&ctx, link.as_deref()).await?;
    ctx.say(format!("{:?}", dump)).await?;
    Ok(())
}
