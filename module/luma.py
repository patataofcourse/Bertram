from discord.ext import commands
import requests
from struct import unpack_from

def setup(bot):
    bot.add_command(luma)

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
    await ctx.send(hex(unpack_from("<2I", f)[0]))
    await ctx.send(hex(unpack_from("<2I", f)[1]))
    version, processor, exceptionType, _, nbRegisters, codeDumpSize, stackDumpSize, additionalDataSize = unpack_from("<8I", f, 8)
    await ctx.send(hex(version))
    await ctx.send("arm9" if processor == 9 else "arm11")
    await ctx.send(["FIQ", "undefined instruction", "prefetch abort", "data abort"][exceptionType])
    