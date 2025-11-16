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
        "stderr": "bold #CC3333",
        "system": "bold grey82",
        "prompt": "grey82",
        # Log Styles
        "info": "dim cyan",
        "warning": "bold #CCCC33",
        "danger": "bold #CC3333",
    }
)

# Create the single, shared console instance
console = Console(theme=supershell_theme)


def get_console() -> Console:
    """Returns the shared console instance."""
    return console
