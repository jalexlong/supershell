"""
This is the "Smart" Event Handler. It's a "script executor"
that runs the 'on_complete_script' from an objective.
"""

import logging

from supershell.game import actions, quest_manager
from supershell.tui import dialogue

log = logging.getLogger(__name__)


def handle_game_start(user: str, host: str):
    """
    Runs once at the very beginning of the game.
    """
    dialogue.say(f"System online. You are {user}@{host}.", character="system")
    dialogue.say(
        "Welcome to supershell. Your next objective is loaded.", character="system"
    )

    current_quest = quest_manager.get_current_quest()
    if current_quest:
        # Run the "intro cutscene" script
        log.info(f"Running on_quest_start script for {current_quest.id}")
        for action_data in current_quest.on_quest_start:
            actions.run_action(action_data)
    else:
        dialogue.say("No quests loaded. System idle.", character="cypher")


def handle_objective_completion(completed_id: str):
    """
    The main event router.
    """
    log.debug(f"Event: {completed_id}")

    # 1. Get the *data* for the completed objective
    obj = quest_manager.get_completed_objective(completed_id)
    if not obj:
        log.warning(f"Could not find objective data for {completed_id}")
        return

    # 2. Get the script from the objective
    script = obj.on_complete_script
    if not script:
        log.warning(f"Objective {completed_id} has no 'on_complete_script'.")
        # Default behavior: just advance
        actions.run_action({"action": "advance_objective"})
        return

    # 3. Execute the script
    log.info(f"Running on_complete_script for {completed_id}...")
    try:
        for action_data in script:
            actions.run_action(action_data)
    except Exception as e:
        log.error(f"Error in quest script for {completed_id}: {e}", exc_info=True)
