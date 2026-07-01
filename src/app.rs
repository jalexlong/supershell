use crate::engine::{
    Progression, advance_progress, apply_rewards, is_command_relevant, validate_task_logic,
};
use crate::quest::{Course, Library};
use crate::state::GameState;
use crate::ui;
use crate::world::WorldEngine;
use std::path::{Path, PathBuf};

// --- OUTCOME TYPE ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckCommandOutcome {
    NoChange,
    LogicFailure,
    RefreshUi,
}

impl CheckCommandOutcome {
    pub fn exit_code(self) -> i32 {
        match self {
            CheckCommandOutcome::NoChange => 0,
            CheckCommandOutcome::LogicFailure => 1,
            CheckCommandOutcome::RefreshUi => 2,
        }
    }
}

// --- HELPERS ---

pub fn resolve_course_path(game: &GameState, lib: &Library) -> Option<PathBuf> {
    if !game.current_course.is_empty() {
        let path = lib.root_dir.join(&game.current_course);
        if path.exists() {
            return Some(path);
        }
    }
    let intro = lib.root_dir.join("intro.yaml");
    if intro.exists() {
        return Some(intro);
    }
    None
}

pub fn reset_game(save_path: &Path) -> GameState {
    if save_path.exists() {
        std::fs::remove_file(save_path).expect("Failed to delete save file");
    }
    println!(">> [SYSTEM] Save Data Wiped.");
    GameState::new()
}

pub fn save_game_state(game: &GameState, save_path: &Path) -> bool {
    match game.save(save_path.to_str().unwrap()) {
        Ok(()) => true,
        Err(err) => {
            eprintln!(">> [WARN] Failed to save game state: {err}");
            false
        }
    }
}

// --- GAMEPLAY HANDLERS ---

pub fn perform_validation(path_str: &str) {
    let path = Path::new(path_str);
    if !path.exists() {
        println!(">> [ERROR] File not found: {}", path_str);
        return;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            println!(">> [ERROR] Could not read file: {}", e);
            return;
        }
    };

    match serde_yml::from_str::<Course>(&content) {
        Ok(course) => {
            println!(">> [SUCCESS] YAML Syntax is valid.");
            println!(">> Title:   {}", course.title);
            println!(">> Author:  {}", course.author);
            println!(">> Version: {}", course.version);
        }
        Err(e) => {
            println!(">> [FAIL] YAML Parsing Error:");
            println!("{}", e);
        }
    }
}

pub fn handle_refresh_sequence(game: &GameState, course: &Course) {
    if game.current_task_index == 0 {
        if let Some((_, chapter, _)) = course.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            ui::play_cutscene(&chapter.intro);
            print!("\x1b[2J\x1b[H");
        }
    } else {
        println!();
    }

    handle_status_display(game, course);
}

pub fn handle_status_display(game: &GameState, course: &Course) {
    if game.is_finished {
        println!(">> [SYSTEM] Quest Complete. Run 'supershell --menu' for more.");
        return;
    }

    if let Some((quest, chapter, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        ui::draw_status_card(
            &quest.title,
            &chapter.title,
            &task.instruction,
            &task.objective,
            game.current_task_index + 1,
            chapter.tasks.len(),
        );
    }
}

pub fn handle_check_command(
    user_cmd: &str,
    cwd_override: Option<&Path>,
    game: &mut GameState,
    course: &Course,
    save_path: &Path,
    world: &WorldEngine,
) -> CheckCommandOutcome {
    // Auto-restore if the Construct was destroyed (e.g. `rm -rf ~/Construct`)
    let construct_destroyed = !world.is_intact();
    if construct_destroyed {
        println!(">> [SYSTEM] Construct corruption detected. Restoring...");
        world.initialize().ok();
    }

    // Run chapter setup on the first task of a chapter or after world destruction
    if construct_destroyed || game.current_task_index == 0 {
        if let Some(quest) = course.quests.iter().find(|q| q.id == game.current_quest_id) {
            if let Some(chapter) = quest.chapters.get(game.current_chapter_index) {
                if !chapter.setup_actions.is_empty() {
                    if construct_destroyed {
                        println!(">> [SYSTEM] Reconfiguring Construct...");
                    }
                    world.build_scenario(&chapter.setup_actions);
                }
            }
        }
    }

    if let Some((quest, chapter, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        // --- PASS 1: RELEVANCE (Permissive) ---
        if !is_command_relevant(user_cmd, task, game) {
            return CheckCommandOutcome::NoChange;
        }

        // --- PASS 2: LOGIC (Strict) ---
        if let Err(msg) = validate_task_logic(user_cmd, task, game, cwd_override) {
            game.failure_count += 1;
            save_game_state(game, save_path);
            let hint = if game.failure_count >= 3 && !task.hint.is_empty() {
                task.hint.as_str()
            } else {
                ""
            };
            ui::print_fail(&msg, hint);
            return CheckCommandOutcome::LogicFailure;
        }

        // --- SUCCESS ---
        println!("\r\n");
        ui::print_success(&task.success_msg);

        apply_rewards(game, &task.rewards);
        game.failure_count = 0;

        let progression = advance_progress(game, chapter.tasks.len(), quest.chapters.len());

        match progression {
            Progression::NextTask => {
                println!("\n\x1b[0;90m[ PRESS ENTER TO CONTINUE ]\x1b[0m");
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();
            }
            Progression::NextChapter => {
                ui::play_cutscene(&chapter.outro);

                let next_chapter = &quest.chapters[game.current_chapter_index];
                if !next_chapter.setup_actions.is_empty() {
                    println!(">> [SYSTEM] Reconfiguring Construct...");
                    world.build_scenario(&next_chapter.setup_actions);
                }
            }
            Progression::ModuleComplete => {
                ui::play_cutscene(&chapter.outro);
                println!("\n\x1b[1;32m>> [SYSTEM] ALL MODULES COMPLETE. DISCONNECTING...\x1b[0m");

                save_game_state(game, save_path);

                return CheckCommandOutcome::RefreshUi;
            }
        }

        if !save_game_state(game, save_path) {
            return CheckCommandOutcome::NoChange;
        }
        return CheckCommandOutcome::RefreshUi;
    }

    CheckCommandOutcome::NoChange
}
