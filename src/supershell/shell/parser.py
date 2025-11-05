"""
Parses user input to differentiate game commands from bash commands.

This is the main "router" for the game loop.
"""

from supershell.tui import dialogue
from supershell.tui.console import get_console
from supershell.game import quest_manager

# The set of "verbs" that are specific to the game
GAME_COMMANDS = {
    "help",
    "quest",
    "log",
    "cypher",
}

def parse_and_handle(command_str: str) -> bool:
    """
    Checks if a command is a game command and handles it.

    Args:
        command_str: The raw string from the user.

    Returns:
        True if the command was a game command and was handled.
        False if it is a bash command (and should be passed to the executor).
    """
    parts = command_str.strip().lower().split()
    
    if not parts:
        return False  # Empty input is not a game command

    verb = parts[0]
    args = parts[1:]

    # Is the first word a known game command?
    if verb in GAME_COMMANDS:
        try:
            # --- Route to the correct handler ---
            if verb == "help":
                _handle_help(args)
            elif verb == "quest" or verb == "log":
                _handle_quest(args)
            elif verb == "cypher":
                _handle_cypher(args)
            
            return True  # We handled it!
            
        except Exception as e:
            # Gracefully handle errors in our game commands
            console = get_console()
            console.log(f"[danger]Error in game command '{verb}': {e}[/danger]")
            dialogue.say(f"My apologies, Operator. My '{verb}' function seems to be corrupted.", actor="cypher")
            return True # Still "handled," just with an error

    # If it's not in GAME_COMMANDS, it's a bash command
    return False

# --- Private Handler Functions ---
# These functions do the actual work for each game command.

def _handle_help(args: list[str]):
    """Handler for the 'help' command."""
    if not args:
        dialogue.say(
            "I'm here to help!\n\n"
            "This is `supershell`, an interactive terminal. "
            "You can type any bash command (like `ls`, `pwd`, `cd`) "
            "and it will run just like a real terminal.\n\n"
            "I also have special **game commands**:\n"
            "  * [bold cyan]quest[/bold cyan]:   Show your current objectives.\n"
            "  * [bold cyan]cypher[/bold cyan]:  Talk to me directly (try `cypher hint`).\n"
            "  * [bold cyan]help[/bold cyan]:    You are here.\n",
            actor="cypher"
        )
    elif args[0] == "quest" or args[0] == "log":
        dialogue.say("The `quest` or `log` command shows your mission objectives. It's your 'to-do' list.", actor="cypher")
    elif args[0] == "cd":
        dialogue.say("`cd` stands for 'change directory'. You use it to move. For example: `cd /var/log`", actor="cypher")
    else:
        dialogue.say(f"I don't have a specific help file for `{args[0]}`. Try asking the `man` with `man {args[0]}`", actor="cypher")

def _handle_quest(args: list[str]):
    """Handler for the 'quest' or 'log' command."""
    console = get_console()
    quest_panel = quest_manager.get_quest_display()
    console.print(quest_panel)

def _handle_cypher(args: list[str]):
    """Handler for the 'cypher' command."""
    if not args:
        dialogue.say(message="I'm here, operator. Did you need something? You can ask me for a `hint`.", actor="cypher")
    elif args[0] == "hint":
        hint = quest_manager.get_contextual_hint()
        dialogue.say(hint, actor="cypher")
    elif args[0] in ("status", "lore"):
        dialogue.say("My origins? They're... complicated. I'm just a fragment, really. Trying to keep the signal alive.", actor="cypher")
    else:
        dialogue.say(f"I don't understand `{args[0]}`. Try `cypher hint` or `cypher status`.", actor="cypher")

