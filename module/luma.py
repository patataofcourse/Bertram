import discord
from discord.ext import commands

from constants import *

import capstone
from capstone import CS_ARCH_ARM, CS_MODE_ARM
import csv
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

class Err(Exception):
    def __init__(self, err):
        self.err = err
    def __str__(self):
        return self.err
    def __repr__(self):
        return f"Err({repr(self.err)})"

async def fetch_dump(ctx, link = None):
    if link == None:
        try:
            return requests.get(ctx.message.attachments[0].url).content
        except IndexError or NameError:
            raise Err("Add a link to the dump, or attach it to your message!")
        except:
            raise Err("Could not get file from link!")
    else:
        try:
            return requests.get(link).content
        except:
            raise Err("Could not get file from link!")

class LumaDump:
    def __init__(self, f):
        if unpack_from("<2I", f) != (0xdeadc0de, 0xdeadcafe):
            raise Err("Not a Luma3DS crash dump!")
        
        self.version, processor, self.exc_type = unpack_from("<3I", f, 8)
        self.num_regs, code_size, stack_size, extra_size = unpack_from("<4I", f, 24)
        self.num_regs //= 4
        self.processor, self.core = processor & 0xffff, processor >> 16

        if self.version < luma_ver(1,0,2):
            raise Err(f"Unsupported crash dump (version {print_ver(version)}, minimum supported 1.0.2)")

        self.r = list(unpack_from("<{0}I".format(self.num_regs), f, 40)) # registers
        self.r.extend([None] * max(0, 23 - len(self.r)))
        self.sp, self.lr, self.pc, self.cpsr = self.r[13:17]
        self.dfsr, self.ifsr, self.far = self.r[17:20]
        self.fpexc, self.fpinst, self.fpinst2 = self.r[20:23]

        code_pos = 40 + 4 * self.num_regs
        self.code = f[code_pos : code_pos + code_size]
        stack_pos = code_pos + code_size
        self.stack = f[stack_pos : stack_pos + stack_size]
        extra_pos = stack_pos + stack_size
        self.extra = f[extra_pos : extra_pos + extra_size]

class Analysis:
    def __init__(self):
        self.oob_pc = False
        self.function_pos = 0
        self.function = None
        self.call_stack = []
    def render(self, dump):
        embed = discord.Embed(
            title = "Crash dump analysis",
            description = "@ {0:08x} -> {1:08x} (@ PC -> LR)".format(dump.pc, dump.lr),
            color=BOT_COLOR,
        )
        embed.add_field(
            name = "Symbol information",
            value = ("Located at {0} ({1:08x})".format(self.function, self.function_pos) if self.function != None else "") + \
                    ("- PC out of bounds\n" if self.oob_pc else ""),
            inline = False,
        )
        if self.call_stack != []:
            embed.add_field(
                name = "CTGP-7-style call stack",
                value = "\n".join(["- {0:08x} ({1})".format(i[0], i[1]) for i in self.call_stack]),
                inline = False,
            )
        return embed

@commands.group(
    name = "luma",
    usage = "<crash dump as link or attachment> / (stack/code/analyze/solve) ...",
    description = "Parses a Luma3DS crash dump",
    invoke_without_command = True
)
async def luma(ctx, link = None):
    if ctx.invoked_subcommand is None:
        try:
            f = await fetch_dump(ctx, link)
            dump = LumaDump(f)
        except Err as e:
            await ctx.send(e)
            return

        out = "**Luma3DS crash**:\n```\n"
        if dump.processor == 9:
            out += "Processor: arm9\n"
        else:
            out += f"Processor: arm11 (core {dump.core})\n"
        out += "Exception type: "
        out += exception_types[dump.exc_type]

        if dump.exc_type == 2: # prefetch
            if dump.cpsr & 0x20 and len(dump.code) > 4:
                instruction = unpack_from("<I", dump.code[-4:])[0]
                match instruction:
                    case 0xe12fff7e: # cdpvc p15, #0xf, c2, c15, c1, #7
                        out += " (kernel panic)"
                    case 0xef00003c:
                        out += svcBreak_reasons[dump.r[0]] if dump.r[0] < 3 else " (svcBreak)"
        elif dump.processor != 9 and (dump.fpexc & 0x80000000):
            out += " (VFP exception)"

        out += "\n"

        if dump.processor == 11 and dump.exc_type >= 2: # data/prefetch abort
            out += "Fault status: "
            out += fault_sources[(dump.ifsr if dump.exc_type == 2 else dump.dfsr) & 0xF] + "\n"
        
        if len(dump.extra) != 0:
            if dump.processor == 11:
                    out += "Current process: {0} ({1:016x})".format(dump.extra[:8].decode("ascii"), unpack_from("<Q", dump.extra, 8)[0]) + "\n"
            else:
                    out += "<Dump contains ARM9 memory>\n"
        
        # registers
        out += "\nRegister dump:\n\n"
        for i in range(0, dump.num_regs - (dump.num_regs % 2), 2):
            out += "{0:<15}{1:<20}{2:<15}{3:<20}\n".format(
                reg_names[i], "{0:08x}".format(dump.r[i]),
                reg_names[i + 1], "{0:08x}".format(dump.r[i + 1])
            )
        if dump.num_regs % 2 == 1:
            out += "{0:<15}{1:<20}\n".format(reg_names[dump.num_regs - 1], "{0:08x}".format(dump.r[dump.num_regs - 1]))

        out += "\n- !luma stack <dump> to get a stack dump\n"
        if len(dump.code) > 4:
            out += "- !luma code <dump> to obtain a code dump\n"
        out += "- !luma analyze <dump> to analyze the crash data\n"

        out += "```"
        await ctx.send(out)

