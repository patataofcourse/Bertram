use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};

use bertram::crash::{luma::LumaProcessor, ExcType, FAULT_STATUS_SOURCES};

use super::fetch_luma_dump;

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
