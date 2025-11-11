"""
This is the "template" (Base Class) for all quests.
"""

import logging
import os
import shutil

from supershell.game import quest_manager
from supershell.game.models import Objective
from supershell.tui import dialogue

log = logging.getLogger(__name__)


class BaseQuest:
    id: str = "base_quest"
    title: str = "Base Quest"
    description: str = "A template for a quest."
    objectives: list[Objective] = []

    def __init__(self):
        """
        Initializes the quest. We'll add a list
        to track all files this quest creates.
        """
        self._tracked_files = set()
        self._tracked_dirs = set()

        # Reset all objectives on load
        for obj in self.objectives:
            obj.completed = False

    def on_quest_start(self):
        """
        A "cutscene" function that runs ONCE when this
        quest is first loaded.
        """
        # The base version does nothing.
        # Child quests will override this.
        pass

    def on_objective_complete(self, completed_id: str, obj: Objective):
        """
        Runs when an objective is finished.
        Quests override this to add custom dialogue and events.
        """
        # The base version just prints the default success message.
        pass

    def handle_event(self, completed_id: str, obj: Objective):
        """
        Advances the quest to the next objective or starts the next quest.
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
        """
        Called on game load. Re-creates any necessary files
        based on the full list of completed objectives.

        Args:
            completed_ids: A *set* of all completed objective IDs
                           from the save file.
        """
        # This base quest does nothing.
        # Child quests will override this.
        pass

    def _spawn_file(self, path: str, content: str = ""):
        """
        Creates a file AND tracks it for cleanup.
        """
        full_path = os.path.expanduser(path)
        try:
            with open(full_path, "w") as f:
                f.write(content)
            log.info(f"Spawned file: {full_path}")
            self._tracked_files.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create file {full_path}: {e}")

    def _spawn_dir(self, path: str):
        """
        Creates a directory AND tracks it for cleanup.
        """
        full_path = os.path.expanduser(path)
        try:
            os.mkdir(full_path)
            log.info(f"Spawned tracked directory: {full_path}")
            self._tracked_dirs.add(full_path)
        except FileExistsError:
            log.warning(f"Tracked directory {full_path} already exists.")
        except (IOError, OSError) as e:
            log.error(f"Could not create directory {full_path}: {e}")

    def _cleanup_quest_files(self):
        """
        Removes all files and dirs created by this quest.
        """
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
