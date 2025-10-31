"""
Checks the result of a command against active quest objectives.
"""

import os
import logging
from supershell.game import quest_manager
from supershell.shell.executor import CommandResult
from supershell.tui import cypher

log = logging.getLogger(__name__)

def check(command_result: CommandResult):
    """
    The main check function, called by the game loop after every command.
    """
    active_obj = quest_manager.get_active_objective()
    
    # If there's no active objective, we don't need to check anything.
    if not active_obj:
        return
    
    # Don't check if the command itself failed, unless the objective
    # is specifically to *cause* a failure (which we don't have yet).
    if command_result.return_code != 0:
        return

    log.debug(f"Checking {command_result.command} against {active_obj.id} ({active_obj.type})")
    
    try:
        is_complete = False
        
        # --- Route to the correct checker function ---
        if active_obj.type == "command_run":
            is_complete = _check_command_run(active_obj, command_result)
        
        elif active_obj.type == "path_exists":
            is_complete = _check_path_exists(active_obj, command_result)
        
        # (Future)
        # elif active_obj.type == "network_state":
        #    is_complete = _check_network_state(active_obj, command_result)

        # --- If complete, handle success ---
        if is_complete:
            _handle_objective_success(active_obj)

    except Exception as e:
        log.error(f"Error during objective check for {active_obj.id}: {e}")

def _handle_objective_success(obj: quest_manager.Objective):
    """
    Marks an objective complete and checks if the whole quest should advance.
    """
    # 1. Mark this one objective as done
    quest_manager.mark_objective_complete(obj.id)
    
    # 2. Give the user success feedback
    cypher.say(obj.success_message)
    
    # 3. Check if this was the *last* objective in the quest
    if quest_manager.get_active_objective() is None:
        quest_manager.advance_quest()

# --- Specific Checker Functions ---

def _check_command_run(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """Checks if the user ran a specific command."""
    
    # Get the "verb" of the command, e.g. "ls" from "ls -l"
    command_ran = res.command.strip().split()[0].lower()
    expected_command = obj.criteria.get('command')
    
    return command_ran == expected_command

def _check_path_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """
    Checks if a file or directory now exists (e.g., after 'mkdir').
    This check runs *after* the command, so it sees the new state.
    """
    path_to_check = obj.criteria.get('path')
    expected_type = obj.criteria.get('type') # 'file' or 'dir'
    
    if not path_to_check:
        return False
        
    # os.path.expanduser handles paths like '~/safehouse'
    full_path = os.path.expanduser(path_to_check)
    
    if expected_type == 'dir':
        return os.path.isdir(full_path)
    elif expected_type == 'file':
        return os.path.isfile(full_path)
        
    return False

