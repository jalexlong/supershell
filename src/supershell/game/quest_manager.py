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

from supershell.game import actions
from supershell.game.models import Objective, Quest

# --- Module-Level State ---
_quests: OrderedDict[str, Quest] = OrderedDict()
_current_quest_id: Optional[str] = None
_active_quest_obj: Optional[Quest] = None
_last_generated_secret_password: Optional[str] = None
# -------------------------

_SAVE_FILE_PATH = os.path.expanduser("~/.local/share/supershell/save.json")
log = logging.getLogger(__name__)

# --- Save/Load Functions (Unchanged) ---


def get_save_data() -> dict:
    if not os.path.exists(_SAVE_FILE_PATH):
        return {}
    try:
        with open(_SAVE_FILE_PATH, "r") as f:
            return json.load(f)
    except (IOError, json.JSONDecodeError):
        return {}


def _save_progress():
    log.debug(f"Attempting to save game progress to {_SAVE_FILE_PATH}")
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
        log.info(
            f"Progress saved to {_SAVE_FILE_PATH}. Current quest: {_current_quest_id}, Completed objectives: {len(completed_ids)}"
        )
    except (IOError, OSError) as e:
        log.error(f"Failed to save progress: {e}")


def _load_progress():
    global _current_quest_id, _active_quest_obj
    log.debug(f"Attempting to load game progress from {_SAVE_FILE_PATH}")
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
                # Only mark complete if it was in the saved state
                obj.completed = obj.id in completed_ids

        saved_quest_id = save_data.get("current_quest_id")
        if saved_quest_id and saved_quest_id in _quests:
            _current_quest_id = saved_quest_id
            if _current_quest_id:
                _active_quest_obj = _quests[_current_quest_id]
        else:
            if _quests:
                _current_quest_id = list(_quests.keys())[0]
                _active_quest_obj = _quests[_current_quest_id]
    except (IOError, json.JSONDecodeError) as e:
        log.error(f"Failed to load progress from {_SAVE_FILE_PATH}: {e}")
    log.debug(
        f"Progress loaded. Current quest ID: {_current_quest_id}, Active objectives marked complete."
    )
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


def load_quests():
    global _current_quest_id, _active_quest_obj
    log.debug("Initiating quest loading process...")

    # We load quests from outside the 'src' folder
    quest_dir = Path("assets/quests")
    if not quest_dir.exists():
        log.error(f"Quest directory not found at: {quest_dir}")
        return

    quest_files = sorted(list(quest_dir.glob("*.yml")))
    log.info(f"Found {len(quest_files)} quest files.")

    for quest_file in quest_files:
        try:
            with open(quest_file, "r") as f:
                data = yaml.safe_load(f)

            quest_instance = Quest.from_yaml(data)
            _quests[quest_instance.id] = quest_instance
            log.debug(
                f"Successfully loaded quest from file: {quest_file}, ID: {quest_instance.id}"
            )
        except Exception as e:
            log.error(f"Failed to load quest {quest_file}: {e}", exc_info=True)

    if _quests:
        log.debug(f"All quests loaded. First quest ID: {list(_quests.keys())[0]}")
        _current_quest_id = list(_quests.keys())[0]
        _active_quest_obj = _quests[_current_quest_id]

        log.debug("Loading progress from save file...")
        completed_ids = _load_progress()
        log.debug(f"Loaded completed objective IDs: {completed_ids}")

        log.info("Syncing world state for all loaded quests...")
        for quest in _quests.values():
            for action_data in quest.on_load_sync:
                log.debug(
                    f"Processing on_load_sync action: {action_data.get('action')} for quest {quest.id}"
                )
                not_completed = action_data.get("not_completed")
                if not_completed and not_completed in completed_ids:
                    log.debug(
                        f"Skipping on_load_sync action because objective '{not_completed}' is completed."
                    )
                    continue

                is_completed = action_data.get("id")
                if is_completed and is_completed not in completed_ids:
                    log.debug(
                        f"Skipping on_load_sync action because objective '{is_completed}' is NOT completed."
                    )
                    continue

                # Remove conditional keys which are not arguments for the action function
                run_params = action_data.copy()
                run_params.pop("id", None)
                run_params.pop("not_completed", None)
                log.debug(
                    f"Running on_load_sync action '{run_params.get('action')}' with params {run_params}"
                )
                actions.run_action(run_params)

        log.info(f"Loaded {len(_quests)} quests. Current quest: {_current_quest_id}")
    else:
        log.warning("No quests were loaded.")


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
        log.debug(f"Marking objective '{objective_id}' as complete.")
        obj.completed = True
        log.info(f"Objective complete: {objective_id}")
        _save_progress()
    else:
        log.warning(
            f"Attempted to mark non-existent objective '{objective_id}' as complete."
        )


