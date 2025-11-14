"""
Handles all TUI output for in-game characters.
"""

from supershell.tui import effects
from supershell.tui.console import get_console

CHARACTER_PROPERTIES = {
    # Character ID: {display_name, style_string, char_delay}
    # UI Styles
    "system": {"name": "System", "style": "bold white", "delay": 0},
    "quest": {"name": "Quest Log", "style": "bold white", "delay": 0},
    # Character Styles
    "cypher": {"name": "Cypher", "style": "bold yellow", "delay": 0.04},
    "glitch": {"name": "Glitch", "style": "bold bright_black", "delay": 0.02},
    "hunter": {"name": "The Hunter", "style": "bold red", "delay": 0},
}


def say(message: str, character: str = "cypher"):
    """
    Prints a message to the console from a specific character.
    """
    # Look up the character's properties
    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])

    # Call the typewriter tool with the character's properties
    effects.typewriter_print(
        message=message,
        prefix=f"{props['name']}:",
        prefix_style=props["style"],
        message_style="stdout",
        char_delay=props["delay"],
    )


def ask(prompt: str, character: str = "cypher") -> str:
    """
    Asks the user a question *as an character*.
    """
    console = get_console()

    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])
    style = props["style"]
    name = props["name"]

    # Print character's name as colored text (if necessary) and
    # text as normal stdout style
    rich_prompt = f"  [{style}]{name}: {prompt}[/{style}] [prompt]> [/prompt]"
    return console.input(rich_prompt)
