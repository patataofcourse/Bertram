from discord.ext import commands
import requests
from struct import unpack_from

def setup(bot):
    bot.add_command(luma)

reg_names = [
    "r0", "r1", "r2", "r3", "r4", "r5", "r6",
    "r7", "r8", "r9", "r10", "r11", "r12",
    "sp", "lr", "pc","cpsr",
    "dfsr", "ifsr", "far",
    "fpexc", "fpinst", "fpinst2"
]
exception_types = ["FIQ", "undefined instruction", "prefetch abort", "data abort"]
svcBreak_reasons = ["(svcBreak: panic)", "(svcBreak: assertion failed)", "(svcBreak: user-related)"]
fault_sources = {
    0b0001:'Alignment',
    0b0010:'Debug event',
    0b0011:'Access bit - Section', 
    0b0100:'Instruction cache maintenance operation fault',
    0b0101:'Translation - Section',
    0b0110:'Access bit - Page',
    0b0111:'Translation - Page',
    0b1000:'Precise External Abort',
    0b1001:'Domain - Section',
    0b1010:'Imprecise External Abort',
    0b1011:'Domain - Page',
    0b1100:'External Abort on translation - First-level',
    0b1101:'Permission - Section',
    0b1110:'External Abort on translation - Second-level',
    0b1111:'Permission - Page',
}

def luma_ver(maj, min, mic):
    return (maj << 16) + (min << 8) + mic

def print_ver(ver):
    out = str(ver >> 16)
    out += "." + str((ver & 0xFFFF) >> 8)
    if ver & 0xFF != 0:
        out += "." + str((ver & 0xFF))
    return out

@commands.command(
    name = "luma",
    usage = "<crash dump as link or attachment>",
    description = "Parses a Luma3DS crash dump"
)
async def luma(ctx, link = None):
    if link == None:
        try:
            f = requests.get(ctx.message.attachments[0].url).content
        except IndexError:
            await ctx.send("Add a link to the dump, or attach it to your message!")
            return
        except:
            await ctx.send("Could not get file from link!")
            return
    else:
        try:
            f = requests.get(link).content
        except:
            await ctx.send("Could not get file from link!")
            return
    
    if unpack_from("<2I", f) != (0xdeadc0de, 0xdeadcafe):
        await ctx.send("Not a Luma3DS crash dump!")
    version, processor, exc_type, _, num_regs, code_size, stack_size, extra_size = unpack_from("<8I", f, 8)
    num_regs //= 4
    processor, core = processor & 0xffff, processor >> 16

    if version < luma_ver(1,0,2):
        await ctx.send(f"Unsupported crash dump (version {print_ver(version)}, minimum supported 1.0.2)")
    
    r = list(unpack_from("<{0}I".format(num_regs), f, 40)) # registers
    r.extend([None] * max(0, 23 - len(r)))
    sp, lr, pc, cpsr = r[13:17]
    dfsr, ifsr, far = r[17:20]
    fpexc, fpinst, fpinst2 = r[20:23]
    print(r)

    code_pos = 40 + 4 * num_regs
    code = f[code_pos : code_pos + code_size]
    stack_pos = code_pos + code_size
    stack = f[stack_pos : stack_pos + stack_size]
    extra_pos = stack_pos + stack_size
    extra = f[extra_pos : extra_pos + extra_size]

    out = "Luma3DS exception:\n```\n"
    if processor == 9:
        out += "Processor: arm9\n"
    else:
        out += f"Processor: arm11 (core {core})\n"
    out += "Exception type: "
    out += exception_types[exc_type]

    if exc_type == 2: # prefetch
        if cpsr & 0x20 and code_size > 4:
            instruction = unpack_from("<I", code[-4:])[0]
            match instruction:
                case 0xe12fff7e: # cdpvc p15, #0xf, c2, c15, c1, #7
                    out += " (kernel panic)"
                case 0xef00003c:
                    out += svcBreak_reasons[r[0]] if r[0] < 3 else " (svcBreak)"
    elif processor != 9 and (fpexc & 0x80000000):
        out += " (VFP exception)"

    out += "\n"

    if processor == 11 and exc_type >= 2: # data/prefetch abort
        out += "Fault status: "
        out += fault_sources[ifsr if exc_type == 2 else dfsr] + "\n"
    
    if extra_size != 0:
        if processor == 11:
                out += "Current process: {0} ({1:016x})".format(extra[:8].decode("ascii"), unpack_from("<Q", extra, 8)[0]) + "\n"
        else:
                out += "<Dump contains ARM9 memory>\n"
    
    # registers
    out += "\nRegister dump:\n\n"
    for i in range(0, num_regs - (num_regs % 2), 2):
        out += "{0:<15}{1:<20}{2:<15}{3:<20}\n".format(
            reg_names[i], "{0:08x}".format(r[i]),
            reg_names[i + 1], "{0:08x}".format(r[i + 1])
        )
        
        
    if num_regs % 2 == 1:
        out += "{0:<15}{1:<20}\n".format(reg_names[num_regs - 1], "{0:08x}".format(r[num_regs - 1]))

    out += "```"
    await ctx.send(out)
