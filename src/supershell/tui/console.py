"""
Centralized Rich Console.

All other modules should import the `console` object from this file
to ensure consistent TUI output.
"""

from rich.console import Console
from rich.theme import Theme

supershell_theme = Theme({
    "info": "dim cyan",
    "warning": "bold yellow",
    "danger": "bold red",

    "system": "bold green",
    "hunter": "bold red",
    "glitch": "bright_black",  # This is the terminal's "dark grey"
    "cypher": "yellow",

    "stdout": "grey74",
    "stderr": "bold bright_red",
    "prompt": "white",
})

# Create the single, shared console instance
console = Console(theme=supershell_theme)

def get_console() -> Console:
    """Returns the shared console instance."""
    return console
