use crate::quest::{Condition, ConditionType, Reward, Task, ValidationResult};
use crate::state::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progression {
    NextTask,
    NextChapter,
    ModuleComplete,
}

pub fn is_command_relevant(user_cmd: &str, task: &Task, game: &GameState) -> bool {
    task.conditions
        .iter()
        .filter(|condition| is_command_match_condition(condition))
        .any(|condition| matches!(condition.check(user_cmd, game), ValidationResult::Valid))
}

pub fn validate_task_logic(user_cmd: &str, task: &Task, game: &GameState) -> Result<(), String> {
    for condition in &task.conditions {
        if is_command_match_condition(condition) {
            continue;
        }

        match condition.check(user_cmd, game) {
            ValidationResult::Valid => continue,
            ValidationResult::LogicError(message) => return Err(message),
            ValidationResult::SyntaxError => continue,
        }
    }

    Ok(())
}

pub fn apply_rewards(game: &mut GameState, rewards: &[Reward]) {
    for reward in rewards {
        match reward {
            Reward::SetFlag { key, value } => game.set_flag(key, *value),
            Reward::SetVar { key, value } => game.set_var(key, *value),
            Reward::AddVar { key, amount } => game.mod_var(key, *amount),
        }
    }
}

pub fn advance_progress(
    game: &mut GameState,
    chapter_task_count: usize,
    quest_chapter_count: usize,
) -> Progression {
    game.advance_task();

    if game.current_task_index < chapter_task_count {
        return Progression::NextTask;
    }

    game.advance_chapter();

    if game.current_chapter_index < quest_chapter_count {
        Progression::NextChapter
    } else {
        game.is_finished = true;
        Progression::ModuleComplete
    }
}

fn is_command_match_condition(condition: &Condition) -> bool {
    matches!(
        condition.condition_type,
        ConditionType::CommandMatches { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest::{Condition, ConditionType, Reward};

    fn command_condition(pattern: &str) -> Condition {
        Condition {
            condition_type: ConditionType::CommandMatches {
                pattern: pattern.to_string(),
            },
            failure_message: None,
        }
    }

    fn flag_condition(key: &str, failure_message: &str) -> Condition {
        Condition {
            condition_type: ConditionType::FlagIsTrue {
                key: key.to_string(),
            },
            failure_message: Some(failure_message.to_string()),
        }
    }

    fn task_with_conditions(conditions: Vec<Condition>) -> Task {
        Task {
            description: "Test task".to_string(),
            instruction: "Test instruction".to_string(),
            objective: "Test objective".to_string(),
            success_msg: "Success".to_string(),
            hint: String::new(),
            conditions,
            rewards: Vec::<Reward>::new(),
        }
    }

    #[test]
    fn command_is_relevant_when_command_condition_matches() {
        let game = GameState::new();
        let task = task_with_conditions(vec![command_condition(r"^ls(\s.*)?$")]);

        assert!(is_command_relevant("ls", &task, &game));
        assert!(is_command_relevant("ls -la", &task, &game));
    }

    #[test]
    fn command_is_not_relevant_when_command_condition_does_not_match() {
        let game = GameState::new();
        let task = task_with_conditions(vec![command_condition(r"^ls(\s.*)?$")]);

        assert!(!is_command_relevant("pwd", &task, &game));
    }

    #[test]
    fn logic_validation_returns_error_when_non_command_condition_fails() {
        let game = GameState::new();
        let task = task_with_conditions(vec![
            command_condition(r"^ls$"),
            flag_condition("scanner_enabled", "Scanner is not enabled."),
        ]);

        let result = validate_task_logic("ls", &task, &game);

        assert_eq!(result, Err("Scanner is not enabled.".to_string()));
    }

    #[test]
    fn logic_validation_passes_when_non_command_conditions_pass() {
        let mut game = GameState::new();
        game.set_flag("scanner_enabled", true);

        let task = task_with_conditions(vec![
            command_condition(r"^ls$"),
            flag_condition("scanner_enabled", "Scanner is not enabled."),
        ]);

        assert_eq!(validate_task_logic("ls", &task, &game), Ok(()));
    }
}

#[test]
fn apply_rewards_sets_flags() {
    let mut game = GameState::new();

    apply_rewards(
        &mut game,
        &[Reward::SetFlag {
            key: "scanner_enabled".to_string(),
            value: true,
        }],
    );

    assert!(game.get_flag("scanner_enabled"));
}

#[test]
fn apply_rewards_sets_and_adds_variables() {
    let mut game = GameState::new();

    apply_rewards(
        &mut game,
        &[
            Reward::SetVar {
                key: "credits".to_string(),
                value: 10,
            },
            Reward::AddVar {
                key: "credits".to_string(),
                amount: 5,
            },
        ],
    );

    assert_eq!(game.get_var("credits"), 15);
}

#[test]
fn advance_progress_moves_to_next_task_when_chapter_has_more_tasks() {
    let mut game = GameState::new();

    let progression = advance_progress(&mut game, 2, 1);

    assert_eq!(progression, Progression::NextTask);
    assert_eq!(game.current_task_index, 1);
    assert_eq!(game.current_chapter_index, 0);
    assert!(!game.is_finished);
}

#[test]
fn advance_progress_moves_to_next_chapter_when_chapter_is_complete() {
    let mut game = GameState::new();

    let progression = advance_progress(&mut game, 1, 2);

    assert_eq!(progression, Progression::NextChapter);
    assert_eq!(game.current_task_index, 0);
    assert_eq!(game.current_chapter_index, 1);
    assert!(!game.is_finished);
}

#[test]
fn advance_progress_marks_module_complete_when_final_chapter_is_complete() {
    let mut game = GameState::new();

    let progression = advance_progress(&mut game, 1, 1);

    assert_eq!(progression, Progression::ModuleComplete);
    assert_eq!(game.current_task_index, 0);
    assert_eq!(game.current_chapter_index, 1);
    assert!(game.is_finished);
}
