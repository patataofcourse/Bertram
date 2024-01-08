use bertram::crash::{saltwater::SWDType, ExcType, FAULT_STATUS_SOURCES};

use super::fetch_saltwater_dump;

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
