"""
Loads and manages the game's quest state
"""

import importlib
import json
import logging
import os
from pathlib import Path
from typing import Any, Optional, OrderedDict

from rich.panel import Panel

from supershell.game.base_quest import BaseQuest
from supershell.game.models import Objective

# --- Module-Level State ---
# This now holds the *instance* of the quest class
_quests: OrderedDict[str, BaseQuest] = OrderedDict()
_current_quest_id: Optional[str] = None
_active_quest_obj: Optional[BaseQuest] = None
# -------------------------

_SAVE_FILE_PATH = os.path.expanduser("~/.local/share/supershell/save.json")

log = logging.getLogger(__name__)


def _save_progress():
    """Saves the current quest and custom data to the JSON file."""
    # Start with any custom data that's *already* in the file
    save_data = get_save_data()

    # Overwrite with *quest* data
    completed_ids = []
    for quest in _quests.values():
        for obj in quest.objectives:
            if obj.completed:
                completed_ids.append(obj.id)

    save_data["current_quest_id"] = _current_quest_id
    save_data["completed_objectives"] = completed_ids

    try:
        save_dir = os.path.dirname(_SAVE_FILE_PATH)
        os.makedirs(save_dir, exist_ok=True)
        with open(_SAVE_FILE_PATH, "w") as f:
            json.dump(save_data, f, indent=2)
        log.info(f"Progress saved to {_SAVE_FILE_PATH}")
    except (IOError, OSError) as e:
        log.error(f"Failed to save progress: {e}")


def _load_progress():
    """Loads quest state from the JSON file and applies it."""
    global _current_quest_id, _active_quest_obj

    completed_ids = set()

    if not os.path.exists(_SAVE_FILE_PATH):
        log.info("No save file found. Starting fresh.")
        return completed_ids

    try:
        with open(_SAVE_FILE_PATH, "r") as f:
            save_data = json.load(f)

        # Apply the saved state to our loaded quests
        saved_completed_ids = save_data.get("completed_objectives", [])
        completed_ids = set(saved_completed_ids)

        for quest in _quests.values():
            for obj in quest.objectives:
                if obj.id in completed_ids:
                    obj.completed = True

        # Set the current quest
        saved_quest_id = save_data.get("current_quest_id")
        if saved_quest_id and saved_quest_id in _quests:
            _current_quest_id = saved_quest_id
            if _current_quest_id:
                _active_quest_obj = _quests[_current_quest_id]
        else:
            log.info("Save file was present, but quest ID was invalid. Starting fresh.")
            # Fallback to the first quest
            _current_quest_id = list(_quests.keys())[0]
            _active_quest_obj = _quests[_current_quest_id]

    except (IOError, json.JSONDecodeError) as e:
        log.error(f"Failed to load progress from {_SAVE_FILE_PATH}: {e}")

    return completed_ids


def get_save_data() -> dict:
    """Helper to read the entire save file."""
    if not os.path.exists(_SAVE_FILE_PATH):
        return {}
    try:
        with open(_SAVE_FILE_PATH, "r") as f:
            return json.load(f)
    except (IOError, json.JSONDecodeError):
        return {}


def set_save_data(key: str, value: Any):
    """Saves a single custom key/value pair."""
    save_data = get_save_data()
    save_data[key] = value  # Add or overwrite the key

    try:
        save_dir = os.path.dirname(_SAVE_FILE_PATH)
        os.makedirs(save_dir, exist_ok=True)
        with open(_SAVE_FILE_PATH, "w") as f:
            json.dump(save_data, f, indent=2)
        log.info(f"Custom data saved: {key}")
    except (IOError, OSError) as e:
        log.error(f"Failed to save custom data: {e}")


def get_quest_by_id(quest_id: str) -> Optional[BaseQuest]:
    """Finds a loaded quest class instance by its ID."""
    return _quests.get(quest_id)


