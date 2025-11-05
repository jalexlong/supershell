"""
This file contains all content for the "First Contact" quest.
"""
import os
import logging
from supershell.game.models import Objective
from supershell.game.base_quest import BaseQuest
from supershell.tui import dialogue
from supershell.game import quest_manager

log = logging.getLogger(__name__)

def hunter_broadcast(message: str):
    """Prints a message from the 'auditd' Hunter."""
    console = dialogue.get_console() 
    console.print(
        f"\n[hunter]Broadcast message from root@The_Host (PID 1024 - 'auditd'):[/hunter]\n"
        f"[hunter]> {message}[/hunter]",
        highlight=False
    )

class Quest(BaseQuest):
    id = "00_first_contact"
    title = "First Contact"
    description = "Whoa... a new process? You're not one of theirs. How did you get in here? Do you even know where you are? Let's start with the basics. Where *are* we? Find your 'present working directory'."
    
    objectives = [
        Objective(
            id="00_obj_a_pwd",
            description="Run the `pwd` command to see your current directory.",
            type="command_run",
            criteria={"command": "pwd"},
            hint="Just type `pwd` and press Enter.",
            success_message="Okay, We're in `~`... This is your home directory. Good. At least we're not in `/var/spool` or something... ugh, *spool*."
        ),
        Objective(
            id="00_obj_b_ls",
            description="List the files in your home directory.",
            type="command_run",
            criteria={"command": "ls"},
            hint="Type `ls` to 'list' files.",
            success_message="Empty. Clean. A blank slate. I *like* blank slates. So much room for... creative organization."
        ),
        Objective(
            id="00_obj_c_mkdir",
            description="Create a 'safehouse' directory for your files.",
            type="path_exists",
            criteria={"path": "~/safehouse", "type": "dir"},
            hint="Use the 'make directory' command: `mkdir safehouse`",
            success_message="Excellent. A cozy little hidey-hole. You'll need this."
        ),
        Objective(
            id="00_obj_d_spawn_file",
            description="I've created a `test_file.txt` for you. Run `ls` to see it.",
            type="command_run",
            criteria={"command": "ls"},
            hint="Run `ls` again.",
            success_message="See it? Good. Time for a magic trick. Make it *disappear*... into the `safehouse`, of course."
        ),
        Objective(
            id="00_obj_e_mv_file",
            description="Move the `test_file.txt` into your `safehouse` directory.",
            type="path_exists",
            criteria={"path": "~/safehouse/test_file.txt", "type": "file"},
            hint="Use the 'move' command: `mv test_file.txt safehouse/`",
            success_message="Perfect. You're a fast learner. Good. Because... hmm. We need to talk."
        )
    ]

    def handle_event(self, completed_id: str, obj: Objective):
        
        # --- Run the *default* action (print success message) ---
        # We call 'super()' to run the code from BaseQuest
        super().handle_event(completed_id, obj) 
        
        # --- Handle our custom "action" events ---
        if completed_id == "00_obj_c_mkdir":
            # Manually add the player-created dir to our tracking list
            self._tracked_dirs.add(os.path.expanduser("~/safehouse"))
            # Spawn the quest file
            self._spawn_file("~/test_file.txt", "This is just a test.\n")

        elif completed_id == "00_obj_e_mv_file":
            dialogue.say("Okay... now that you have the basics, we need to talk.", character="cypher")
            dialogue.say("You know how you're not... *supposed* to exist? Well, the system's janitor, the **Hunter**, *really* doesn't like things that aren't supposed to exist.", character="cypher")
            self._spawn_file("~/CORE_SIGNATURE.dat", "PROCESS_MANIFEST::ANOMALY_0x8A3F\n")
            dialogue.say("This `CORE_SIGNATURE.dat` file the Hunter just made? That's your 'ankle bracelet.'... If the Hunter finds it here, you're going to go *poof*.", character="cypher")

    def sync_world_state(self, completed_ids: set[str]):
        """
        Re-spawns files for the quest based on the 
        player's saved progress.
        """
        log.info(f"Syncing world state for {self.id}...")

        # If 'mkdir' is done, the 'safehouse' should exist.
        if "00_obj_c_mkdir" in completed_ids:
            self._spawn_dir("~/safehouse")

        # If 'spawn_file' (the 'ls' objective) is done...
        if "00_obj_d_spawn_file" in completed_ids:
            # ...but the 'mv' objective is NOT...
            if "00_obj_e_mv_file" not in completed_ids:
                # ...then 'test_file.txt' should be in the HOME dir.
                self._spawn_file("~/test_file.txt", "This is just a test.\n")

        # If the 'mv' objective IS complete...
        if "00_obj_e_mv_file" in completed_ids:
            # ...then 'test_file.txt' should be in the SAFEHOUSE.
            self._spawn_file("~/safehouse/test_file.txt", "This is just a test.\n")
            # AND the 'CORE_SIGNATURE.dat' file should exist
            # (since it's spawned in the same event).
            self._spawn_file("~/CORE_SIGNATURE.dat", "PROCESS_MANIFEST::ANOMALY_0x8A3F\n")

