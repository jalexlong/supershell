"""
Handles the "Read" part of the REPL.
Uses prompt_toolkit for history and arrow key support.
"""

from prompt_toolkit import PromptSession
from prompt_toolkit.auto_suggest import AutoSuggest, Suggestion
from prompt_toolkit.completion import (
    Completer,
    PathCompleter,
    WordCompleter,
)
from prompt_toolkit.document import Document
from prompt_toolkit.history import InMemoryHistory
from prompt_toolkit.styles import Style

from supershell.shell.parser import GAME_COMMANDS
from supershell.tui.console import get_console

BASH_COMMANDS = [
    "ls",
    "cd",
    "pwd",
    "mkdir",
    "rm",
    "rmdir",
    "cp",
    "mv",
    "cat",
    "echo",
    "grep",
    "clear",
]

all_commands_to_suggest = list(GAME_COMMANDS) + BASH_COMMANDS

# Completer for first word (commands)
command_completer = WordCompleter(all_commands_to_suggest, ignore_case=False)

# Completer for second+ word (paths)
path_completer = PathCompleter()


class ConditionalFileOrCommandCompleter(Completer):
    """
    A smart completer that uses the WordCompleter for the first word
    and the PathCompleter for all subsequent words.
    """

    def __init__(self, command_completer, path_completer):
        self.command_completer = command_completer
        self.path_completer = path_completer

    def get_completions(self, document, complete_event):
        text = document.text_before_cursor
        words = text.split()

        # Check if we are on the first word
        if len(words) <= 1 and not text.endswith(" "):
            # We are on the first word, use command completer
            yield from self.command_completer.get_completions(document, complete_event)
        else:
            # We are on the second word or later, use path completer
            yield from self.path_completer.get_completions(document, complete_event)


class AutoSuggestFromCompleter(AutoSuggest):
    """
    Custom auto-suggest that provides suggestions from the session's completer.
    This creates the "fish-shell-like" inline suggestions.
    """

    def get_suggestion(self, buffer, document: Document) -> Suggestion | None:
        # Get the completer from the running session
        completer = buffer.completer
        if completer is None:
            return None

        # We only want to suggest if the cursor is at the end of the line
        if document.text_after_cursor:
            return None

        # Get completions
        try:
            # We pass 'complete_event' as None
            completions = list(completer.get_completions(document, None))

            if completions:
                # Take the first completion
                first_completion = completions[0]

                # 'first_completion.text' is the full word (e.g., "test_file.txt")
                # 'text_before_cursor' is what the user has typed (e.g., "mv test_f")

                # Find the part of the text we're actually completing
                word_being_completed = document.get_word_before_cursor(WORD=True)
                if not word_being_completed:
                    return None

                if first_completion.text.startswith(word_being_completed):
                    # The suffix is the part of the completion that
                    # the user *hasn't* typed yet
                    suffix = first_completion.text[len(word_being_completed) :]
                    if suffix:
                        return Suggestion(suffix)

        except Exception:
            # Completer might fail (e.g., during regex), just return None
            return None

        return None


# Instantiate our new completer
conditional_completer = ConditionalFileOrCommandCompleter(
    command_completer, path_completer
)

# We define a style for prompt_toolkit that matches our Rich theme.
# The 'prompt' style in Rich is 'bold green'.
_pt_style = Style.from_dict(
    {
        # This 'prompt' class name is arbitrary, we'll use it below.
        "auto-suggestion": "#666666",  # A faded, dark grey
        "cwd": "#007bff",  # Blue for the CWD
        "userhost": "bold #00ff00",  # Bold Green
    }
)

# We create a single, shared session at the module level.
# This is what stores and remembers the command history.
_session = PromptSession(
    history=InMemoryHistory(),
    completer=conditional_completer,
    auto_suggest=AutoSuggestFromCompleter(),
)


def get_command(prompt_parts: list[tuple[str, str]]) -> str:
    """
    Gets a line of input from the user using prompt_toolkit.

    Args:
        prompt: A list of (style_class, text) tuples for the prompt.
    """
    console = get_console()

    try:
        # Use the session's prompt method instead of console.input
        command = _session.prompt(prompt_parts, style=_pt_style)
        return command.strip()

    except KeyboardInterrupt:
        # User pressed Ctrl+C
        return ""  # Return empty string to re-prompt

    except EOFError:
        # User pressed Ctrl+D
        console.print("\n[system]...Disconnecting from supershell...[/system]")
        return "exit"
