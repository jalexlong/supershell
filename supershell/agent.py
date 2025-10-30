import os
import subprocess
from typing import Dict
from .quest import Quest

class AgentGuide:
    def __init__(self, quests: Dict[str, Quest]):
        self.quests = quests
        self.current_quest_id = "L3Q1_SECURE_LOGS" # Hardcoded start for demo
        self.current_quest: Quest = self.quests[self.current_quest_id]
        
    def run_interaction_loop(self):
        """Runs the main game loop, displaying the prompt and auditing the system."""
        
        prereq_met, feedback = self.current_quest.get_prerequisites_met()
        if not prereq_met:
            print("--- FATAL ERROR ---")
            print(f"Mission startup failed: {feedback}")
            print("Please ensure the environment is correctly set up (e.g., run main.py once).")
            return

        print("\n--- NEW MISSION ---")
        print(f"**{self.current_quest.quest_id}**")
        print(f"**OBJECTIVE:** {self.current_quest.objective}")
        print("-------------------\n")

        while True:
            # 1. Display the custom prompt
            # Show the current working directory (e.g., 'home' if in /home/student)
            prompt_dir = os.getcwd().split('/')[-1]
            user_input = input(f"[{self.current_quest_id} @ {prompt_dir}] $ ")
            user_input = user_input.strip()

            if not user_input:
                continue
            if user_input.lower() == 'quit':
                print("\nMission aborted. Agent Guide shutting down.")
                break
            
            # 2. Execute the command in the live shell
            # This is the core interaction: the user command runs on the live system.
            try:
                # shell=True is necessary for pipes, aliases, and shell commands
                subprocess.run(user_input, shell=True, check=False)
            except Exception as e:
                print(f"SuperShell: Execution Error - {e}")
                continue

            # 3. Audit the System State
            is_complete, feedback = self.current_quest.is_complete()

            if is_complete:
                print("\n" + feedback + "\n")
                # **PROFESSIONAL EXPANSION POINT:** Add logic here to load the next quest
                # self.current_quest = self.load_next_quest(self.current_quest_id)
                break 
            else:
                # If incomplete, print the specific failure hint provided by Quest.is_complete()
                print(feedback)
                print(f"Current Status: Auditing System State. Keep trying, Agent.")

