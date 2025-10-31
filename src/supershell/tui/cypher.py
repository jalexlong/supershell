"""
Handles all TUI output related to your companion, Cypher.
"""

from rich.text import Text
from supershell.tui.console import get_console

def say(message: str, title: str = "Cypher"):
    """
    Prints a message to the console as if Cypher is saying it.
    
    Args:
        message: The plain text message.
        title: The prefix to use (defaults to "Cypher").
    """
    console = get_console()
    
    # We wrap the *entire line* in the 'cypher' style tag.
    # Rich will apply the style and also parse any
    # formatting tags (like [bold]) inside the message.
    console.print(f"{title}: {message}", style="cypher")

def ask(prompt: str) -> str:
    """
    Asks the user a question *as Cypher*.
    """
    console = get_console()
    
    # We combine the "cypher" style for the prompt text
    # with the "prompt" style for the ">"
    rich_prompt = f"  [cypher]{prompt}[/cypher] [prompt]> [/prompt]"
    return console.input(rich_prompt)

