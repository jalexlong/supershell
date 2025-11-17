from prompt_toolkit import PromptSession
from prompt_toolkit.auto_suggest import AutoSuggest, Suggestion
from prompt_toolkit.completion import (
    Completer,
    Completion,
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
command_completer = WordCompleter(all_commands_to_suggest, ignore_case=False)
path_completer = PathCompleter()


class ConditionalFileOrCommandCompleter(Completer):
    def __init__(self, command_completer, path_completer):
        self.command_completer = command_completer
        self.path_completer = path_completer

    def get_completions(self, document, complete_event):
        text = document.text_before_cursor
        words = text.split()

        if len(words) <= 1 and not text.endswith(" "):
            yield from self.command_completer.get_completions(document, complete_event)
        else:
            yield from self.path_completer.get_completions(document, complete_event)


class AutoSuggestFromCompleter(AutoSuggest):
    def get_suggestion(self, buffer, document: Document) -> Suggestion | None:
        completer = buffer.completer
        if completer is None:
            return None

        if document.text_after_cursor:
            return None

        try:
            completions = list(completer.get_completions(document, None))
            if completions:
                first_completion = completions[0]
                word_being_completed = document.get_word_before_cursor(WORD=True)
                if not word_being_completed:
                    return None

                if first_completion.text.startswith(word_being_completed):
                    suffix = first_completion.text[len(word_being_completed) :]
                    if suffix:
                        return Suggestion(suffix)
        except Exception:
            return None
        return None


conditional_completer = ConditionalFileOrCommandCompleter(
    command_completer, path_completer
)

_pt_style = Style.from_dict(
    {
        "auto-suggestion": "#666666",
        "cwd": "#007bff",
        "userhost": "bold #00ff00",
    }
)

_session = PromptSession(
    history=InMemoryHistory(),
    completer=conditional_completer,
    auto_suggest=AutoSuggestFromCompleter(),
)


def get_command(prompt_parts: list[tuple[str, str]]) -> str:
    console = get_console()
    try:
        command = _session.prompt(prompt_parts, style=_pt_style)
        return command.strip()
    except KeyboardInterrupt:
        return ""
    except EOFError:
        console.print("\n[system]...Disconnecting from supershell...[/system]")
        return "exit"
