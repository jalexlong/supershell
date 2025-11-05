"""
Handles all TUI output for in-game actors.
Includes a controllable typewriter effect
that pauses at punctuation.
"""

import time
import random
from rich.text import Text
from supershell.tui.console import get_console

ACTOR_STYLES = {
    # Actor ID: ("Name", "Style from console.py", "Typing Speed")
    "glitch": ("Glitch", "glitch"),
    "cypher": ("Cypher", "cypher"),
    "hunter": ("Hunter", "hunter"),
    "system": ("System", "system"),
    "mission": ("New Quest", "system"),
    "log": ("[bold]Quest Log[/bold]", "system"),
}

# (0.04 = ~25 chars/sec).
DEFAULT_CHAR_DELAY = 0.04

def say(message: str, actor: str = "cypher", char_delay: float | None = None):
    """
    Prints a message to the console as if the actor is saying it.
    
    Args:
        actor: The ID of the actor (e.g., "glitch", "system").
               This ID is also used as the theme style.
        message: The text to print.
    """
    console = get_console()
    
    # Get the (Title, Style) from our dictionary.
    title, style = ACTOR_STYLES.get(actor, (actor, "system"))

    # Print the "Speaker" part instantly
    console.print(f"[{style}]{title}:[/{style}] ", end="")

    # Determine the speed
    if char_delay is None:
        delay = DEFAULT_CHAR_DELAY
    else:
        delay = char_delay

    # If delay is 0, print it instantly
    if delay <= 0:
        console.print(message)
        return

    # Else, handle the typewriter effect
    text = Text.from_markup(message)

    for char in text:
        console.print(char, end="")
        console.file.flush()

        # Pause before printing next character,
        #   and longer for punctuation
        base_sleep = delay * random.uniform(0.7, 1.4)
        if char.plain in (',', '...'):
            time.sleep(base_sleep + 0.3)
        elif char.plain in ('.', '!', '?'):
            time.sleep(base_sleep + 0.6)
        else:
            time.sleep(base_sleep)
    # Print newline for the terminal prompt
    console.print()

def ask(prompt: str, actor: str = "cypher") -> str:
    """
    Asks the user a question *as an actor*.
    """
    console = get_console()
    
    title, style = ACTOR_STYLES.get(actor, (actor.capitalize(), "system"))

    rich_prompt = f"  [{style}]{title}: {prompt}[/{style}] [prompt]> [/prompt]"
    return console.input(rich_prompt)
