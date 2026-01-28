// ui.rs

use crate::content::Objective;
use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp},
    event::{Event, KeyCode, read},
    execute,
    style::{Attribute, Color, SetAttribute, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;

// --- CONFIGURATION ---
const WIDTH: u16 = 80;
const BORDER_COLOR: Color = Color::White;
const TITLE_COLOR: Color = Color::White;

// Lower numbers = Faster animation
const FRAME_DELAY: u64 = 15;
const TEXT_DELAY: u64 = 30;

/// Renders the animated HUD box
pub fn render_mission_hud(
    mission_title: &str,
    objective: &Objective,
    step_current: usize,
    step_total: usize,
) {
    let mut stdout = stdout();

    // 1. Prepare Content
    // We format strings first to know the height of the box
    let content = vec![
        "".to_string(), // Spacer
        format!("🎯 OBJECTIVE: {}", objective.title),
        "".to_string(), // Spacer
        format!("💡 HINT: {}", objective.description),
        "".to_string(), // Spacer
        format!("[Progress: {}/{}]", step_current, step_total),
    ];

    let box_height = content.len();

    // 2. PHASE 1: Draw the Wireframe (Empty Box)
    let header = format!("Module: {}", mission_title);
    print_top_border(WIDTH, &header);

    for _ in 0..box_height {
        execute!(stdout, SetForegroundColor(Color::DarkGrey)).unwrap();
        print!("│");
        print!("{}", " ".repeat((WIDTH - 2) as usize));
        print!("│\r\n");
        // Animation delay for "scanning" effect
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(FRAME_DELAY));
    }
    print_bottom_border(WIDTH);

    // 3. PHASE 2: Data Injection
    // Jump cursor back UP inside the box
    let jump_height = (box_height + 1) as u16;
    execute!(stdout, MoveUp(jump_height)).unwrap();

    // Print text line by line
    // NOTE: iterate over &content so we don't consume the vector
    for line in &content {
        execute!(stdout, MoveDown(1), MoveToColumn(3)).unwrap();
        execute!(
            stdout,
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold)
        )
        .unwrap();
        print!("{}", line);

        // Typing delay effect
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(TEXT_DELAY));
    }

    // 4. Cleanup
    // Move cursor back to bottom so user can type
    let distance_to_bottom = (box_height - content.len() + 1) as u16;
    execute!(stdout, MoveDown(distance_to_bottom)).unwrap();
    execute!(stdout, SetAttribute(Attribute::Reset)).unwrap();
    println!(); // Final newline
}

/// Plays a "Cutscene" (Static text that waits for Enter)
pub fn play_cutscene(text: &str) {
    let _stdout = stdout();

    // Enable raw mode to catch single keystrokes
    enable_raw_mode().unwrap();

    let header = "MISSION BRIEF";
    print_top_border(WIDTH, header);

    for line in text.lines() {
        print!("│ {:<76} │\r\n", line);
    }
    print_bottom_border(WIDTH);

    println!("\r\n>> PRESS [ENTER] TO CONTINUE...");

    // Loop until user hits Enter
    loop {
        if let Event::Key(event) = read().unwrap() {
            if event.code == KeyCode::Enter {
                break;
            }
        }
    }
    // CRITICAL: Always disable raw mode!
    disable_raw_mode().unwrap();
}

// --- PRIVATE HELPER FUNCTIONS ---

fn print_top_border(width: u16, title: &str) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
    print!("┌──");

    execute!(
        stdout,
        SetForegroundColor(TITLE_COLOR),
        SetAttribute(Attribute::Bold)
    )
    .unwrap();
    print!(" {} ", title);

    execute!(
        stdout,
        SetForegroundColor(BORDER_COLOR),
        SetAttribute(Attribute::Reset)
    )
    .unwrap();

    let used_len = 3 + 1 + title.len() + 1;
    let remaining = (width as usize).saturating_sub(used_len + 1);

    print!("{}", "─".repeat(remaining));
    print!("┐\r\n");
}

fn print_bottom_border(width: u16) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
    print!("└{}┘\r\n", "─".repeat((width - 2) as usize));
}
