"""
Centralized Rich Console.

All other modules should import the `console` object from this file
to ensure consistent TUI output.
"""

from rich.console import Console
from rich.theme import Theme

supershell_theme = Theme(
    {
        # UI Styles
        "stdout": "grey74",
        "stderr": "bold bright_red",
        "system": "bold white",
        "prompt": "white",
        # Log Styles
        "info": "dim cyan",
        "warning": "bold yellow",
        "danger": "bold red",
    }
)

# Create the single, shared console instance
console = Console(theme=supershell_theme)


def get_console() -> Console:
    """Returns the shared console instance."""
    return console