def load_quests():
    """
    Loads all quest *modules* from the quests directory.
    """
    global _current_quest_id, _active_quest_obj

    # This path now points to our new quest scripts folder
    quest_dir = Path(__file__).parent / "quests"

    if not quest_dir.exists():
        log.error(f"Quest script directory not found at: {quest_dir}")
        return

    # Find all quest files (e.g., quest_00_...py, quest_01_...py)
    # We sort them to ensure "00" loads first.
    quest_files = sorted(list(quest_dir.glob("quest_*.py")))

    for quest_file in quest_files:
        # e.g., "supershell.game.quests.quest_00_orphan_chase"
        module_name = f"supershell.game.quests.{quest_file.stem}"
        try:
            # 1. Import the file as a module
            module = importlib.import_module(module_name)

            # 2. Get the 'BaseQuest' class from inside the file
            QuestClass = getattr(module, "Quest")

            # 3. Create an *instance* of the class
            quest_instance = QuestClass()

            # 4. Store the instance, keyed by its ID
            _quests[quest_instance.id] = quest_instance
            log.info(f"Loaded quest script: {quest_instance.title}")

        except Exception as e:
            log.error(f"Failed to load quest module {module_name}: {e}", exc_info=True)

    if _quests:
        # Set the first quest as the active one
        _current_quest_id = list(_quests.keys())[0]
        _active_quest_obj = _quests[_current_quest_id]

        # Load progress and get the list of completed IDs
        completed_ids = _load_progress()

        # Tell *all* quests to sync their state
        log.info("Syncing world state for all loaded quests...")
        for quest in _quests.values():
            quest.sync_world_state(completed_ids)

        log.info(f"Loaded {len(_quests)} quests. Current quest: {_current_quest_id}")
    else:
        log.warning("No quests were loaded.")


def get_current_quest() -> Optional[BaseQuest]:
    """Returns the full Quest *instance* for the active quest."""
    return _active_quest_obj


def get_active_objective() -> Optional[Objective]:
    """Finds the first non-completed objective in the current quest."""
    quest = get_current_quest()
    if not quest:
        return None

    for obj in quest.objectives:
        if not obj.completed:
            return obj
    return None


def get_completed_objective(objective_id: str) -> Optional[Objective]:
    """Finds a specific objective by its ID."""
    quest = get_current_quest()
    if not quest:
        return None
    for obj in quest.objectives:
        if obj.id == objective_id:
            return obj
    return None


def mark_objective_complete(objective_id: str):
    """Marks a specific objective as complete."""
    obj = get_completed_objective(objective_id)
    if obj:
        obj.completed = True
        log.info(f"Objective complete: {objective_id}")
        _save_progress()


def advance_to_next_objective() -> bool:
    """
    Checks if the current quest is finished.
    - If it is, returns False.
    - If it's not, it silently finds the next objective and returns True.
    """
    quest = get_current_quest()
    if not quest:
        return False

    # Check if all objectives are *already* complete
    if all(obj.completed for obj in quest.objectives):
        # This quest is done. Return False.
        return False
    else:
        # This quest is *not* done.
        # We don't need to do anything else.
        return True


def advance_quest() -> bool:
    """
    Marks the current quest complete, advances to the next one,
    and returns True if a new quest was loaded.
    """
    global _current_quest_id, _active_quest_obj
    quest = get_current_quest()
    if not quest:
        return False

    quest_ids = list(_quests.keys())
    try:
        current_index = quest_ids.index(quest.id)
        if current_index + 1 < len(quest_ids):
            # We have a new quest!
            _current_quest_id = quest_ids[current_index + 1]
            _active_quest_obj = _quests[_current_quest_id]
            _save_progress()  # Save that we changed quests

            # Run the "intro cutscene" for the new quest
            _active_quest_obj.on_quest_start()

            # A new quest was started
            return True
        else:
            # No more quests!
            _current_quest_id = None
            _active_quest_obj = None
            _save_progress()
            return False

    except ValueError:
        _current_quest_id = None
        _active_quest_obj = None
        return False


def get_quest_display():
    """Returns a Rich Panel for the 'quest' command."""
    quest = get_current_quest()
    if not quest:
        return Panel(
            "[info]No active quest.[/info]",
            title="[bold]Quest Log[/bold]",
            border_style="system",
        )

    # We read the data directly from the class instance
    output = [f"[bold]{quest.title}[/bold]\n", f"{quest.description}\n"]
    output.append("[bold]Objectives:[/bold]")

    for obj in quest.objectives:
        if obj.completed:
            output.append(f"  [dim]• {obj.description} (Done)[/dim]")
        else:
            output.append(f"  [white]• {obj.description}[/white]")
            break

    return Panel(
        "\n".join(output), title="[bold]Quest Log[/bold]", border_style="system"
    )


def get_contextual_hint() -> str:
    """Gets the hint for the current active objective."""
    obj = get_active_objective()
    if obj:
        return obj.hint
    return "I don't have a specific hint right now. Check your `quest` log."


def cleanup_all_quest_files():
    """
    Tells all loaded quests to run their _cleanup_quest_files() method.
    """
    log.info("Running cleanup for all quests...")
    if not _quests:
        return

    for quest in _quests.values():
        # We call the helper function you defined in BaseQuest
        quest._cleanup_quest_files()
