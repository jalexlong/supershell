import logging
import os
from collections import OrderedDict
from unittest.mock import MagicMock, patch

import pytest

from supershell.game import actions
from supershell.game.models import Objective, Quest
from supershell.shell.executor import CommandResult


# Mock quest_manager for actions that depend on it
@pytest.fixture(autouse=True)
def mock_quest_manager():
    with (
        patch(
            "supershell.game.quest_manager.get_current_quest"
        ) as mock_get_current_quest,
        patch(
            "supershell.game.quest_manager.get_active_objective"
        ) as mock_get_active_objective,
    ):
        # Default mocks to return None, preventing unexpected side effects
        mock_get_current_quest.return_value = None
        mock_get_active_objective.return_value = None
        yield {
            "get_current_quest": mock_get_current_quest,
            "get_active_objective": mock_get_active_objective,
        }


@pytest.fixture
def mock_dialogue():
    with patch("supershell.tui.dialogue.say") as mock_say:
        yield mock_say


@pytest.fixture
def sample_objective_data():
    return {
        "id": "test_obj_fail",
        "type": "command_run",
        "criteria": {"command": "test_command"},
        "hint": "This is a hint.",
        "description": "Test objective for fail script.",
        "on_complete_script": [
            {"action": "say", "character": "system", "message": "Objective complete!"}
        ],
        "on_command_fail_script": [
            {
                "action": "conditional_say_on_fail",
                "character": "cypher",
                "message": "Specific fail message.",
                "if_command": "bad_command",
            },
            {
                "action": "say",
                "character": "system",
                "message": "General fail message.",
            },
        ],
    }


@pytest.fixture
def sample_hard_fail_quest():
    obj = Objective(id="dummy_obj", type="any_command", criteria={}, hint="dummy hint")
    return Quest(
        id="hard_fail_quest",
        title="Hard Fail Quest",
        description="A quest that can hard fail.",
        on_quest_start=[],
        objectives=[obj],
        on_load_sync=[],
        on_hard_fail_script=[
            {"action": "say", "character": "system", "message": "Hard fail message 1."},
            {"action": "reset_current_quest"},
        ],
    )


def test_objective_loads_on_command_fail_script(sample_objective_data):
    """
    Tests that the Objective dataclass correctly loads the on_command_fail_script.
    """
    objective = Objective.from_dict(sample_objective_data)
    assert objective.id == "test_obj_fail"
    assert len(objective.on_command_fail_script) == 2
    assert objective.on_command_fail_script[0]["action"] == "conditional_say_on_fail"
    assert objective.on_command_fail_script[1]["action"] == "say"


def test_conditional_say_on_fail_triggers_on_matching_command(mock_dialogue):
    """
    Tests that conditional_say_on_fail triggers when the command matches criteria.
    """
    command_result = CommandResult(
        command="bad_command arg1", stdout="", stderr="", return_code=1
    )
    action_data = {
        "action": "conditional_say_on_fail",
        "character": "cypher",
        "message": "Specific fail message.",
        "if_command": "bad_command",
        "command_result": command_result,
    }
    actions.run_action(action_data)
    mock_dialogue.assert_called_once_with("Specific fail message.", character="cypher")


def test_conditional_say_on_fail_triggers_on_matching_args(mock_dialogue):
    """
    Tests that conditional_say_on_fail triggers when arguments match criteria.
    """
    command_result = CommandResult(
        command="rm test_copy.txt", stdout="", stderr="", return_code=1
    )
    action_data = {
        "action": "conditional_say_on_fail",
        "character": "cypher",
        "message": "You tried to delete the copy!",
        "if_command": "rm",
        "if_args_contain": "test_copy.txt",
        "command_result": command_result,
    }
    actions.run_action(action_data)
    mock_dialogue.assert_called_once_with(
        "You tried to delete the copy!", character="cypher"
    )


def test_conditional_say_on_fail_does_not_trigger_on_non_matching_command(
    mock_dialogue,
):
    """
    Tests that conditional_say_on_fail does not trigger when command does not match.
    """
    command_result = CommandResult(
        command="another_command", stdout="", stderr="", return_code=1
    )
    action_data = {
        "action": "conditional_say_on_fail",
        "character": "cypher",
        "message": "Specific fail message.",
        "if_command": "bad_command",
        "command_result": command_result,
    }
    actions.run_action(action_data)
    mock_dialogue.assert_not_called()


def test_conditional_say_on_fail_does_not_trigger_on_non_matching_args(mock_dialogue):
    """
    Tests that conditional_say_on_fail does not trigger when arguments do not match.
    """
    command_result = CommandResult(
        command="rm wrong.txt", stdout="", stderr="", return_code=1
    )
    action_data = {
        "action": "conditional_say_on_fail",
        "character": "cypher",
        "message": "You tried to delete the copy!",
        "if_command": "rm",
        "if_args_contain": "test_copy.txt",
        "command_result": command_result,
    }
    actions.run_action(action_data)
    mock_dialogue.assert_not_called()


def test_on_command_fail_script_execution(
    mock_quest_manager, mock_dialogue, sample_objective_data
):
    """
    Simulates the game loop calling on_command_fail_script.
    """
    objective = Objective.from_dict(sample_objective_data)
    mock_quest_manager["get_active_objective"].return_value = objective

    failed_command_result = CommandResult(
        command="bad_command", stdout="", stderr="", return_code=1
    )

    # Simulate game_loop's conditional fail handling
    if objective.on_command_fail_script:
        for action_data in objective.on_command_fail_script:
            run_params = action_data.copy()
            run_params["command_result"] = failed_command_result
            actions.run_action(run_params)

    # Expect the conditional message first for "bad_command"
    mock_dialogue.assert_any_call("Specific fail message.", character="cypher")
    # Expect the general message next
    mock_dialogue.assert_any_call("General fail message.", character="system")
    assert mock_dialogue.call_count == 2


