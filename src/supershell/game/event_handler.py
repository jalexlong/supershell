"""
This is a simple "Router" that delegates events
to the *active quest object*.
"""

import logging

from supershell.game import quest_manager
from supershell.tui import dialogue

log = logging.getLogger(__name__)


def handle_game_start(user: str, host: str):
    """
    Runs once at the very beginning of the game.
    """
    current_quest = quest_manager.get_current_quest()
    if not current_quest:
        dialogue.say("No quests loaded. System idle.", character="cypher")


def handle_objective_completion(completed_id: str):
    """
    The main event router.
    """
    log.debug(f"Event: {completed_id}")

    # 1. Get the *data* for the completed objective
    #   Also displays success_message
    obj = quest_manager.get_completed_objective(completed_id)
    if not obj:
        log.warning(f"Could not find objective data for {completed_id}")
        return

    # 2. Get the *active quest object* (the class instance)
    active_quest = quest_manager.get_current_quest()
    if not active_quest:
        log.warning(f"No active quest to handle event for {completed_id}")
        return

    # 3. Tell the quest to handle its own event
    try:
        active_quest.handle_event(completed_id, obj)
    except Exception as e:
        log.error(f"Error in quest script for {active_quest.id}: {e}", exc_info=True)
