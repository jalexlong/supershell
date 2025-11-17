import logging
import os
from dataclasses import dataclass, field
from typing import List

from supershell.game import quest_manager
from supershell.game.base_quest import BaseQuest, Objective
from supershell.shell.executor import CommandResult
from supershell.tui import dialogue

log = logging.getLogger(__name__)


@dataclass
class Quest(BaseQuest):
    id: str = "quest_01_boot_camp"
    title: str = "Boot Camp"
    description: str = "Time for your first lesson. We'll learn the basics: how to look, move, and build."
    objectives: List[Objective] = field(
        default_factory=lambda: [
            Objective(
                id="01_a_whoami",
                type="command_run",
                description="Learn who you're logged in as.",
                criteria={"command": "whoami"},
                hint="Type `whoami` to see your user.",
            ),
            Objective(
                id="01_b_pwd",
                type="command_run",
                description="Find your 'Present Working Directory.'",
                criteria={"command": "pwd"},
                hint="Type `pwd` to see your 'present working directory'.",
            ),
            Objective(
                id="01_c_mkdir",
                type="path_exists",
                description="Make a directory named 'bootcamp'.",
                criteria={"path": "~/bootcamp", "type": "dir"},
                hint="Make a practice folder: `mkdir bootcamp`",
            ),
            Objective(
                id="01_d_cd",
                type="cwd_is",
                description="Change directories into 'bootcamp'.",
                criteria={"path": "~/bootcamp"},
                hint="Now, 'change directory' into it: `cd bootcamp`",
            ),
            Objective(
                id="01_e_touch",
                type="path_exists",
                description="Make a new file named 'test.txt'.",
                criteria={"path": "~/bootcamp/test.txt", "type": "file"},
                hint="Create an empty file: `touch test.txt`",
            ),
            Objective(
                id="01_f_mv",
                description="Rename 'test.txt' to 'test_renamed.txt'",
                type="checklist",
                criteria={
                    "objectives": [
                        {
                            "type": "path_not_exists",
                            "criteria": {"path": "~/bootcamp/test.txt"},
                            "description": "Original file 'test.txt' is gone.",
                        },
                        {
                            "type": "path_exists",
                            "criteria": {
                                "path": "~/bootcamp/test_renamed.txt",
                                "type": "file",
                            },
                            "description": "New file 'test_renamed.txt' exists.",
                        },
                    ]
                },
                hint="Let's rename it: `mv test.txt test_renamed.txt`",
            ),
            Objective(
                id="01_g_cp",
                type="path_exists",
                description="Copy 'test_renamed.txt' to 'test_copy.txt'",
                criteria={"path": "~/bootcamp/test_copy.txt", "type": "file"},
                hint="Use `cp` to 'copy' the file: `cp test_renamed.txt test_copy.txt`",
            ),
            Objective(
                id="01_h_ls",
                type="command_run",
                description="List the contents of the directory.",
                criteria={"command": "ls"},
                hint="Use `ls` to 'list' the contents of the directory: `ls`",
            ),
            Objective(
                id="01_i_rm",
                type="path_not_exists",
                description="Remove 'test_copy.txt'",
                criteria={"path": "~/bootcamp/test_copy.txt"},
                hint="Let's clean up. `rm` will 'remove' the copy: `rm test_copy.txt`",
                fail_type="path_not_exists",
                fail_criteria={"path": "~/bootcamp/test_renamed.txt"},
            ),
        ]
    )

    def on_quest_start(self):
        speech = [
            "Oh, hi there! I've never seen you around here before. You must be new to this system...",
            "Well, welcome to Bash! This is the command line, where you have [italic]complete[/italic] control over your system.",
            "You type in 'commands' to perform actions on the command line.",
            "In fact, to let you get some hands on experience now, let's run through a Bash Bootcamp!",
            "Let's start with who you are. To see who you're logged in as, you can use the 'whoami' command. Try it now!",
            "Type 'whoami' into the command line and press 'Enter' now.",
        ]
        dialogue.say_speech(speech, character="cypher")

    def on_objective_complete(self, completed_id: str, obj: Objective):
        if completed_id == "01_a_whoami":
            speech = [
                "That's you! Sometimes it helps to just know who you're logged in as on a system...",
                "Now it's time to find out [italic]where[/italic] we are...",
                "Try running the 'pwd' command now.",
            ]
            dialogue.say_speech(speech, character="cypher")

        elif completed_id == "01_b_pwd":
            user = os.getlogin()
            speech = [
                f"Good. `/home/{user}/`. That's your home directory. Your corner of the system. Its nickname is `~`, by the way...",
                "Now, let's make our own directory to play around in.",
                "Try using 'mkdir' now to make a directory named 'bootcamp'.",
            ]
            dialogue.say_speech(speech, character="cypher")

        elif completed_id == "01_c_mkdir":
            speech = [
                "Nice. A little sandbox for us to play in.",
                "To 'change directory', use the `cd` command. Try it: `cd bootcamp`",
            ]
            dialogue.say_speech(speech, character="cypher")
            self._tracked_dirs.add(os.path.expanduser("~/bootcamp"))

        elif completed_id == "01_d_cd":
            speech = [
                "You're in the directory now. See? The prompt changed.",
                "Before we continue, let me make sure you understand how the 'path' system works in Bash.",
                "Alright, let's make our very first file. Try `touch test.txt`.",
            ]
            dialogue.say_speech(speech, character="cypher")

        elif completed_id == "01_e_touch":
            speech = [
                "Perfect. You've created a file. Files made with 'touch' are empty.",
                "But let's say we accidentally gave it the wrong name. We need to rename it!",
                "In Bash, you `mv` (move) files to rename them. Try `mv test.txt test_renamed.txt`",
            ]
            dialogue.say_speech(speech, character="cypher")
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test.txt"))

        elif completed_id == "01_f_mv":
            speech = [
                "See? `mv` is a 2-for-1. Now, let's make a backup.",
                "Copy 'test_renamed.txt' and name it 'test_copy.txt'. Try it now!",
            ]
            dialogue.say_speech(speech, character="cypher")
            self._tracked_files.remove(os.path.expanduser("~/bootcamp/test.txt"))
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test_renamed.txt"))

        elif completed_id == "01_g_cp":
            dialogue.say(
                "Great. Now you have two files. Let's take a look at them with `ls`.",
                character="cypher",
            )
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test_copy.txt"))

        elif completed_id == "01_h_ls":
            speech = [
                "Fantastic! You're starting to get better at this.",
                "Let's clean up. `rm` will 'remove' the *copy*. Be careful, there's no undo!",
                "Run `rm test_copy.txt`",
            ]
            dialogue.say_speech(speech, character="cypher")

        elif completed_id == "01_i_rm":
            dialogue.say("And it's gone. *Poof*... See? Permanent.", character="cypher")
            self._tracked_files.remove(os.path.expanduser("~/bootcamp/test_copy.txt"))

    def on_objective_failure(self, command_result: CommandResult):
        active_obj = quest_manager.get_active_objective()
        if not active_obj:
            return

        if active_obj.id == "01_i_rm":
            command = command_result.command
            if "rm" in command and "test_renamed.txt" in command:
                speech = [
                    "Whoa! Good try, but that's the *original* file.",
                    "We're trying to remove `test_copy.txt`. Leave the original alone!",
                ]
                dialogue.say_speech(speech, character="cypher")
            else:
                dialogue.say(
                    f"Not quite. Try this: {active_obj.hint}", character="cypher"
                )
        else:
            if command_result.return_code == 0:
                dialogue.say(
                    f"That's not it. Remember: {active_obj.hint}", character="cypher"
                )

    def sync_world_state(self, completed_ids: set[str]):
        log.info(f"Syncing world state for {self.id}...")

        if "01_c_mkdir" in completed_ids:
            self._spawn_dir("~/bootcamp")
        if "01_e_touch" in completed_ids:
            if "01_f_mv" not in completed_ids:
                self._spawn_file("~/bootcamp/test.txt")
        if "01_f_mv" in completed_ids:
            if "01_i_rm" not in completed_ids:
                self._spawn_file("~/bootcamp/test_renamed.txt")
        if "01_g_cp" in completed_ids:
            if "01_i_rm" not in completed_ids:
                self._spawn_file("~/bootcamp/test_copy.txt")
