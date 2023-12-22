import asyncio
import os
import discord
import discord.ext.commands
import discord.ext.tasks
import dotenv

dotenv.load_dotenv()
DISCORD_TOKEN = os.getenv('DISCORD_TOKEN')
CHANNEL_ID = int(os.getenv('CHANNEL_ID'))

bot = discord.ext.commands.Bot(command_prefix='fuzz ', intents=discord.Intents.all())


@bot.command()
async def ping(ctx):
    await ctx.send('pong')


@discord.ext.tasks.loop(hours=24 * 14)
async def cron():
    channel = bot.get_channel(CHANNEL_ID)

    fuzzing = await asyncio.create_subprocess_shell('bash run.sh develop')
    await fuzzing.wait()
    message = await asyncio.create_subprocess_shell(
        'cd deps/ckb-vm && git log -1 --pretty=\'%s\'',
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.STDOUT
    )
    message = await message.communicate()
    message = message[0].decode()
    if fuzzing.returncode == 0:
        await channel.send('Develop branch passed' + '\n* ' + message)
    else:
        await channel.send('Develop branch failed' + '\n* ' + message)

    fuzzing = await asyncio.create_subprocess_shell('bash run.sh release-0.24')
    await fuzzing.wait()
    message = await asyncio.create_subprocess_shell(
        'cd deps/ckb-vm && git log -1 --pretty=\'%s\'',
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.STDOUT
    )
    message = await message.communicate()
    message = message[0].decode()
    if fuzzing.returncode == 0:
        await channel.send('Release branch passed' + '\n* ' + message)
    else:
        await channel.send('Release branch failed' + '\n* ' + message)


@bot.listen()
async def on_ready():
    cron.start()

bot.run(DISCORD_TOKEN)
