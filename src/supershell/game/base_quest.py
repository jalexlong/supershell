"""
This is the "template" (Base Class) for all quests.
It defines the *logic* for how to run a quest.
"""

import logging
import os
import shutil
from dataclasses import dataclass, field
from typing import List, Set

from supershell.game import quest_manager
from supershell.game.models import Objective
from supershell.shell.executor import CommandResult
from supershell.tui import dialogue

log = logging.getLogger(__name__)


@dataclass
class BaseQuest:
    """
    The BaseQuest class is a "logic engine" that knows how
    to run a list of Objective objects.
    """

    id: str = "base_quest"
    title: str = "Base Quest"
    description: str = "A template for a quest."

    # We must use 'default_factory' for mutable types like lists and sets
    # to prevent all quests from sharing the same list in memory.
    objectives: List[Objective] = field(default_factory=list)
    _tracked_files: Set[str] = field(default_factory=set)
    _tracked_dirs: Set[str] = field(default_factory=set)

    def __post_init__(self):
        """Initializes the quest after creation."""
        # Reset all objectives on load
        for obj in self.objectives:
            obj.completed = False

    def on_quest_start(self):
        """
        A "cutscene" function that runs ONCE when this
        quest is first loaded.
        """
        pass  # Child quests will override this.

    def on_objective_complete(self, completed_id: str, obj: Objective):
        """(HOOK) Runs when an objective is finished."""
        pass  # Quests override this to add custom dialogue and events.

    def on_objective_failure(self, command_result: CommandResult):
        """
        (HOOK) Runs when the user enters a command that
        triggers a 'fail_condition'.
        """
        active_obj = quest_manager.get_active_objective()
        if active_obj:
            dialogue.say(
                f"That's not quite it. Hint: {active_obj.hint}", character="cypher"
            )

    def handle_event(self, completed_id: str, obj: Objective):
        """
        (CONDUCTOR) Advances the quest to the next objective or starts the next quest.
        """
        self.on_objective_complete(completed_id, obj)

        is_quest_still_active = quest_manager.advance_to_next_objective()
        if not is_quest_still_active:
            new_quest_was_loaded = quest_manager.advance_quest()
            if new_quest_was_loaded:
                dialogue.say(
                    f"Quest Complete: [bold]{self.title}[/bold]", character="mission"
                )

    def sync_world_state(self, completed_ids: set[str]):
        """Called on game load. Re-creates any necessary files."""
        pass

    def _spawn_file(self, path: str, content: str = ""):
        """Creates a file AND tracks it for cleanup."""
        full_path = os.path.expanduser(path)
        try:
            # Use makedirs with exist_ok=True to ensure parent dirs exist
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, "w") as f:
                f.write(content)
            log.info(f"Spawned file: {full_path}")
            self._tracked_files.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create file {full_path}: {e}")

    def _spawn_dir(self, path: str):
        """Creates a directory AND tracks it for cleanup."""
        full_path = os.path.expanduser(path)
        try:
            # Use makedirs with exist_ok=True, which is safer
            os.makedirs(full_path, exist_ok=True)
            log.info(f"Spawned tracked directory: {full_path}")
            self._tracked_dirs.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create directory {full_path}: {e}")

    def _cleanup_quest_files(self):
        """Removes all files and dirs created by this quest."""
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
