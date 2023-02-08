import discord
from discord.ext import commands

from constants import *

def setup(bot):
    bot.add_command(asset)

@commands.command(
    name = "asset",
    usage = "<game name>",
    description = "Gives you information on a specific game's assets"
)
async def asset(ctx, *, name):
    await ctx.send(name)