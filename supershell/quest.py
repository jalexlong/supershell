from typing import Dict, Any, Tuple
from .auditors import AUDIT_FUNCTIONS # Import the map of functions

class Quest:
    def __init__(self, quest_data: Dict[str, Any]):
        """
        Initializes a Quest object from a dictionary loaded from config/quests.json.
        """
        self.quest_id = quest_data["quest_id"]
        self.objective = quest_data["objective"]
        # Parse the declarative checks into callable functions with arguments
        self.prerequisite_checks = self._parse_checks(quest_data.get("prerequisite_checks", []))
        self.completion_checks = self._parse_checks(quest_data["completion_checks"])

    def _parse_checks(self, check_list: list) -> list:
        """
        Converts the JSON check structure into a list of 
        (callable_function, arguments, expected_value, failure_feedback) tuples.
        """
        parsed_checks = []
        for check in check_list:
            func_name = check["function"]
            
            if func_name not in AUDIT_FUNCTIONS:
                raise ValueError(f"Unknown audit function in config: {func_name}")

            # Get the actual callable function
            check_function = AUDIT_FUNCTIONS[func_name]
            
            # Extract arguments, expected value, and feedback
            args = check["args"]
            expected = check["expected"]
            feedback = check["feedback"]
            
            parsed_checks.append((check_function, args, expected, feedback))
        
        return parsed_checks

    def is_complete(self) -> Tuple[bool, str]:
        """Runs all completion checks to see if the quest is finished."""
        for check_function, args, expected_value, failure_feedback in self.completion_checks:
            # Call the specific auditor function with the specified arguments
            current_value = check_function(**args) 
            
            if current_value != expected_value:
                # Returns the failure feedback immediately upon finding a failed check
                return False, failure_feedback
        
        return True, f"âœ… Mission {self.quest_id} Complete! System Secured."

    def get_prerequisites_met(self) -> Tuple[bool, str]:
        """Checks if the environment is ready for the quest to begin."""
        for check_function, args, expected_value, failure_feedback in self.prerequisite_checks:
            if check_function(**args) != expected_value:
                return False, failure_feedback
        return True, ""