@luma.command(
    name = "stack",
    usage = "<crash dump as link or attachment> [lines to display]",
    description = "Returns a debug-optimized stack dump for the given Luma crash dump",
    help = "Defaults to 16 lines"
)
async def stack(ctx, link = None, lines = "16"):
    try:
        lines = int(link)
        link = None
    except:
        lines = int(lines)
    
    try:
        f = await fetch_dump(ctx, link)
        dump = LumaDump(f)
    except Err as e:
        await ctx.send(e)
        return
    
    out = "```\nStack dump (sp = {0:08x}):\n".format(dump.sp)
    out += "(Endianness applied)\n\n"

    for i in range(0, min(len(dump.stack), lines*16), 16):
        if len(dump.stack) - i > 16:
            d = dump.stack[i:i+16]
        else:
            d = dump.stack[i:]
        for i in range(0, len(d), 4):
            out += "{0:08x} ".format(int.from_bytes(d[i:i+4], "little"))
        out = out.rstrip()
        out += "\n"

    out += "```"
    if len(out) <= 4000:
        await ctx.send(out)
    else:
        await ctx.send("Too big! Choose a smaller number of lines")


@luma.command(
    name = "code",
    usage = "<crash dump as link or attachment>",
    description = "Returns a code dump for the given Luma crash dump"
)
async def code(ctx, link = None):
    try:
        f = await fetch_dump(ctx, link)
        dump = LumaDump(f)
    except Err as e:
        await ctx.send(e)
        return

    # REMINDER: it's offset by 0x34 for some stupid reason
    
    out = "Code dump (pc = {0:08x}, far = {1:08x})\n\n".format(dump.pc, dump.far)
    mode = capstone.Cs(CS_ARCH_ARM, CS_MODE_ARM)
    for i in mode.disasm(dump.code, dump.pc - len(dump.code) + 0x34):
        out += "0x%x:\t%s\t%s\n" %(i.address, i.mnemonic, i.op_str)

    await ctx.send("```" + out + "```")

@luma.command(
    name = "analyze",
    usage =  "<crash dump as link or attachment> [call stack length]",
    description = "Analyzes the crash dump and gives some information",
    help = "Call stack length defaults to 6. Set to 0 to show full call stack"
)
async def analyze(ctx, link = None, cs_len = "6"):
    try:
        cs_len = int(link)
        link = None
    except:
        cs_len = int(cs_len)

    try:
        f = await fetch_dump(ctx, link)
        dump = LumaDump(f)
    except Err as e:
        await ctx.send(e)
        return
    
    attr = Analysis()

    bounds = csv.reader(open("sym/bounds.csv"))
    next(bounds)
    for line in bounds:
        if line[0] != "US": continue
        code = int(line[1], 16)
        code_end = int(line[2], 16)
    
    if dump.exc_type == 2 or dump.pc >= code_end or dump.pc < code:
        attr.oob_pc = True
    else:
        symbols = list(csv.reader(open("sym/rhm.us.csv")))[1:]
        last_pos = 0
        last_name = None
        for line in symbols:
            if int(line[1],16) > dump.pc: break
            last_name = (line[2] + "::" if len(line) > 2 and line[2] != "Global" else "") + line[0]
            last_pos = int(line[1], 16)
        attr.function_pos = last_pos
        attr.function = last_name

        i = 0
        while cs_len == 0 or len(attr.call_stack) < cs_len:
            if i >= len(dump.stack): break
            item = int.from_bytes(dump.stack[i:i+4], "little")
            if item >= code and item < code_end:
                last_name = None
                for line in symbols:
                    if int(line[1],16) > item: break
                    last_name = (line[2] + "::" if len(line) > 2 and line[2] != "Global" else "") + line[0]
                attr.call_stack.append((item, last_name))
            i += 4
    
    await ctx.send(embed = attr.render(dump))

@luma.command(
    name = "solve",
    usage = "<crash dump as link or attachment>",
    description = "Gives you some help on what the issue with your dump may be"
)
async def solve(ctx, link = None):
    try:
        f = await fetch_dump(ctx, link)
        dump = LumaDump(f)
    except Err as e:
        await ctx.send(e)
        return
    
    embed = discord.Embed(
        title = "Megamix Crash Solver tm",
        description = "(disclaimer, results may be slightly off)",
        color = BOT_COLOR
    )

    if dump.pc == 0x0011e764 and dump.exc_type == 3:
        embed.add_field (
            name = "Did `call` or `async_call` with a sub number",
            value = "__100%__ chance\nSeems like you should get this fixed, and quick!",
            inline = False
        )
    
    if dump.pc == 0x001392c4 and dump.exc_type == 3:
        embed.add_field (
            name = "Ran out of effect file memory (using Karate Man's effect file)",
            value = "__~80%__ chance\nUse Bunny Hop's effect file instead!",
            inline = False
        )

    await ctx.send(embed = embed)