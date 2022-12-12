import discord
from discord.ext import commands

from constants import *

def setup(bot):
    bot.add_command(ctru_err)

@commands.command(
    name = "ctru",
    usage = "<error code>",
)
async def ctru_err(ctx, arg):
    if arg.startswith("0x"):
        err = int(arg[2:], 16)
    else:
        err = int(arg)
    
    description = err & 0x3FF
    module = (err >> 10) & 0xFF
    #unk = (err >> 18) & 0x7 # always 0?
    summary = (err >> 21) & 0x3F
    level = err >> 27

    embed = discord.Embed(
        title = f"Nintendo error code {arg}",
        description = "In module %s:\n\t- Summary: %s\n\t- Level: %s\n\t- Description: %s (%d)" % (
            get_module(module),
            get_summary(summary),
            get_level(level),
            get_description(description),
            description
        ),
        color = BOT_COLOR
    )
    await ctx.send(embed=embed)

def get_description(description):
    match description:
        case 0:
            return "Success" 
        case 2:
            return "Invalid memory permissions (kernel)" 
        case 4:
            return "Invalid ticket version (AM)" 
        case 5:
            return "Invalid string length. This error is returned when service name length is greater than 8 or zero. (srv)" 
        case 6:
            return "Access denied. This error is returned when you request a service that you don't have access to. (srv)" 
        case 7:
            return "String size does not match string contents. This error is returned when service name contains an unexpected null byte. (srv)" 
        case 8:
            return "Camera already in use/busy (qtm)." 
        case 10:
            return "Not enough memory (os)" 
        case 26:
            return "Session closed by remote (os)" 
        case 32:
            return "Empty CIA? (AM)" 
        case 37:
            return "Invalid NCCH? (AM)" 
        case 39:
            return "Invalid title version (AM)" 
        case 43:
            return "Database doesn't exist/failed to open (AM)" 
        case 44:
            return "Trying to uninstall system-app (AM)" 
        case 47:
            return "Invalid command header (OS)" 
        case 101:
            return "Archive not mounted/mount-point not found (fs)" 
        case 105:
            return "Request timed out (http)" 
        case 106:
            return "Invalid signature/CIA? (AM)" 
        case 120:
            return "Title/object not found? (fs)" 
        case 141:
            return "Gamecard not inserted? (fs)" 
        case 230:
            return "Invalid open-flags / permissions? (fs)" 
        case 271:
            return "Invalid configuration (mvd)." 
        case 391:
            return "NCCH hash-check failed? (fs)" 
        case 392:
            return "RSA/AES-MAC verification failed? (fs)" 
        case 393:
            return "Invalid database? (AM)" 
        case 395:
            return "RomFS/Savedata hash-check failed? (fs)" 
        case 630:
            return "Command not allowed / missing permissions? (fs)" 
        case 702:
            return "Invalid path? (fs)" 
        case 761:
            return "Incorrect read-size for ExeFS? (fs)" 
        case 1000:
            return "Invalid selection" 
        case 1001:
            return "Too large" 
        case 1002:
            return "Not authorized" 
        case 1003:
            return "Already done" 
        case 1004:
            return "Invalid size" 
        case 1005:
            return "Invalid enum value" 
        case 1006:
            return "Invalid combination" 
        case 1007:
            return "No data" 
        case 1008:
            return "Busy" 
        case 1009:
            return "Misaligned address" 
        case 1010:
            return "Misaligned size" 
        case 1011:
            return "Out of memory" 
        case 1012:
            return "Not implemented" 
        case 1013:
            return "Invalid address" 
        case 1014:
            return "Invalid pointer" 
        case 1015:
            return "Invalid handle" 
        case 1016:
            return "Not initialized" 
        case 1017:
            return "Already initialized" 
        case 1018:
            return "Not found" 
        case 1019:
            return "Cancel requested" 
        case 1020:
            return "Already exists" 
        case 1021:
            return "Out of range" 
        case 1022:
            return "Timeout" 
        case 1023:
            return "Invalid result value"
        case _:
            return "Unknown"

def get_level(level):
    match level:
        case 0:
            return "Success" 
        case 1:
            return "Info" 
        case 25:
            return "Status" 
        case 26:
            return "Temporary" 
        case 27:
            return "Permanent" 
        case 28:
            return "Usage" 
        case 29:
            return "Reinitialize" 
        case 30:
            return "Reset" 
        case 31:
            return "Fatal" 
        case c:
            return f"Unknown ({c})"

