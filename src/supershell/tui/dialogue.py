import time

from supershell.tui import effects
from supershell.tui.console import get_console

CHARACTER_PROPERTIES = {
    "system": {"name": "System", "style": "bold white", "delay": 0},
    "quest": {"name": "[bold]Quest Log[/bold]", "style": "bold white", "delay": 0},
    "cypher": {"name": "Cypher", "style": "yellow", "delay": 0.04},
    "glitch": {"name": "Glitch", "style": "bold bright_black", "delay": 0.02},
    "hunter": {"name": "The Hunter", "style": "bold red", "delay": 0},
}


def say(message: str, character: str = "cypher"):
    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])

    effects.typewriter_print(
        message=message,
        prefix=f"{props['name']}:",
        prefix_style=props["style"],
        message_style="stdout",
        char_delay=props["delay"],
    )


def say_speech(speech: list[str], character: str = "cypher"):
    for line in speech:
        say(line, character)
        time.sleep(0.3)


def ask(prompt: str, character: str = "cypher") -> str:
    console = get_console()

    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])
    style = props["style"]
    name = props["name"]

    rich_prompt = f"  [{style}]{name}: {prompt}[/{style}] [prompt]> [/prompt]"
    return console.input(rich_prompt)
