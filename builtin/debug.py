import discord
from discord.ext import commands

from constants import *

def setup(bot):
    bot.add_command(debug)
    global module_errors
    module_errors = bot.module_errors
    if module_errors == "": module_errors = "No errors! :D"
    
@commands.command(name="debug",
             description = "Sends debug menu to a specific private channel only the owner can access. :eyes:",
             usage = "")
@commands.is_owner()
async def debug(ctx):
    await ctx.send("Debug menu sent!")
    embed = discord.Embed(title=f"{BOT_NAME} debug", color=BOT_COLOR)
    embed.add_field(name="Module errors", value=module_errors)
    await ctx.bot.get_channel(BOT_ERROR_CHANNEL).send(embed=embed)