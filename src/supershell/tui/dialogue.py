"""
Handles all TUI output for in-game characters.
This is the async version with skippable typewriter.
"""
import asyncio
from supershell.tui import effects
from supershell.tui.console import get_console

CHARACTER_PROPERTIES = {
    # Actor ID: {display_name, style_string, char_delay}
    "system": {
        "name": "System",
        "style": "bold white",
        "delay": 0
    },
    "quest": {
        "name": "Quest Log",
        "style": "bold white",
        "delay": 0.01
    },
    "log": {
        "name": "[bold]Quest Log[/bold]",
        "style": "bold white",
        "delay": 0
    },
    "cypher": {
        "name": "Cypher",
        "style": "yellow",
        "delay": 0.05
    },
    "glitch": {
        "name": "Glitch",
        "style": "bold bright_black",
        "delay": 0.02
    },
    "hunter": {
        "name": "The Hunter",
        "style": "bold red",
        "delay": 0
    },
}

def say(message: str, character: str = "cypher"):
    """
    Prints a message to the console from a specific character.
    This is a SYNCHRONOUS Function that *calls* an ASYNC task.
    """
    # Look up the character's properties
    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])
    
    char_delay = props['delay']
    prefix = props['name'] + ":"
    prefix_style = props['style']

    # Handle typing effect
    if char_delay <= 0:
        # If no delay, print instantly and return
        console = get_console()
        console.print(prefix, style=prefix_style, end=" ")
        console.print(message)
        return

    # This will take over the terminal, run the typewriter,
    # and listen for 'Enter' to skip.
    try:
        asyncio.run(effects.typewriter_effect(
            message=message,
            prefix=prefix,
            char_delay=char_delay
        ))
    except (KeyboardInterrupt, EOFError):
        # If asyncio.run() is interrupted, print the rest
        console = get_console()
        console.print(f"\r{prefix} {message}")

def ask(prompt: str, character: str = "cypher") -> str:
    """
    Asks the user a question *as an character*.
    """
    console = get_console()
    
    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])
    style = props['style']
    name = props['name']

    # Print character's name as colored text (if necessary) and
    # text as normal stdout style
    rich_prompt = f"  [{style}]{name}: {prompt}[/{style}] [prompt]> [/prompt]"
    return console.input(rich_prompt)

