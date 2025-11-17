"""
This is the "Atomic" Action Registry.
It maps action strings (from YAML) to the Python functions
that perform them.
"""

import inspect
import logging
import os
import random  # Added for password generation

from supershell.game import quest_manager
from supershell.shell import executor
from supershell.tui import dialogue

log = logging.getLogger(__name__)

# --- 1. Define all "verbs" as simple functions ---


def _action_say(character: str, message: str):
    """Prints dialogue to the screen."""
    dialogue.say(message, character=character)


def _action_say_speech(character: str, messages: list[str]):
    """Prints a list of messages to the screen."""
    dialogue.say_speech(speech=messages, character=character)


def _action_advance_objective():
    """Tells the quest manager to advance."""
    quest = quest_manager.get_current_quest()
    next_obj = quest_manager.advance_to_next_objective()

    if next_obj:
        if next_obj.description:  # Only print if description exists
            dialogue.say(f"New Objective: {next_obj.description}", character="mission")
    else:
        _action_advance_quest()


def _action_advance_quest():
    """Completes the current quest and loads the next one."""
    active_quest = quest_manager.get_current_quest()
    if not active_quest:
        dialogue.say("All objectives complete.", character="system")
        return

    new_quest = quest_manager.advance_quest()  # This returns the *new* quest

    if new_quest:
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


def _action_untrack_file(path: str):
    """Stops tracking a file (e.g., after 'rm' or 'mv')."""
    active_quest = quest_manager.get_current_quest()
    if active_quest:
        active_quest._tracked_files.discard(os.path.expanduser(path))
        log.info(f"Stopped tracking file: {path}")


def _action_spawn_file(path: str, content: str = ""):
    """Spawns a file (used by sync_world_state)."""
    active_quest = quest_manager.get_current_quest()
    if active_quest:
        active_quest._spawn_file(path, content)


def _action_spawn_dir(path: str):
    """Spawns a dir (used by sync_world_state)."""
    active_quest = quest_manager.get_current_quest()
    if active_quest:
        active_quest._spawn_dir(path)


def _action_conditional_say_on_fail(
    character: str,
    message: str,
    command_result: executor.CommandResult,
    if_command: str | None = None,
    if_args_contain: str | None = None,
):
    """Conditionally says a message if the command failed matches criteria."""
    should_say = True

    if if_command:
        if not command_result.command.strip().lower().startswith(if_command.lower()):
            should_say = False

    if if_args_contain and should_say:
        if if_args_contain not in command_result.command:
            should_say = False

    if should_say:
        dialogue.say(message, character=character)


def _action_reset_current_quest():
    """Resets the current quest to its starting state."""
    log.warning("Initiating quest reset.")
    quest_manager.reset_current_quest_to_start()


def _action_conditional_hard_fail(
    command_result: executor.CommandResult,
    if_command: str | None = None,
    if_args_contain: str | None = None,
    if_return_code: int | None = None,
):
    """Triggers a hard fail if the command matches criteria."""
    should_fail = True

    if if_command:
        if not command_result.command.strip().lower().startswith(if_command.lower()):
            should_fail = False

    if if_args_contain and should_fail:
        if if_args_contain not in command_result.command:
            should_fail = False

    if if_return_code is not None and should_fail:
        if command_result.return_code != if_return_code:
            should_fail = False

    if should_fail:
        log.error(f"HARD FAIL triggered by command: {command_result.command}")
        current_quest = quest_manager.get_current_quest()
        if current_quest and current_quest.on_hard_fail_script:
            for action_data in current_quest.on_hard_fail_script:
                # Pass command_result if action expects it
                run_params = action_data.copy()
                # The 'command_result' might not be expected by all actions in on_hard_fail_script
                # It will be filtered by run_action if not in the signature.
                run_params["command_result"] = command_result
                run_action(run_params)


def _action_generate_and_save_password():
    """Generates a random password, saves it, and makes it available for dialogue."""
    words = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta"]
    password = f"{random.choice(words)}-{random.choice(words)}"
    quest_manager.set_save_data("secret_password", password)
    quest_manager._last_generated_secret_password = (
        password  # Make available for dialogue
    )
    log.info(f"Generated and saved secret password: {password}")


def _action_cleanup_all_tracked_files():
    """Calls quest_manager to clean up all tracked files across all quests."""
    quest_manager.cleanup_all_quest_files()
    log.info("Triggered cleanup of all tracked quest files.")


# --- 2. Create the "Dispatch Map" ---
ACTION_REGISTRY = {
    "say": _action_say,
    "say_speech": _action_say_speech,
    "advance_objective": _action_advance_objective,
    "advance_quest": _action_advance_quest,
    "track_dir": _action_track_dir,
    "track_file": _action_track_file,
    "untrack_file": _action_untrack_file,
    "spawn_tracked_file": _action_spawn_file,
    "spawn_tracked_dir": _action_spawn_dir,
    "conditional_say_on_fail": _action_conditional_say_on_fail,
    "reset_current_quest": _action_reset_current_quest,
    "conditional_hard_fail": _action_conditional_hard_fail,
    "generate_and_save_password": _action_generate_and_save_password,
    "cleanup_all_tracked_files": _action_cleanup_all_tracked_files,
}


# --- 3. Create the "Executor" ---
def run_action(action_data: dict):
    """
    Looks up an action from the registry and runs it.
    """
    action_name = action_data.pop("action", None)
    if not action_name:
        return

    func = ACTION_REGISTRY.get(action_name)
    if not func:
        log.warning(f"Unknown action in quest script: {action_name}")
        return

    # Check if the function expects 'command_result'
    if "command_result" in action_data:
        sig = inspect.signature(func)
        if "command_result" not in sig.parameters:
            log.debug(
                f"Removing 'command_result' from params for action '{action_name}' as it's not expected."
            )
            action_data.pop("command_result", None)

    try:
        func(
            **action_data
        )  # Pass the remaining items in action_data as keyword arguments
    except TypeError as e:
        log.error(f"Action '{action_name}' called with wrong arguments: {e}")
    except Exception as e:
        log.error(f"Error running action '{action_name}': {e}", exc_info=True)
