"""
Handles all TUI output for in-game characters.
Supports both direct-to-console printing and buffered output for the daemon.
"""

import time
from typing import Callable, Optional

from supershell.tui import effects
from supershell.tui.console import get_console

# --- Global Output Handler ---
# This allows the daemon to "capture" output instead of printing it.
_output_handler: Optional[Callable[[str], None]] = None


def set_output_handler(handler: Optional[Callable[[str], None]]):
    """
    Sets a global handler to capture output from `say()`.
    If None, output is printed directly to the console.

    Args:
        handler: A function that takes a single string argument, or None.
    """
    global _output_handler
    _output_handler = handler


# --- Character Definitions ---
CHARACTER_PROPERTIES = {
    # Character ID: {display_name, style_string, char_delay}
    # UI Styles
    "system": {"name": "System", "style": "bold white", "delay": 0},
    "quest": {"name": "Quest Log", "style": "bold white", "delay": 0},
    # Character Styles
    "cypher": {"name": "Cypher", "style": "bold #CCCC33", "delay": 0.02},
    "glitch": {"name": "Glitch", "style": "bold bright_black", "delay": 0.02},
    "hunter": {"name": "The Hunter", "style": "bold #CC3333", "delay": 0},
}


def say(message: str, character: str = "cypher"):
    """
    Prints a message.

    If an output handler is set (e.g., in the daemon), it sends the formatted
    string to the handler. Otherwise, it prints directly to the console.
    """
    # Look up the character's properties
    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])

    # Format the message using Rich-style markup for the daemon to interpret.
    # The client-side will eventually render this with Rich.
    formatted_message = (
        f"[{props['style']}]{props['name']}:[/{props['style']}] {message}"
    )

    if _output_handler:
        # If a handler is set, send the raw, formatted string to it.
        # We don't use the typewriter effect here because the daemon needs
        # to send the full text back to the client immediately.
        _output_handler(formatted_message)
    else:
        # Otherwise, print to the console with the typewriter effect for the
        # old, non-daemon execution path (if any remains).
        effects.typewriter_print(
            message=message,
            prefix=f"{props['name']}:",
            prefix_style=props["style"],
            message_style="stdout",
            char_delay=props["delay"],
        )


def say_speech(speech: list[str], character: str = "cypher"):
    """
    Prints a sequence of messages from a character.
    """
    for line in speech:
        say(line, character)
        # In non-daemon mode, this adds a dramatic pause.
        # In daemon mode, this will have no effect as the handler is synchronous.
        if not _output_handler:
            time.sleep(0.2)


def ask(prompt: str, character: str = "cypher") -> str:
    """
    Asks the user a question *as a character*.

    NOTE: This function is part of the old architecture and will only work when
    no output handler is set. The daemon architecture does not support direct
    synchronous input from the user.
    """
    if _output_handler:
        # Cannot ask for input in daemon mode. This is a developer error.
        say(
            "Warning: `ask()` was called in daemon mode. This is not supported.",
            character="system",
        )
        return ""

    console = get_console()
    props = CHARACTER_PROPERTIES.get(character, CHARACTER_PROPERTIES["system"])
    style = props["style"]
    name = props["name"]

    rich_prompt = f"  [{style}]{name}: {prompt}[/{style}] [prompt]> [/prompt]"
    return console.input(rich_prompt)
