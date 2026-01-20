use crossterm::{
    cursor::MoveToColumn,
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::env;
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;
use textwrap::fill;

// --- CONSTANTS ---
// We use simple white on black for maximum readability and high contrast.
const BORDER_COLOR: Color = Color::White;
const TITLE_COLOR: Color = Color::White;
const TEXT_COLOR: Color = Color::White;

// --- HELPER: DRAW BORDERS ---

/// Draws the top border with an embedded title.
/// Example: ┌── MISSION BRIEF ───────────┐
fn print_top_border(width: u16, title: &str) {
    let mut stdout = stdout();

    // 1. Draw the top-left corner and the start of the line
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
    print!("┌──");

    // 2. Draw the Title (Bold)
    execute!(
        stdout,
        SetForegroundColor(TITLE_COLOR),
        SetAttribute(Attribute::Bold)
    )
    .unwrap();
    print!(" {} ", title);

    // 3. Draw the rest of the dashed line
    execute!(
        stdout,
        SetForegroundColor(BORDER_COLOR),
        SetAttribute(Attribute::Reset)
    )
    .unwrap();

    let safe_width = width as usize;
    // Calculate how much space the "┌── TITLE " took up
    let used_len = 3 + 1 + title.chars().count() + 1;
    let remaining = if safe_width > used_len {
        safe_width - used_len
    } else {
        0
    };

    print!("{}", "─".repeat(remaining));

    // 4. Force cursor to the exact right edge.
    // We do this to correct for "Emoji Width" bugs where icons like 📂
    // are calculated as 1 char width by the code but render as 2 chars wide,
    // which would otherwise push the corner off-alignment.
    execute!(stdout, MoveToColumn(width - 1)).unwrap();

    // In Raw Mode, standard \n only moves down; we need \r to reset to left.
    print!("┐\r\n");
}

/// Draws the bottom border.
/// Example: └────────────────────────────┘
fn print_bottom_border(width: u16) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();

    // Draw the line
    print!("└{}", "─".repeat((width as usize).saturating_sub(2)));

    // Force alignment for the bottom-right corner
    execute!(stdout, MoveToColumn(width - 1)).unwrap();
    print!("┘\r\n");

    execute!(stdout, ResetColor).unwrap();
}

// --- RENDER ENGINE ---

/// The core rendering function.
/// - Handles Raw Mode (to capture Enter key without echoing).
/// - Handles Text Wrapping.
/// - Handles the Typewriter/Skip logic.
fn render_inline_card(title: &str, lines: Vec<String>, use_typewriter: bool) {
    // If running in automated tests, skip visual effects
    if env::var("SUPERSHELL_TEST_MODE").is_ok() {
        println!("[{}]", title);
        for line in &lines {
            println!("{}", line);
        }
        return;
    }

    // 1. Enable Raw Mode
    // This allows us to intercept the 'Enter' key instantly to skip the typing animation,
    // and prevents the user's keystrokes from appearing on screen during the cutscene.
    enable_raw_mode().unwrap();

    let mut stdout = stdout();
    let (term_cols, _) = size().unwrap_or((80, 24));

    // Calculate responsive width (Max 80 chars, but shrinks for small screens)
    let width = std::cmp::min(std::cmp::max(term_cols, 20), 80);
    let content_width = (width as usize).saturating_sub(4); // -2 border, -2 padding

    // 2. Pre-process Text (Wrapping)
    let mut final_lines = Vec::new();
    for raw_line in lines {
        if raw_line.is_empty() {
            final_lines.push("".to_string());
        } else {
            let wrapped = fill(&raw_line, content_width);
            for subline in wrapped.split('\n') {
                final_lines.push(subline.to_string());
            }
        }
    }

    // 3. Draw Top Border
    print_top_border(width, title);

    let mut skipped = false;

    // 4. Draw Content Loop
    for line in final_lines {
        // Draw Left Border
        execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
        print!("│ ");

        // Draw Text Content
        execute!(stdout, SetForegroundColor(TEXT_COLOR)).unwrap();

        if use_typewriter {
            for char in line.chars() {
                print!("{}", char);
                stdout.flush().unwrap();

                // If the user hasn't skipped yet, check for input
                if !skipped {
                    // poll checks if a key is available immediately (0 wait time)
                    if poll(Duration::from_secs(0)).unwrap() {
                        if let Event::Key(key) = read().unwrap() {
                            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                                // User pressed Enter: Enable Skip Mode
                                skipped = true;
                            }
                        }
                    }

                    // Only sleep if we are NOT skipping
                    if !skipped {
                        thread::sleep(Duration::from_millis(15));
                    }
                }
                // If skipped is true, the loop continues instantly without sleep
            }
        } else {
            // Instant render (no animation)
            print!("{}", line);
        }

        // Draw Right Border
        execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
        execute!(stdout, MoveToColumn(width - 1)).unwrap();
        print!("│\r\n");
    }

    // 5. Draw Bottom Border
    print_bottom_border(width);

    // 6. Handle Interaction (Wait for user to dismiss cutscenes)
    if use_typewriter {
        print!("\r\n>> PRESS [ENTER] TO CONTINUE...");
        stdout.flush().unwrap();

        // Blocking loop: Wait until Enter is pressed
        loop {
            if let Event::Key(key) = read().unwrap() {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                    break;
                }
            }
        }
        // Double newline to create space after the user presses Enter
        print!("\r\n\r\n");
    } else {
        // For Status Cards, add one newline so the shell prompt
        // doesn't appear glued to the bottom of the box.
        print!("\r\n");
    }

    // 7. Cleanup: Restore terminal to normal behavior
    disable_raw_mode().unwrap();
}

// --- PUBLIC API ---

/// Displays a dramatic, animated "Mission Brief" style message.
/// Usage: Use for story moments or chapter endings.
pub fn play_cutscene(text: &str) {
    render_inline_card("MISSION BRIEF", vec![text.to_string()], true);
}

/// Displays an animated Chapter intro with a loading effect.
/// Usage: Use when the user enters a new Chapter.
pub fn play_chapter_intro(chapter_title: &str, description: &str) {
    let content = vec![
        format!("DATA LOADING..."),
        String::from(""),
        format!("{}", description),
    ];
    render_inline_card(chapter_title, content, true);
}

/// Displays a static, instant status card.
/// Usage: Use for frequent feedback (like completing a task).
pub fn draw_status_card(
    title: &str,
    chapter: &str,
    instruction: &str,
    objective: &str,
    current: usize,
    total: usize,
) {
    let content = vec![
        format!("📂 {}", chapter),
        String::from(""),
        format!("🎯 OBJECTIVE: {}", objective),
        String::from(""),
        format!("💡 HINT: {}", instruction),
        String::from(""),
        format!("[Progress: {}/{}]", current, total),
    ];

    // Note: use_typewriter is false here for instant feedback
    render_inline_card(title, content, false);
}
