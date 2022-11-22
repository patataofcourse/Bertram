import discord
from discord.ext import commands
import os
import platform

from builtin import roles
from constants import *

def setup(_bot):
    _bot.add_command(kill)
    _bot.add_command(errwipe)
    _bot.add_command(execcmd)
    _bot.add_command(aexec)
    global bot
    bot = _bot

@commands.command(name="aexec",
                usage="<code>",
                description="Runs some async code.\nOwner only")
@commands.is_owner()
async def aexec(ctx, *args):
    code = " ".join(ctx.message.content.split(" ")[1:]).strip("`")
    # Make an async function with the code and `exec` it
    exec(
        f'async def __ex(ctx): ' +
        ''.join(f'\n {l}' for l in code.split('\n'))
    )
    await locals()['__ex'](ctx)
    
@commands.command(name="exec",
                    usage="<code>",
                    description="Evals some code.\n Owner only.")
@commands.is_owner()
async def execcmd(ctx, code):
    await ctx.send(eval(code))

@roles.is_helper()
@commands.command(name="kill", usage="", description="Kills the bot.", aliases=("ded","die"))
async def kill(ctx):
    await ctx.send(BOT_KILL_MESSAGE)
    await ctx.bot.close()

@roles.is_helper()
@commands.command(name="errwipe", usage="", description="Deletes all /error files.", aliases=("ew",))
async def errwipe(ctx):
    if platform.system() not in ("Linux", "Windows", "Darwin"):
        await ctx.send("ERROR: Can't use the errwipe command in current platform")
    for file in os.listdir("error"):
        if file == "_":
            continue
        if platform.system() == "Windows":
            os.system(f"del error/{file}")
        else:
            os.system(f"rm error/{file}")
    await ctx.send(BOT_ERRWIPE_MESSAGE)