"""
Checks the result of a command against active quest objectives.
"""

import os
import logging
from supershell.game import quest_manager
from supershell.shell.executor import CommandResult

log = logging.getLogger(__name__)

def check(command_result: CommandResult) -> str | None:
    """
    Checks the command result against the active objective.
    If complete, returns the objective's ID.
    Otherwise, returns None.
    """
    active_obj = quest_manager.get_active_objective()
    if not active_obj:
        return None # No active quest

    log.debug(f"Checking {command_result.command} against {active_obj.id} ({active_obj.type})")
    
    try:
        is_complete = False
        
        # --- Route to the correct checker function ---
        if active_obj.type == "command_run":
            is_complete = _check_command_run(active_obj, command_result)
        
        elif active_obj.type == "path_exists":
            is_complete = _check_path_exists(active_obj, command_result)
            
        elif active_obj.type == "file_contains":
            is_complete = _check_file_contains(active_obj, command_result)

        elif active_obj.type == "any_command":
            is_complete = _check_any_command(active_obj, command_result)

        # --- Return the ID if complete ---
        if is_complete:
            log.info(f"Objective complete: {active_obj.id}")
            quest_manager.mark_objective_complete(active_obj.id)
            return active_obj.id # This is the "event"

    except Exception as e:
        log.error(f"Error during objective check for {active_obj.id}: {e}")
        
    return None # Not complete

# --- CHECKER FUNCTIONS ---

def _check_command_run(obj: quest_manager.Objective, res: CommandResult) -> bool:
    if res.return_code != 0:
        return False
    command_ran = res.command.strip().split()[0].lower()
    expected_command = obj.criteria.get('command')
    return command_ran == expected_command

def _check_path_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    path_to_check = obj.criteria.get('path')
    expected_type = obj.criteria.get('type')
    if not path_to_check: return False
    
    full_path = os.path.expanduser(path_to_check)
    if expected_type == 'dir':
        return os.path.isdir(full_path)
    elif expected_type == 'file':
        return os.path.isfile(full_path)
    return False

def _check_file_contains(obj: quest_manager.Objective, res: CommandResult) -> bool:
    path_to_check = obj.criteria.get('path')
    expected_content = obj.criteria.get('content')
    if not path_to_check or not expected_content: return False
    
    full_path = os.path.expanduser(path_to_check)
    if not os.path.isfile(full_path): return False
        
    try:
        with open(full_path, 'r') as f:
            return expected_content in f.read()
    except (IOError, OSError):
        return False

def _check_any_command(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """
    Checks if *any* command was run successfully.
    """
    return res.return_code == 0