def advance_to_next_objective() -> Optional[Objective]:
    log.debug("Attempting to advance to next objective.")
    quest = get_current_quest()
    if not quest:
        log.debug("No active quest to advance objective.")
        return None

    next_obj = get_active_objective()
    if next_obj:
        log.debug(f"Next active objective found: '{next_obj.id}'")
        return next_obj
    else:
        log.debug("No further active objectives in current quest.")
        return None


def advance_quest() -> Optional[Quest]:
    global _current_quest_id, _active_quest_obj
    quest = get_current_quest()
    if not quest:
        return None

    quest_ids = list(_quests.keys())
    try:
        current_index = quest_ids.index(quest.id)
        if current_index + 1 < len(quest_ids):
            _current_quest_id = quest_ids[current_index + 1]
            _active_quest_obj = _quests[_current_quest_id]
            _save_progress()
            return _active_quest_obj
        else:
            _current_quest_id = None
            _active_quest_obj = None
            _save_progress()
            return None
    except ValueError:
        _current_quest_id = None
        _active_quest_obj = None
        return None


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
        # NEW LOGIC FOR GENERAL DIRECTORY-AWARE HINTS
        if obj.required_cwd:  # Check if the objective explicitly states a required CWD
            full_required_path = os.path.expanduser(obj.required_cwd)
            current_path = os.getcwd()

            if current_path != full_required_path:
                # Provide a specific hint to change directory
                display_path = obj.required_cwd.replace(os.path.expanduser("~"), "~", 1)
                return f"You need to be in the `{display_path}` directory to complete this task. Try `cd {display_path}`."
        # END NEW LOGIC

        return obj.hint  # Default to the objective's hint
    return "I don't have a specific hint right now. Check your `quest` log."


def cleanup_all_quest_files():
    log.info("Running cleanup for all quests...")
    if not _quests:
        return
    for quest in _quests.values():
        quest._cleanup_quest_files()


def reset_current_quest_to_start():
    """Resets the current quest to its first objective and clears progress."""
    global _current_quest_id, _active_quest_obj
    current_quest = get_current_quest()

    if not current_quest:
        log.warning("No active quest to reset.")
        return

    log.info(f"Resetting quest '{current_quest.id}' to start.")

    # 1. Mark all objectives as incomplete
    for obj in current_quest.objectives:
        obj.completed = False

    # 2. Reset to the first objective
    _current_quest_id = current_quest.id
    _active_quest_obj = (
        current_quest  # Ensure the quest object itself is correctly referenced
    )

    # 3. Clean up and re-spawn initial world state for the quest
    current_quest._cleanup_quest_files()  # Remove existing tracked files/dirs
    # Re-run on_load_sync for the current quest to recreate necessary files
    log.info(f"Re-syncing world state for quest '{current_quest.id}'.")
    for action_data in current_quest.on_load_sync:
        run_params = action_data.copy()
        run_params.pop("id", None)
        run_params.pop("not_completed", None)
        actions.run_action(run_params)

    # 4. Save progress (which will now reflect the reset state)
    _save_progress()

    log.info(f"Quest '{current_quest.id}' reset successfully.")
