import discord
from discord.ext import commands
import traceback

from constants import *

def setup(_bot):
    _bot.on_command_error = bot_error
    global bot
    bot = _bot

async def bot_error(ctx, error):
    try:
        if type(error) == commands.errors.NotOwner:
            await ctx.send("This command can only be used by the owner!")
        # elif type(error) == commands.CommandNotFound:
        #     await ctx.send(f"Command {ctx.message.content.split()[0][len(bot.command_prefix):]} doesn't exist!")
        elif type(error) == commands.errors.MissingRequiredArgument:
            await ctx.send(f"The correct usage for that command is: `{ctx.bot.command_prefix}{ctx.command.name} {ctx.command.usage}`")
        elif type(error) == commands.errors.CheckFailure:
            #TODO: better checks
            await ctx.send("Oops! Some sort of check failed.")
        elif type(error) == commands.errors.CommandOnCooldown:
            await ctx.send(f"Command is on cooldown! Try again in {int(error.retry_after)+1} seconds!") #how to round up instead of down: add 1! :ventidab:
        else:
            await ctx.send(f"{BOT_ERROR_MESSAGE}```\nError type: {' '.join(str(type(error.original)).split(' ')[1:]).strip('>')}\nError code: {hex(ctx.message.id)}```")
            errfile = open("error/"+hex(ctx.message.id), "w")
            errfile.write("".join(traceback.format_exception(type(error.original), error.original, error.original.__traceback__)))
            errfile.close()
            try:
                await ctx.bot.get_channel(BOT_ERROR_CHANNEL).send(str(hex(ctx.message.id)) + "```" +
                    "".join(traceback.format_exception(type(error.original), error.original, error.original.__traceback__)) + "```")
            except Exception as e:
                errfile = open("error/"+hex(ctx.message.id), "rw")
                errfile.write("".join(traceback.format_exception(type(e), e, e.__traceback__)))
                errfile.close()
                await ctx.bot.get_channel(BOT_ERROR_CHANNEL).send(f"{str(hex(ctx.message.id))}\nCouldn't send, check the error logs")
    except Exception as e:
        errfile = open("error/"+hex(ctx.message.id), "rw")
        errfile.write("".join(traceback.format_exception(type(e), e, e.__traceback__)))
        errfile.close()
        print("".join(traceback.format_exception(type(e), e, e.__traceback__)))
        await ctx.send("Not only was there an error, there was also an error in sending the report.\nNotify this to Villa#9733 ASAP, or else.")