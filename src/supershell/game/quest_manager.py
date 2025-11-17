import importlib
import json
import logging
import os
from pathlib import Path
from typing import Any, Optional, OrderedDict

from rich.panel import Panel

from supershell.game.base_quest import BaseQuest
from supershell.game.models import Objective

_quests: OrderedDict[str, BaseQuest] = OrderedDict()
_current_quest_id: Optional[str] = None
_active_quest_obj: Optional[BaseQuest] = None
_quests_loaded: bool = False

_SAVE_FILE_PATH = os.path.expanduser("~/.local/share/supershell/save.json")

log = logging.getLogger(__name__)


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
            log.info("Save file was present, but quest ID was invalid. Starting fresh.")
            _current_quest_id = list(_quests.keys())[0]
            _active_quest_obj = _quests[_current_quest_id]

    except (IOError, json.JSONDecodeError) as e:
        log.error(f"Failed to load progress from {_SAVE_FILE_PATH}: {e}")

    return completed_ids


def get_save_data() -> dict:
    if not os.path.exists(_SAVE_FILE_PATH):
        return {}
    try:
        with open(_SAVE_FILE_PATH, "r") as f:
            return json.load(f)
    except (IOError, json.JSONDecodeError):
        return {}


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


def get_quest_by_id(quest_id: str) -> Optional[BaseQuest]:
    return _quests.get(quest_id)


def load_quests():
    global _current_quest_id, _active_quest_obj, _quests_loaded
    if _quests_loaded:
        return

    quest_dir = Path(__file__).parent / "quests"

    if not quest_dir.exists():
        log.error(f"Quest script directory not found at: {quest_dir}")
        return

    quest_files = sorted(list(quest_dir.glob("quest_*.py")))

    for quest_file in quest_files:
        module_name = f"supershell.game.quests.{quest_file.stem}"
        try:
            module = importlib.import_module(module_name)
            QuestClass = getattr(module, "Quest")
            quest_instance = QuestClass()
            _quests[quest_instance.id] = quest_instance
            log.info(f"Loaded quest script: {quest_instance.title}")
        except Exception as e:
            log.error(f"Failed to load quest module {module_name}: {e}", exc_info=True)

    if _quests:
        _current_quest_id = list(_quests.keys())[0]
        _active_quest_obj = _quests[_current_quest_id]

        completed_ids = _load_progress()

        log.info("Syncing world state for all loaded quests...")
        for quest in _quests.values():
            quest.sync_world_state(completed_ids)

        log.info(f"Loaded {len(_quests)} quests. Current quest: {_current_quest_id}")
    else:
        log.warning("No quests were loaded.")

    _quests_loaded = True


def get_current_quest() -> Optional[BaseQuest]:
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


def advance_to_next_objective() -> bool:
    quest = get_current_quest()
    if not quest:
        return False
    if all(obj.completed for obj in quest.objectives):
        return False
    else:
        return True


def advance_quest() -> bool:
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
            _active_quest_obj.on_quest_start()
            return True
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
        if obj.required_cwd:
            full_required_path = os.path.expanduser(obj.required_cwd)
            current_path = os.getcwd()

            if current_path != full_required_path:
                display_path = obj.required_cwd.replace(os.path.expanduser("~"), "~", 1)
                return f"You need to be in the `{display_path}` directory to complete this task. Try `cd {display_path}`."

        return obj.hint
    return "I don't have a specific hint right now. Check your `quest` log."


def cleanup_all_quest_files():
    log.info("Running cleanup for all quests...")
    if not _quests:
        return
    for quest in _quests.values():
        quest._cleanup_quest_files()