def test_on_command_fail_script_no_execution_if_no_script(
    mock_quest_manager, mock_dialogue
):
    """
    Tests that no fail script executes if the objective has none.
    """
    objective_data_no_fail = {
        "id": "no_fail_obj",
        "type": "any_command",
        "criteria": {},
        "hint": "No fail script here.",
        "on_complete_script": [],
        "on_command_fail_script": [],  # Empty script
    }
    objective = Objective.from_dict(objective_data_no_fail)
    mock_quest_manager["get_active_objective"].return_value = objective

    failed_command_result = CommandResult(
        command="some_command", stdout="", stderr="", return_code=1
    )

    if objective.on_command_fail_script:
        for action_data in objective.on_command_fail_script:
            run_params = action_data.copy()
            run_params["command_result"] = failed_command_result
            actions.run_action(run_params)

    mock_dialogue.assert_not_called()


def test_on_command_fail_script_with_non_matching_conditional(
    mock_quest_manager, mock_dialogue, sample_objective_data
):
    """
    Tests that a conditional message is skipped if criteria don't match,
    but subsequent actions still run.
    """
    objective = Objective.from_dict(sample_objective_data)
    mock_quest_manager["get_active_objective"].return_value = objective

    # This command does not match "bad_command"
    failed_command_result = CommandResult(
        command="different_command", stdout="", stderr="", return_code=1
    )

    if objective.on_command_fail_script:
        for action_data in objective.on_command_fail_script:
            run_params = action_data.copy()
            run_params["command_result"] = failed_command_result
            actions.run_action(run_params)

    # "Specific fail message" for "bad_command" should NOT be called
    mock_dialogue.assert_called_once_with("General fail message.", character="system")
    assert mock_dialogue.call_count == 1


def test_action_reset_current_quest(mock_quest_manager):
    """Tests that the reset_current_quest action calls quest_manager.reset_current_quest_to_start."""
    with patch(
        "supershell.game.quest_manager.reset_current_quest_to_start"
    ) as mock_reset:
        actions._action_reset_current_quest()
        mock_reset.assert_called_once()


def test_conditional_hard_fail_triggers_on_match(
    mock_quest_manager, mock_dialogue, sample_hard_fail_quest, caplog
):
    """Tests that conditional_hard_fail triggers and executes the on_hard_fail_script when criteria match."""
    mock_quest_manager["get_current_quest"].return_value = sample_hard_fail_quest

    command_result = CommandResult(
        command="rm -rf /", stdout="", stderr="", return_code=1
    )

    # We need to mock run_action itself to prevent recursive calls and assert its arguments
    with patch("supershell.game.actions.run_action") as mock_run_action_internal:
        with caplog.at_level(logging.ERROR):
            actions._action_conditional_hard_fail(
                command_result=command_result,
                if_command="rm",
                if_args_contain="-rf /",
                if_return_code=1,
            )

        # Assert log messages for hard fail
        assert "HARD FAIL triggered by command: rm -rf /" in caplog.text

        # The on_hard_fail_script actions should have been run
        # First action: say message
        mock_run_action_internal.assert_any_call(
            {
                "action": "say",
                "character": "system",
                "message": "Hard fail message 1.",
                "command_result": command_result,
            }
        )
        # Second action: reset_current_quest
        mock_run_action_internal.assert_any_call(
            {"action": "reset_current_quest", "command_result": command_result}
        )
        assert mock_run_action_internal.call_count == 2

    # Verify that dialogue.say was not called directly by _action_conditional_hard_fail
    # since it dispatches to run_action, which then calls dialogue.say
    mock_dialogue.assert_not_called()


def test_conditional_hard_fail_does_not_trigger_on_mismatch(
    mock_quest_manager, mock_dialogue, sample_hard_fail_quest
):
    """Tests that conditional_hard_fail does not trigger when criteria do not match."""
    mock_quest_manager["get_current_quest"].return_value = sample_hard_fail_quest

    # Command does not match
    command_result_no_match = CommandResult(
        command="ls -l", stdout="", stderr="", return_code=0
    )
    with patch("supershell.game.actions.run_action") as mock_run_action_internal:
        actions._action_conditional_hard_fail(
            command_result=command_result_no_match,
            if_command="rm",
            if_args_contain="-rf /",
            if_return_code=1,
        )
        mock_dialogue.assert_not_called()
        mock_run_action_internal.assert_not_called()

    # Args do not match
    command_result_wrong_args = CommandResult(
        command="rm test.txt", stdout="", stderr="", return_code=1
    )
    with patch("supershell.game.actions.run_action") as mock_run_action_internal:
        actions._action_conditional_hard_fail(
            command_result=command_result_wrong_args,
            if_command="rm",
            if_args_contain="-rf /",
            if_return_code=1,
        )
        mock_dialogue.assert_not_called()
        mock_run_action_internal.assert_not_called()

    # Return code does not match
    command_result_wrong_rc = CommandResult(
        command="rm -rf /", stdout="", stderr="", return_code=0
    )
    with patch("supershell.game.actions.run_action") as mock_run_action_internal:
        actions._action_conditional_hard_fail(
            command_result=command_result_wrong_rc,
            if_command="rm",
            if_args_contain="-rf /",
            if_return_code=1,
        )
        mock_dialogue.assert_not_called()
        mock_run_action_internal.assert_not_called()
