import logging
import os

from supershell.game.base_quest import BaseQuest
from supershell.game.models import Objective
from supershell.tui import dialogue

log = logging.getLogger(__name__)


class Quest(BaseQuest):
    id = "quest_01_boot_camp"
    title = "Boot Camp"
    description = "Time for your first lesson. We'll learn the basics: how to look, move, and build."

    objectives = [
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
            type="path_exists",
            description="",
            criteria={"path": "~/bootcamp/test_renamed.txt", "type": "file"},
            hint="Let's rename it. `mv` is for 'move', but it's also for renaming: `mv test.txt test_renamed.txt`",
        ),
        Objective(
            id="01_g_cp",
            type="path_exists",
            description="",
            criteria={"path": "~/bootcamp/test_copy.txt", "type": "file"},
            hint="Use `cp` to 'copy' the file: `cp test_renamed.txt test_copy.txt`",
        ),
        Objective(
            id="01_h_rm",
            type="path_not_exists",
            description="",
            criteria={"path": "~/bootcamp/test_renamed.txt"},
            hint="Let's clean up. `rm` will 'remove' the original: `rm test_renamed.txt`",
        ),
    ]

    def on_quest_start(self):
        dialogue.say(
            "Oh, hi there! I've never seen you around here before. You must be new to this system..."
        )
        dialogue.say(
            "Well, welcome to Bash! This is the command line, where you have [italic]complete[/italic] control."
        )
        dialogue.say("You type in 'commands' to perform actions on the command line.")
        dialogue.say(
            "In fact, to let you get some hands on experience now, let's run through a Bash Bootcamp!"
        )
        dialogue.say(
            "Let's start with who you are. To see who you're logged in as, you can use the 'whoami' command."
        )
        dialogue.say(
            "Try it now! Type 'whoami' into the command line and press 'Enter' now."
        )

    def on_objective_complete(self, completed_id: str, obj: Objective):
        # Track our created files for the main cleanup function
        if completed_id == "01_a_whoami":
            dialogue.say(
                "That's you! Sometimes it helps to just know who you're logged in as on a system."
            )
            dialogue.say("Now it's time to find out *where* we are...")
            dialogue.say(
                "Do you ever get to doing something and then completely forget where you are?..."
            )
            dialogue.say(
                "Yeah, me neither. But if you ever get lost, it never hurts to find your"
            )
            dialogue.say("'Present Working Directory'.")
        elif completed_id == "01_b_pwd":
            dialogue.say(
                "Good. `~`. That's your home directory. Our little corner of the system."
            )
            dialogue.say("Now, let's make our own directory to play around in.")
            dialogue.say("To make a directory, we just type in 'mkdir', a space, and")
            dialogue.say("then the name of the directory we want to create. Try using")
            dialogue.say("'mkdir' now to make a directory named 'bootcamp'.")
        elif completed_id == "01_c_mkdir":
            dialogue.say("Nice. A little sandbox for us to play in.")
            self._tracked_dirs.add(os.path.expanduser("~/bootcamp"))
        elif completed_id == "01_d_cd":
            dialogue.say(
                "You're in the sandbox. See? The prompt changed. Now for files."
            )
        elif completed_id == "01_e_touch":
            dialogue.say("Perfect. You've created a file. You can check it with `ls`.")
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test.txt"))
        elif completed_id == "01_f_mv":
            dialogue.say("See? `mv` is a 2-for-1. Now, let's make a backup.")
            self._tracked_files.remove(os.path.expanduser("~/bootcamp/test.txt"))
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test_renamed.txt"))
        elif completed_id == "01_g_cp":
            dialogue.say("Great. Now you have two files. We're making a mess.")
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test_copy.txt"))
        elif completed_id == "01_h_rm":
            dialogue.say(
                "And it's gone. *Poof*. Be careful with `rm`, it doesn't ask twice."
            )
            self._tracked_files.remove(
                os.path.expanduser("~/bootcamp/test_renamed.txt")
            )


def sync_world_state(self, completed_ids: set[str]):
    """
    Re-spawns files for this quest based on saved progress.
    """
    log.info(f"Syncing world state for {self.id}...")

    # If 'mkdir' is done, the 'safehouse' should exist.
    if "01_c_mkdir" in completed_ids:
        self._spawn_dir("~/bootcamp")

    # If 'touch' is done...
    if "01_e_touch" in completed_ids:
        # ...but 'mv' is NOT...
        if "01_f_mv" not in completed_ids:
            # ...then 'test.txt' should be in the bootcamp dir.
            self._spawn_file("~/bootcamp/test.txt")

    # If 'mv' is done...
    if "01_f_mv" in completed_ids:
        # ...but 'rm' is NOT...
        if "01_h_rm" not in completed_ids:
            # ...then 'test_renamed.txt' should be there.
            self._spawn_file("~/bootcamp/test_renamed.txt")

    # If 'cp' is done...
    if "01_g_cp" in completed_ids:
        # ...then 'test_copy.txt' should be there.
        self._spawn_file("~/bootcamp/test_copy.txt")
