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
            id="01_h_ls",
            type="path_exists",
            description="",
            criteria={"path": "~/bootcamp/test_copy.txt", "type": "file"},
            hint="Use `ls` to 'list' the contents of the directory: `ls`",
        ),
        Objective(
            id="01_i_rm",
            type="path_not_exists",
            description="",
            criteria={"path": "~/bootcamp/test_renamed.txt"},
            hint="Let's clean up. `rm` will 'remove' the original: `rm test_renamed.txt`",
        ),
    ]

    def on_quest_start(self):
        dialogue.say("Oh, hi there! I've never seen you around here before. You must")
        dialogue.say("be new to this system...")
        dialogue.say("Well, welcome to Bash! This is the command line, where you have")
        dialogue.say("[italic]complete[/italic] control over your system.")
        dialogue.say("You type in 'commands' to perform actions on the command line.")
        dialogue.say("In fact, to let you get some hands on experience now, let's run")
        dialogue.say("through a Bash Bootcamp!")
        dialogue.say("Let's start with who you are. To see who you're logged in as,")
        dialogue.say("you can use the 'whoami' command. Try it now! Type 'whoami' into")
        dialogue.say("the command line and press 'Enter' now.")

    def on_objective_complete(self, completed_id: str, obj: Objective):
        # Track our created files for the main cleanup function
        if completed_id == "01_a_whoami":
            dialogue.say("That's you! Sometimes it helps to just know who you're")
            dialogue.say("logged in as on a system...")
            dialogue.say("Now it's time to find out [italic]where[/italic] we are...")
            dialogue.say("Do you ever get to doing something and then completely")
            dialogue.say("forget where you are?...")
            dialogue.say("Yeah, me neither. But if you ever get lost, it never hurts")
            dialogue.say("to find your 'Present Working Directory', or 'pwd' for")
            dialogue.say("short. Try running the 'pwd' command now.")

        elif completed_id == "01_b_pwd":
            user = os.getlogin()
            dialogue.say(f"Good. `/home/{user}/`. That's your home directory. Your")
            dialogue.say("corner of the system. This directory holds all of your")
            dialogue.say("personal files and directories. Its nickname is '~', btw...")
            dialogue.say("Now, let's make our own directory to play around in.")
            dialogue.say("To make a directory, we just type in 'mkdir', a space, and")
            dialogue.say("then the name of the directory we want to create. Try using")
            dialogue.say("'mkdir' now to make a directory named 'bootcamp'.")

        elif completed_id == "01_c_mkdir":
            dialogue.say("Nice. A little sandbox for us to play in.")
            dialogue.say("Okay, so now that the directory is made, we need to actually")
            dialogue.say("move inside of it. Think of directories like a building for")
            dialogue.say("other files and folders.")
            self._tracked_dirs.add(os.path.expanduser("~/bootcamp"))

        elif completed_id == "01_d_cd":
            user = os.getlogin()
            dialogue.say("You're in the directory now. Nice. When the prompt shows")
            dialogue.say("up, you will see that you're in the bootcamp directory...")
            dialogue.say("Before we continue, let me make sure you understand")
            dialogue.say("how the 'path' system works in Bash.")
            dialogue.say(f"Right now, you're in '/home/{user}/bootcamp/'...")
            dialogue.say(f"But there's another way to refer to the 'home/{user}/'")
            dialogue.say("directory. You can use '~' to refer to your home directory.")
            dialogue.say("That means that '~/bootcamp/' is the same as")
            dialogue.say(f"'/home/{user}/bootcamp/'...")
            dialogue.say(f"So basically, '~' is the same as '/home/{user}/'")
            dialogue.say("Another thing to note is that directories end with a '/'")
            dialogue.say("so you can distinguish them from files...")
            dialogue.say("That way, you can easily navigate your file system without")
            dialogue.say("wondering what is a file or what is a folder...")
            dialogue.say("Alright, now it's time to make our very first file.")
            dialogue.say("To make a file, we just type in 'touch', a space, and")
            dialogue.say("then the name of the file you want to create. Try using")
            dialogue.say("the name 'test.txt'.")

        elif completed_id == "01_e_touch":
            dialogue.say("Perfect. You've created a file. Files made with 'touch' are")
            dialogue.say("empty, so don't expect them to have anything inside them...")
            dialogue.say("But let's say that we accidentally gave 'test.txt' the")
            dialogue.say("wrong name. We need to rename it! To do that in Bash, you")
            dialogue.say("just use the 'mv' command. Normally, the 'mv' command is")
            dialogue.say("used to *move* files, but you can also use it to *rename*")
            dialogue.say("files instead. Try to rename the 'test.txt' file to")
            dialogue.say("'test_renamed.txt'...")
            dialogue.say("By the way, if you ever need help or get stuck, you can")
            dialogue.say("always just ask for my help by running the 'cypher' command.")
            dialogue.say("Okay, now let's rename that file!")
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test.txt"))

        elif completed_id == "01_f_mv":
            dialogue.say("See? `mv` is a 2-for-1. Now, let's make a backup.")
            dialogue.say("It's always good to have an extra copy of important")
            dialogue.say("files, so let's practice on our 'test_renamed.txt' file.")
            dialogue.say("To make a copy of a file or directory, just type 'cp' and")
            dialogue.say("then the name of the file/directory to copy, and then where")
            dialogue.say("you want to store that copy...")
            dialogue.say("For example, 'cp file.txt backup_file.txt' will copy")
            dialogue.say("a file named 'file.txt' and name that copy")
            dialogue.say("'backup_file.txt'...")
            dialogue.say("I want you to copy your 'test_renamed.txt' file and name")
            dialogue.say("it 'test_copy.txt'. Try it now!")
            self._tracked_files.remove(os.path.expanduser("~/bootcamp/test.txt"))
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test_renamed.txt"))

        elif completed_id == "01_g_cp":
            dialogue.say("Great. Now you have two files. Let's take a look at them")
            dialogue.say("with 'ls' to make sure our files look just like they're")
            dialogue.say("supposed to...")
            self._tracked_files.add(os.path.expanduser("~/bootcamp/test_copy.txt"))

        elif completed_id == "01_h_ls":
            dialogue.say("Fantastic! You're starting to get better at the command")
            dialogue.say("line...")
        elif completed_id == "01_i_rm":
            dialogue.say("And it's gone. *Poof*...")
            dialogue.say("Be careful with 'rm'. It doesn't ask twice...")
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
