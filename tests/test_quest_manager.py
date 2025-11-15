import json
import os
from collections import OrderedDict
from unittest.mock import MagicMock, patch

import pytest

from supershell.game import actions, quest_manager
from supershell.game.models import Objective, Quest

# Define a dummy save file path for testing
TEST_SAVE_FILE = os.path.expanduser("~/supershell_test_save.json")


# Mock the SAVE_FILE_PATH in quest_manager to use our dummy file
@pytest.fixture(autouse=True)
def mock_save_file_path():
    with patch("supershell.game.quest_manager._SAVE_FILE_PATH", TEST_SAVE_FILE):
        yield
    # Clean up the dummy save file after each test
    if os.path.exists(TEST_SAVE_FILE):
        os.remove(TEST_SAVE_FILE)


@pytest.fixture
def mock_quests():
    # Create a simple quest structure for testing
    obj1 = Objective(id="obj1", type="any_command", criteria={}, hint="hint 1")
    obj2 = Objective(id="obj2", type="any_command", criteria={}, hint="hint 2")
    obj3 = Objective(id="obj3", type="any_command", criteria={}, hint="hint 3")

    quest1 = Quest(
        id="quest1",
        title="Test Quest 1",
        description="Desc 1",
        on_quest_start=[],
        objectives=[obj1, obj2, obj3],
        on_load_sync=[{"action": "spawn_tracked_file", "path": "~/test_file_q1.txt"}],
        on_hard_fail_script=[],
    )

    objA = Objective(id="objA", type="any_command", criteria={}, hint="hint A")
    quest2 = Quest(
        id="quest2",
        title="Test Quest 2",
        description="Desc 2",
        on_quest_start=[],
        objectives=[objA],
        on_load_sync=[{"action": "spawn_tracked_dir", "path": "~/test_dir_q2"}],
        on_hard_fail_script=[],
    )

    # Manually populate _quests in quest_manager for the test session
    with patch(
        "supershell.game.quest_manager._quests", OrderedDict()
    ) as mock_internal_quests:
        mock_internal_quests["quest1"] = quest1
        mock_internal_quests["quest2"] = quest2
        yield mock_internal_quests


@pytest.fixture
def setup_active_quest(mock_quests):
    # Simulate quest1 being active and obj1 being completed
    with (
        patch("supershell.game.quest_manager._current_quest_id", "quest1"),
        patch("supershell.game.quest_manager._active_quest_obj", mock_quests["quest1"]),
    ):
        mock_quests["quest1"].objectives[0].completed = True  # Mark obj1 as completed
        mock_quests["quest1"].objectives[1].completed = False
        mock_quests["quest1"].objectives[2].completed = False
        yield


def test_reset_current_quest_to_start(
    mock_quests, mock_save_file_path, setup_active_quest
):
    """Tests that reset_current_quest_to_start resets objective progress and re-runs on_load_sync."""
    with (
        patch("supershell.game.quest_manager._save_progress") as mock_save_progress,
        patch.object(
            mock_quests["quest1"], "_cleanup_quest_files"
        ) as mock_cleanup_files,
        patch("supershell.game.actions.run_action") as mock_run_action,
    ):
        # Ensure obj1 is initially complete as per setup_active_quest
        assert mock_quests["quest1"].objectives[0].completed

        quest_manager.reset_current_quest_to_start()

        # Assert all objectives are reset to incomplete
        for obj in mock_quests["quest1"].objectives:
            assert not obj.completed

        # Assert cleanup was called
        mock_cleanup_files.assert_called_once()

        # Assert on_load_sync actions were re-run
        # The number of calls should match the number of actions in on_load_sync
        assert mock_run_action.call_count == len(mock_quests["quest1"].on_load_sync)
        mock_run_action.assert_any_call(
            {"action": "spawn_tracked_file", "path": "~/test_file_q1.txt"}
        )

        mock_save_progress.assert_called_once()


def test_reset_current_quest_to_start_no_active_quest(mock_quests):
    """Tests that reset_current_quest_to_start handles no active quest gracefully."""
    with (
        patch(
            "supershell.game.quest_manager.get_current_quest", return_value=None
        ) as mock_get_current_quest,
        patch("supershell.game.quest_manager._save_progress") as mock_save_progress,
        patch("supershell.game.actions.run_action") as mock_run_action,
        patch("logging.Logger.warning") as mock_log_warning,
    ):
        quest_manager.reset_current_quest_to_start()

        mock_get_current_quest.assert_called_once()
        mock_log_warning.assert_called_once_with("No active quest to reset.")
        mock_save_progress.assert_not_called()
        mock_run_action.assert_not_called()


def test_load_progress_marks_objectives_correctly_initial_load(
    mock_quests, mock_save_file_path
):
    """Test that _load_progress correctly sets objective completion status on initial load."""
    obj1_id = mock_quests["quest1"].objectives[0].id
    obj2_id = mock_quests["quest1"].objectives[1].id

    # Simulate obj1 completed in save data, obj2 not
    save_data = {
        "current_quest_id": "quest1",
        "completed_objectives": [obj1_id],
    }
    with open(TEST_SAVE_FILE, "w") as f:
        json.dump(save_data, f)

    # Ensure initial state for objectives is clean before loading
    for obj in mock_quests["quest1"].objectives:
        obj.completed = False

    with (
        patch("supershell.game.quest_manager._current_quest_id", "quest1"),
        patch("supershell.game.quest_manager._active_quest_obj", mock_quests["quest1"]),
    ):
        quest_manager._load_progress()

    assert mock_quests["quest1"].objectives[0].completed  # obj1 should be complete
    assert (
        not mock_quests["quest1"].objectives[1].completed
    )  # obj2 should be incomplete
    assert (
        not mock_quests["quest1"].objectives[2].completed
    )  # obj3 should be incomplete


def test_load_progress_handles_empty_save_file(mock_quests, mock_save_file_path):
    """Tests that _load_progress handles an empty/non-existent save file gracefully."""
    if os.path.exists(TEST_SAVE_FILE):
        os.remove(TEST_SAVE_FILE)

    with patch("logging.Logger.info") as mock_log_info:
        quest_manager._load_progress()
        mock_log_info.assert_any_call("No save file found. Starting fresh.")

    for quest in mock_quests.values():
        for obj in quest.objectives:
            assert not obj.completed  # All objectives should be incomplete
