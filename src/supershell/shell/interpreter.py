"""
Handles the "Read" part of the REPL.
Uses prompt_toolkit for history and arrow key support.
"""

from supershell.tui.console import get_console
from supershell.shell.parser import GAME_COMMANDS

from prompt_toolkit import PromptSession
from prompt_toolkit.history import InMemoryHistory
from prompt_toolkit.styles import Style
from prompt_toolkit.completion import WordCompleter
from prompt_toolkit.auto_suggest import AutoSuggest, Suggestion
from prompt_toolkit.document import Document

BASH_COMMANDS = [
    'ls', 'cd', 'pwd', 'mkdir', 'rm', 'rmdir',
    'cp', 'mv', 'cat', 'echo', 'grep', 'clear',
    # Eventually add networking commands from networking quests
    # 'ip', 'ping', 'dhclient', 'ifconfig',
]

all_commands_to_suggest = list(GAME_COMMANDS) + BASH_COMMANDS
word_completer = WordCompleter(all_commands_to_suggest, ignore_case=False)


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
        
        # Get the word before the cursor
        word_before_cursor = document.get_word_before_cursor()
        if not word_before_cursor.strip():
            return None # Need a word to complete

        # Get completions
        try:
            # We pass 'complete_event' as None
            completions = list(completer.get_completions(document, None))
            if completions:
                # Take the first completion
                first_completion = completions[0]
                
                # Calculate the suffix (the part to add)
                suffix = first_completion.text[len(word_before_cursor):]
                
                if suffix:
                    return Suggestion(suffix)
        except Exception:
            # Completer might fail (e.g., during regex), just return None
            return None
        
        return None

# We define a style for prompt_toolkit that matches our Rich theme.
# The 'prompt' style in Rich is 'bold green'.
_pt_style = Style.from_dict({
    # This 'prompt' class name is arbitrary, we'll use it below.
    'auto-suggestion': '#666666',  # A faded, dark grey
    'cwd': '#007bff',  # Cyan for the CWD
    'userhost': 'bold #00ff00',  # Bold Green
})

# We create a single, shared session at the module level.
# This is what stores and remembers the command history.
_session = PromptSession(
    history=InMemoryHistory(),
    completer=word_completer,
    auto_suggest=AutoSuggestFromCompleter()
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
        command = _session.prompt(
            prompt_parts,
            style=_pt_style
        )
        return command.strip()
    
    except KeyboardInterrupt:
        # User pressed Ctrl+C
        return ""  # Return empty string to re-prompt
    
    except EOFError:
        # User pressed Ctrl+D
        console.print("\n[system]...Disconnecting from supershell...[/system]")
        return "exit"

