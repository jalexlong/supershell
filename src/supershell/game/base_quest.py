import logging
import os
import shutil
from abc import ABC, abstractmethod
from typing import List, Set

from supershell.game import quest_manager
from supershell.game.models import Objective
from supershell.shell.executor import CommandResult
from supershell.tui import dialogue

log = logging.getLogger(__name__)


class BaseQuest(ABC):
    def __init__(self):
        self.id: str = "base_quest"
        self.title: str = "Base Quest"
        self.description: str = "A template for a quest."
        self.objectives: List[Objective] = []
        self._tracked_files: Set[str] = set()
        self._tracked_dirs: Set[str] = set()

        self.__post_init__()

    def __post_init__(self):
        for obj in self.objectives:
            obj.completed = False

    def on_quest_start(self):
        pass

    @abstractmethod
    def on_objective_complete(self, completed_id: str, obj: Objective):
        pass

    @abstractmethod
    def on_objective_failure(self, command_result: CommandResult):
        active_obj = quest_manager.get_active_objective()
        if active_obj:
            dialogue.say(
                f"That's not quite it. Hint: {active_obj.hint}", character="cypher"
            )

    def handle_event(self, completed_id: str, obj: Objective):
        self.on_objective_complete(completed_id, obj)

        is_quest_still_active = quest_manager.advance_to_next_objective()
        if not is_quest_still_active:
            new_quest_was_loaded = quest_manager.advance_quest()
            if new_quest_was_loaded:
                dialogue.say(
                    f"Quest Complete: [bold]{self.title}[/bold]", character="mission"
                )

    def sync_world_state(self, completed_ids: set[str]):
        pass

    def _spawn_file(self, path: str, content: str = ""):
        full_path = os.path.expanduser(path)
        try:
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, "w") as f:
                f.write(content)
            log.info(f"Spawned file: {full_path}")
            self._tracked_files.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create file {full_path}: {e}")

    def _spawn_dir(self, path: str):
        full_path = os.path.expanduser(path)
        try:
            os.makedirs(full_path, exist_ok=True)
            log.info(f"Spawned tracked directory: {full_path}")
            self._tracked_dirs.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create directory {full_path}: {e}")

    def _cleanup_quest_files(self):
        log.info(f"Cleaning up files for quest: {self.id}")
        for f_path in self._tracked_files:
            try:
                os.remove(f_path)
            except FileNotFoundError:
                pass

        for d_path in self._tracked_dirs:
            try:
                shutil.rmtree(d_path)
            except FileNotFoundError:
                pass

        self._tracked_files.clear()
        self._tracked_dirs.clear()
