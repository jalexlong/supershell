"""
This is the "Atomic" Action Registry.
It maps action strings (from YAML) to the Python functions
that perform them.
"""

import logging
import os

from supershell.game import quest_manager
from supershell.tui import dialogue

log = logging.getLogger(__name__)

# --- 1. Define all "verbs" as simple functions ---
# These are the "tools" the engine can use.


def _action_say(character: str, message: str):
    """Prints dialogue to the screen."""
    dialogue.say(message, character=character)


def _action_advance_objective():
    """Tells the quest manager to advance."""
    _quest = quest_manager.get_current_quest()
    next_obj = quest_manager.advance_to_next_objective()

    if next_obj:
        # If there's a new objective, print its intro
        dialogue.say(f"New Objective: {next_obj.description}", character="mission")
    else:
        # This was the last objective, so advance the *quest*
        _action_advance_quest()


def _action_advance_quest():
    """Completes the current quest and loads the next one."""
    active_quest = quest_manager.get_current_quest()
    if not active_quest:
        dialogue.say("All objectives complete.", character="system")
        return

    new_quest_was_loaded = quest_manager.advance_quest()

    if new_quest_was_loaded:
        dialogue.say(
            f"Quest Complete: [bold]{active_quest.title}[/bold]", character="mission"
        )
    else:
        dialogue.say("All objectives complete.", character="system")


def _action_track_dir(path: str):
    """Tells the active quest to track a directory for cleanup."""
    active_quest = quest_manager.get_current_quest()
    if active_quest:
        active_quest._tracked_dirs.add(os.path.expanduser(path))
        log.info(f"Now tracking directory: {path}")


def _action_track_file(path: str):
    """Tells the active quest to track a file for cleanup."""
    active_quest = quest_manager.get_current_quest()
    if active_quest:
        active_quest._tracked_files.add(os.path.expanduser(path))
        log.info(f"Now tracking file: {path}")


# --- 2. Create the "Dispatch Map" ---
ACTION_REGISTRY = {
    "say": _action_say,
    "advance_objective": _action_advance_objective,
    "advance_quest": _action_advance_quest,
    "track_dir": _action_track_dir,
    "track_file": _action_track_file,
    # (You will add more actions here, like "spawn_file")
}


# --- 3. Create the "Executor" ---
def run_action(action_data: dict):
    """
    Looks up an action from the registry and runs it.
    """
    action_name = action_data.get("action")
    if not action_name:
        return

    # Get the correct function from our map
    func = ACTION_REGISTRY.get(action_name)
    if not func:
        log.warning(f"Unknown action in quest script: {action_name}")
        return

    # Prepare the arguments by removing 'action'
    params = action_data.copy()
    del params["action"]

    # Run the function with the parameters
    try:
        func(**params)
    except TypeError as e:
        log.error(f"Action '{action_name}' called with wrong arguments: {e}")
    except Exception as e:
        log.error(f"Error running action '{action_name}': {e}", exc_info=True)
