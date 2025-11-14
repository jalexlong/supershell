"""
Loads and manages the game's quest state from YAML files.
"""

import json
import logging
import os
from pathlib import Path
from typing import Any, Optional, OrderedDict

import yaml
from rich.panel import Panel

# --- Import YAML-based Quest and Objective models ---
from supershell.game.models import Objective, Quest
from supershell.tui import console as rich_console

# --- Module-Level State ---
_quests: OrderedDict[str, Quest] = OrderedDict()
_current_quest_id: Optional[str] = None
_active_quest_obj: Optional[Quest] = None
# -------------------------

_SAVE_FILE_PATH = os.path.expanduser("~/.local/share/supershell/save.json")
log = logging.getLogger(__name__)

# --- Save/Load Functions (Unchanged from your file) ---


def get_save_data() -> dict:
    if not os.path.exists(_SAVE_FILE_PATH):
        return {}
    try:
        with open(_SAVE_FILE_PATH, "r") as f:
            return json.load(f)
    except (IOError, json.JSONDecodeError):
        return {}


def _save_progress():
    save_data = get_save_data()
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
    global _current_quest_id, _active_quest_obj
    completed_ids = set()
    if not os.path.exists(_SAVE_FILE_PATH):
        log.info("No save file found. Starting fresh.")
        return completed_ids
    try:
        with open(_SAVE_FILE_PATH, "r") as f:
            save_data = json.load(f)
        saved_completed_ids = save_data.get("completed_objectives", [])
        completed_ids = set(saved_completed_ids)
        for quest in _quests.values():
            for obj in quest.objectives:
                if obj.id in completed_ids:
                    obj.completed = True
        saved_quest_id = save_data.get("current_quest_id")
        if saved_quest_id and saved_quest_id in _quests:
            _current_quest_id = saved_quest_id
            if _current_quest_id:
                _active_quest_obj = _quests[_current_quest_id]
        else:
            if _quests:  # Fallback to first quest
                _current_quest_id = list(_quests.keys())[0]
                _active_quest_obj = _quests[_current_quest_id]
    except (IOError, json.JSONDecodeError) as e:
        log.error(f"Failed to load progress from {_SAVE_FILE_PATH}: {e}")
    return completed_ids


def set_save_data(key: str, value: Any):
    save_data = get_save_data()
    save_data[key] = value
    try:
        save_dir = os.path.dirname(_SAVE_FILE_PATH)
        os.makedirs(save_dir, exist_ok=True)
        with open(_SAVE_FILE_PATH, "w") as f:
            json.dump(save_data, f, indent=2)
        log.info(f"Custom data saved: {key}")
    except (IOError, OSError) as e:
        log.error(f"Failed to save custom data: {e}")


def get_quest_by_id(quest_id: str) -> Optional[Quest]:
    return _quests.get(quest_id)


# --- THIS IS THE NEW YAML LOADER ---
def load_quests():
    global _current_quest_id, _active_quest_obj
    _console = rich_console.get_console()

    # We load quests from outside the 'src' folder
    quest_dir = Path("assets/quests")
    if not quest_dir.exists():
        log.error(f"Quest directory not found at: {quest_dir}")
        return

    # Find all .yml files
    quest_files = sorted(list(quest_dir.glob("*.yml")))
    log.info(f"Found {len(quest_files)} quest files.")

    for quest_file in quest_files:
        try:
            with open(quest_file, "r") as f:
                data = yaml.safe_load(f)

            # Create a Quest object from the YAML data
            quest_instance = Quest.from_yaml(data)
            _quests[quest_instance.id] = quest_instance
            log.info(f"Loaded quest: {quest_instance.title}")

        except Exception as e:
            log.error(f"Failed to load quest {quest_file}: {e}", exc_info=True)

    if _quests:
        _current_quest_id = list(_quests.keys())[0]
        _active_quest_obj = _quests[_current_quest_id]

        _load_progress()  # Load save file

        # We no longer sync world state here, we
        # let the quest scripts do it via actions.

        log.info(f"Loaded {len(_quests)} quests. Current quest: {_current_quest_id}")
    else:
        log.warning("No quests were loaded.")


# --- (The rest of your functions are almost perfect) ---


def get_current_quest() -> Optional[Quest]:
    return _active_quest_obj


def get_active_objective() -> Optional[Objective]:
    quest = get_current_quest()
    if not quest:
        return None
    for obj in quest.objectives:
        if not obj.completed:
            return obj
    return None


def get_completed_objective(objective_id: str) -> Optional[Objective]:
    quest = get_current_quest()
    if not quest:
        return None
    for obj in quest.objectives:
        if obj.id == objective_id:
            return obj
    return None


def mark_objective_complete(objective_id: str):
    obj = get_completed_objective(objective_id)
    if obj:
        obj.completed = True
        log.info(f"Objective complete: {objective_id}")
        _save_progress()


def advance_to_next_objective() -> Optional[Objective]:
    """
    Checks if the current quest is finished.
    - If it is, returns None.
    - If it's not, it finds the next active objective and returns it.
    """
    quest = get_current_quest()
    if not quest:
        return None

    # Find the *next* uncompleted objective
    next_obj = get_active_objective()
    if next_obj:
        return next_obj

    # If we return None, it means the quest is finished
    return None


def advance_quest() -> bool:
    """
    Marks the current quest complete, advances to the next one.
    Returns True if a new quest was loaded.
    """
    global _current_quest_id, _active_quest_obj
    quest = get_current_quest()
    if not quest:
        return False

    quest_ids = list(_quests.keys())
    try:
        current_index = quest_ids.index(quest.id)
        if current_index + 1 < len(quest_ids):
            _current_quest_id = quest_ids[current_index + 1]
            _active_quest_obj = _quests[_current_quest_id]
            _save_progress()
            return True  # A new quest was started
        else:
            _current_quest_id = None
            _active_quest_obj = None
            _save_progress()
            return False
    except ValueError:
        _current_quest_id = None
        _active_quest_obj = None
        return False


def get_quest_display():
    quest = get_current_quest()
    if not quest:
        return Panel(
            "[info]No active quest.[/info]",
            title="[bold]Quest Log[/bold]",
            border_style="system",
        )

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
    obj = get_active_objective()
    if obj:
        return obj.hint
    return "I don't have a specific hint right now. Check your `quest` log."


def cleanup_all_quest_files():
    log.info("Running cleanup for all quests...")
    if not _quests:
        return
    for quest in _quests.values():
        quest._cleanup_quest_files()
