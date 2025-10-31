"""
Centralized Rich Console.

All other modules should import the `console` object from this file
to ensure consistent TUI output.
"""

from rich.console import Console
from rich.theme import Theme

# Define your Cypherpunk theme
# Feel free to expand this with more styles
cypher_theme = Theme({
    "info": "dim cyan",
    "warning": "bold yellow",
    "danger": "bold red",
    "system": "bold green",
    "prompt": "bold green",
    "cypher": "magenta",
    "stdout": "grey74",
    "stderr": "bold bright_red",
})

# Create the single, shared console instance
console = Console(theme=cypher_theme)

def get_console() -> Console:
    """Returns the shared console instance."""
    return console