def get_summary(summary):
    match summary:
        case 0:
            return "Success" 
        case 1:
            return "Nothing happened" 
        case 2:
            return "Would block" 
        case 3:
            return "Out of resource" 
        case 4:
            return "Not found" 
        case 5:
            return "Invalid state" 
        case 6:
            return "Not supported" 
        case 7:
            return "Invalid argument" 
        case 8:
            return "Wrong argument" 
        case 9:
            return "Canceled" 
        case 10:
            return "Status changed" 
        case 11:
            return "Internal" 
        case 63:
            return "Invalid result value"
        case c:
            return f"Unknown ({c})"

def get_module(module):
    match module:
        case 0:
            return "Common" 
        case 1:
            return "Kernel" 
        case 2:
            return "Util" 
        case 3:
            return "File server" 
        case 4:
            return "Loader server" 
        case 5:
            return "TCB" 
        case 6:
            return "OS" 
        case 7:
            return "DBG" 
        case 8:
            return "DMNT" 
        case 9:
            return "PDN" 
        case 10:
            return "GSP" 
        case 11:
            return "I2C" 
        case 12:
            return "GPIO" 
        case 13:
            return "DD" 
        case 14:
            return "CODEC" 
        case 15:
            return "SPI" 
        case 16:
            return "PXI" 
        case 17:
            return "FS" 
        case 18:
            return "DI" 
        case 19:
            return "HID" 
        case 20:
            return "CAM" 
        case 21:
            return "PI" 
        case 22:
            return "PM" 
        case 23:
            return "PM_LOW" 
        case 24:
            return "FSI" 
        case 25:
            return "SRV" 
        case 26:
            return "NDM" 
        case 27:
            return "NWM" 
        case 28:
            return "SOC" 
        case 29:
            return "LDR" 
        case 30:
            return "ACC" 
        case 31:
            return "RomFS" 
        case 32:
            return "AM" 
        case 33:
            return "HIO" 
        case 34:
            return "Updater" 
        case 35:
            return "MIC" 
        case 36:
            return "FND" 
        case 37:
            return "MP" 
        case 38:
            return "MPWL" 
        case 39:
            return "AC" 
        case 40:
            return "HTTP" 
        case 41:
            return "DSP" 
        case 42:
            return "SND" 
        case 43:
            return "DLP" 
        case 44:
            return "HIO_LOW" 
        case 45:
            return "CSND" 
        case 46:
            return "SSL" 
        case 47:
            return "AM_LOW" 
        case 48:
            return "NEX" 
        case 49:
            return "Friends" 
        case 50:
            return "RDT" 
        case 51:
            return "Applet" 
        case 52:
            return "NIM" 
        case 53:
            return "PTM" 
        case 54:
            return "MIDI" 
        case 55:
            return "MC" 
        case 56:
            return "SWC" 
        case 57:
            return "FatFS" 
        case 58:
            return "NGC" 
        case 59:
            return "CARD" 
        case 60:
            return "CARDNOR" 
        case 61:
            return "SDMC" 
        case 62:
            return "BOSS" 
        case 63:
            return "DBM" 
        case 64:
            return "Config" 
        case 65:
            return "PS" 
        case 66:
            return "CEC" 
        case 67:
            return "IR" 
        case 68:
            return "UDS" 
        case 69:
            return "PL" 
        case 70:
            return "CUP" 
        case 71:
            return "Gyroscope" 
        case 72:
            return "MCU" 
        case 73:
            return "NS" 
        case 74:
            return "News" 
        case 75:
            return "RO" 
        case 76:
            return "GD" 
        case 77:
            return "Card SPI" 
        case 78:
            return "EC" 
        case 79:
            return "Web Browser" 
        case 80:
            return "Test" 
        case 81:
            return "ENC" 
        case 82:
            return "PIA" 
        case 83:
            return "ACT" 
        case 84:
            return "VCTL" 
        case 85:
            return "OLV" 
        case 86:
            return "NEIA" 
        case 87:
            return "NPNS" 
        case 90:
            return "AVD" 
        case 91:
            return "L2B" 
        case 92:
            return "MVD" 
        case 93:
            return "NFC" 
        case 94:
            return "UART" 
        case 95:
            return "SPM" 
        case 96:
            return "QTM" 
        case 97:
            return "NFP (amiibo)" 
        case 254:
            return "Application" 
        case 255:
            return "Invalid result value" 
        case c:
            return f"<Unknown ({c})>"