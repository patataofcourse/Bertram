import discord
from discord.ext import commands

from builtin import roles

def setup(bot):
    bot.add_command(a)
    bot.add_command(b)
    bot.add_command(c)

# Command usable by anyone
@commands.command(
    name = "cmd1",
    usage = "",
    description = "Does a thing",
    help = "",
    hidden = False,
    aliases = ("1", "one")
)
async def a(ctx):
    await ctx.send("Hello!")

# Command usable by helpers and admins
@roles.is_helper()
@commands.command(
    name = "cmd2",
    usage = "",
    description = "Does another thing",
    help = "",
    hidden = False,
    aliases = ("2", "two")
)
async def b(ctx):
    await ctx.send("Hello helper!")

# Command usable by admins
@commands.is_owner()
@commands.command(
    name = "cmd3",
    usage = "",
    description = "Does yet another thing",
    help = "",
    hidden = False,
    aliases = ("3", "three")
)
async def c(ctx):
    await ctx.send("Hello admin!")