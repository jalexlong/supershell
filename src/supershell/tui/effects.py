"""
This file provides TUI presentation effects, like a typewriter
"""

import time
import random
from rich.text import Text
from supershell.tui.console import get_console

def typewriter_print(
    message: str,
    prefix: str = "",
    prefix_style: str = "default",
    message_style: str = "default",
    char_delay: float = 0.04
):
    """
    A generic, "dumb" typewriter function.
    
    Args:
        message: The text to print (can include Rich markup).
        prefix: A string to print instantly before the message.
        prefix_style: The Rich style for the prefix.
        message_style: The default Rich style for the message text.
        char_delay: Time in seconds between chars.
    """
    console = get_console()
    
    # Print the "Speaker" prefix instantly
    if prefix:
        console.print(f"[{prefix_style}]{prefix}[/{prefix_style}] ", end="")

    # Handle the typing effect
    if char_delay <= 0:
        # If no delay, print the whole message at once
        console.print(message, style=message_style)
        return

    text = Text.from_markup(message, style=message_style)
    
    for char in text:
        console.print(char, end="")
        console.file.flush()
        
        # Add the punctuation-pause logic
        base_sleep = char_delay * random.uniform(0.7, 1.4)
        if char.plain in (',', '...'):
            time.sleep(base_sleep + 0.3)
        elif char.plain in ('.', '!', '?'):
            time.sleep(base_sleep + 0.6)
        else:
            time.sleep(base_sleep)
    
    # Print the final newline
    console.print()
