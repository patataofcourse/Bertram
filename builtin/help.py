#Help command

import discord
from discord.ext import commands

from builtin import roles
from constants import *

def setup(bot):
    bot.add_command(hlp)

def group_usage(c): #makes usage if it's a group and usage = ""
    if type(c) == commands.Group:
        u = "("
        for cmd in c.commands:
            u += cmd.name + "/"
        u = u.rstrip("/") + ")"
        return u
    else:
        return ""

@commands.command(name="help", description="The help command. You are using it.", usage="<command>")
async def hlp(ctx, *args):
    if args == ():
        # General help menu
        
        standard = "" # standard command text goes in here
        helper = ""   # helper command text goes here
        owner = ""    # owner command text goes in here
        
        for command in ctx.bot.commands:
            if command.hidden:
                continue

            owner_cmd = False
            helper_cmd = False
            for check in command.checks:
                if "is_owner" in str(check):
                    owner_cmd = True
                if "is_helper" in str(check):
                    helper_cmd = True

            if command.usage == None or command.usage == "":
                command.usage = group_usage(command)
            if owner_cmd:
                owner += f"**{ctx.bot.command_prefix}" + command.name + "** " + command.usage + "\n"
            elif helper_cmd:
                helper += f"**{ctx.bot.command_prefix}" + command.name + "** " + command.usage + "\n"
            else:
                standard += f"**{ctx.bot.command_prefix}" + command.name + "** " + command.usage + "\n"
    
        is_owner = ctx.author.id in BOT_OWNERS
        is_helper = is_owner or ctx.author.id in BOT_HELPERS

        # Display the actual menu / embed!
        embed = discord.Embed(title={BOT_HELP_TITLE}, description=f"Use {ctx.bot.command_prefix}help <command> for more info on the command", color=BOT_COLOR)
        embed.add_field(name="Standard commands:", value = standard if len(standard) != 0 else "None", inline = False)
        if is_helper and helper != "":
            embed.add_field(name="Helper commands:", value = helper, inline = False)
        if is_owner and owner != "":
            embed.add_field(name="Owner commands:", value = owner, inline = False)
        await ctx.send(embed=embed)

    else:
        # Display specific help for the command - if it exists

        cmd = args[0]
        for command in ctx.bot.commands:
            if cmd == command.name:

                # If it's a command group, find the subcommand
                if type(command) == commands.Group and len(args) != 1:
                    subc = command.get_command(" ".join(args[1:]))
                    await ctx.send(subc)
                    if subc != None:
                        command = subc

                # Manage default help/usage/aliases parameters
                if command.help == None or command.help == "":
                    if type(command) == commands.Group:
                        command.help = f"Check **{ctx.bot.command_prefix}help {command.name} <subcommand>** for more info!"
                    else:
                        command.help = ""
                if command.usage == None or command.usage == "":
                    command.usage = group_usage(command)
                if command.aliases == None:
                    command.aliases = []
                
                embed = discord.Embed(title=f"{ctx.bot.command_prefix}{command.qualified_name}", description=command.description, color=BOT_COLOR)
                embed.add_field(name="Usage:", value=f"**{ctx.bot.command_prefix}{command.qualified_name} ** {command.usage}\n\n{command.help}", inline=False)
                if len(command.aliases) != 0:
                    embed.add_field(name="Aliases:", value=ctx.bot.command_prefix+('\n'+ctx.bot.command_prefix).join(command.aliases), inline=False)
                await ctx.channel.send(embed=embed)
                return
        # no command found
        await ctx.send(f"Command `{args[0]}` doesn't exist!")