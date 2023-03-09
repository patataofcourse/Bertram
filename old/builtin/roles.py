import discord
from discord.ext import commands

from constants import *

def is_helper():
    def predicate(ctx):
        return (ctx.message.author.id in ctx.bot.owner_ids or ctx.message.author.id in BOT_HELPERS)
    return commands.check(predicate)