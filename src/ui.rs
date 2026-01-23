use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp},
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::env;
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;
use textwrap::fill;

// --- VISUAL CONSTANTS ---
// We use simple white on black for maximum readability and high contrast.
const BORDER_COLOR: Color = Color::White;
const TITLE_COLOR: Color = Color::White;
const TEXT_COLOR: Color = Color::White;

// We cap the width to prevent the text from stretching too wide on large monitors,
// which makes it hard to read (the "measure" or line length principle).
const MAX_WIDTH: u16 = 80;

// --- PUBLIC API ---

/// Plays a "Cutscene" style modal.
/// This renders the box, then types the text out character-by-character.
/// It pauses at the end, requiring the user to press ENTER to continue.
pub fn play_cutscene(text: &str) {
    // We wrap the text in a Vec because render_inline_card expects a list of lines.
    render_inline_card("MISSION BRIEF", vec![text.to_string()], true);
}

/// Draws the "HUD" (Heads Up Display).
/// This renders instantly (no typing effect) and does not pause the game.
pub fn draw_status_card(
    title: &str,
    chapter: &str,
    instruction: &str,
    objective: &str,
    current: usize,
    total: usize,
) {
    // Construct the list of lines to display inside the box.
    let content = vec![
        format!("📂 {}", chapter),
        String::from(""), // Empty line for visual spacing
        format!("🎯 OBJECTIVE: {}", objective),
        String::from(""),
        format!("💡 HINT: {}", instruction),
        String::from(""),
        format!("[Progress: {}/{}]", current, total),
    ];
    // Pass 'false' to use_typewriter so it draws instantly.
    render_inline_card(title, content, false);
}

/// Prints a standard Success message (green/cyan text).
pub fn print_success(msg: &str) {
    let mut stdout = stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("\r\n>> [SUCCESS] "), // \r ensures we start at the beginning of the line
        SetAttribute(Attribute::Reset),
        SetForegroundColor(Color::White),
        Print(msg),
        Print("\n\n")
    )
    .unwrap();
}

/// Prints a structured Failure message with a "System Check" style.
/// This helps the user feel like they are debugging a system, not just failing a test.
pub fn print_fail(error: &str, hint: &str) {
    let mut stdout = stdout();

    // 1. System Integrity Check (Flavor text to build immersion)
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("[+] SYSTEM_INTEGRITY.. OK\n"),
        Print("[+] SYNTAX_VALIDATION. OK\n"),
        // 2. The Error (Red for visibility)
        SetForegroundColor(Color::Red),
        Print("[-] EXECUTION......... FAIL\n"),
        Print("    └── "),
        Print(error),
        Print("\n\n"),
    )
    .unwrap();

    // 3. The Hint (Yellow is standard for warnings/tips)
    if !hint.is_empty() {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(" >> HINT: "),
            SetAttribute(Attribute::Bold),
            Print(hint),
            SetAttribute(Attribute::Reset),
            Print("\n"),
        )
        .unwrap();
    }
    execute!(stdout, ResetColor).unwrap();
}

// --- CORE RENDERING ENGINE ---

/// This is the heart of the UI. It handles the "Draw, Rewind, Type" effect.
///
/// ALGORITHM EXPLANATION:
/// 1. PRE-CALCULATE: We wrap all text *before* printing to know exactly how tall the box is.
/// 2. DRAW SKELETON: We print the Top Border, empty "Wall" lines, and Bottom Border.
/// 3. REWIND: We move the cursor UP back to the top of the box.
/// 4. TYPEWRITER: We print text inside the existing walls, careful not to overwrite borders.
fn render_inline_card(title: &str, raw_lines: Vec<String>, use_typewriter: bool) {
    // 0. BYPASS FOR TESTING
    // If we are running in a CI/CD environment or automated test,
    // we don't want to mess with Raw Mode or delays. Just print simple text.
    if env::var("SUPERSHELL_TEST_MODE").is_ok() {
        println!("[{}]", title);
        for line in &raw_lines {
            println!("{}", line);
        }
        return;
    }

    // 1. ENTER RAW MODE
    // Standard terminal mode ("Cooked Mode") buffers input until Enter is pressed.
    // "Raw Mode" gives us byte-level control over the terminal, which we need
    // to detect keypresses instantly (to skip the typing animation).
    enable_raw_mode().unwrap();
    let mut stdout = stdout();

    // 2. GEOMETRY CALCULATION
    // Get terminal size (cols, rows). Default to 80x24 if it fails.
    let (term_cols, _) = size().unwrap_or((80, 24));

    // Calculate box width:
    // - At least 40 columns (so it's not too skinny)
    // - At most MAX_WIDTH (so it's not too wide)
    // - At most the terminal width (so it doesn't crash on small screens)
    let width = std::cmp::min(std::cmp::max(term_cols, 40), MAX_WIDTH);
    let content_width = (width as usize).saturating_sub(4); // 2 chars for border + 2 chars for padding

    // Wrap text logically before we draw a single pixel.
    // We use the `textwrap` crate to ensure words don't get cut in half.
    let mut final_lines = Vec::new();
    for raw_line in raw_lines {
        if raw_line.is_empty() {
            final_lines.push("".to_string());
        } else {
            let wrapped = fill(&raw_line, content_width);
            for subline in wrapped.split('\n') {
                final_lines.push(subline.to_string());
            }
        }
    }

    let height = final_lines.len();

    // 3. DRAW SKELETON (Instant Render)
    // We draw the empty box first so the user sees the "container" immediately.
    print_top_border(width, title);

    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
    for _ in 0..height {
        print!("│");
        // Print empty space to clear any debris/background
        print!("{}", " ".repeat(content_width + 2));
        // Jump to the right edge to draw the closing wall
        execute!(stdout, MoveToColumn(width - 1)).unwrap();
        print!("│\r\n");
    }
    print_bottom_border(width);

    // 4. THE REWIND
    // We are currently below the bottom border.
    // We move UP past (Bottom Border + All Content Lines).
    // This places the cursor right back at the top-left (start of content).
    execute!(stdout, MoveUp((height + 1) as u16)).unwrap();

    // 5. CONTENT FILL
    let mut skipped = false;

    for line in final_lines {
        // Move cursor to inside the left border (Column 2, 0-indexed)
        execute!(stdout, MoveToColumn(2)).unwrap();
        execute!(stdout, SetForegroundColor(TEXT_COLOR)).unwrap();

        if use_typewriter {
            for char in line.chars() {
                print!("{}", char);
                // Flush is required because in Raw Mode, the buffer might not
                // send the character to the screen immediately otherwise.
                stdout.flush().unwrap();

                // HANDLE SKIP LOGIC (Polling)
                // We check if the user pressed any key. If so, we "skip" the delay.
                if !skipped {
                    // Poll for 0 seconds (instant check)
                    if poll(Duration::from_secs(0)).unwrap() {
                        if let Event::Key(key) = read().unwrap() {
                            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                                skipped = true;
                            }
                        }
                    }
                    if !skipped {
                        // The "Typewriter" delay (15ms is a good "fast typing" speed)
                        thread::sleep(Duration::from_millis(15));
                    }
                }
            }
        } else {
            // Instant render for HUD (no delay)
            print!("{}", line);
        }

        // Move down to the next row (keeping horizontal position reset in next loop)
        execute!(stdout, MoveDown(1)).unwrap();
    }

    // 6. CLEANUP / EXIT
    // We are now on the Bottom Border line. Move down 1 to clear the box.
    execute!(stdout, MoveDown(1)).unwrap();
    execute!(stdout, MoveToColumn(0), ResetColor).unwrap();

    // If it was a cutscene, pause for user confirmation
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
        // Add breathing room after the modal closes
        print!("\r\n");
    } else {
        // Just a small gap for HUD
        print!("\r\n");
    }

    // CRITICAL: Always disable raw mode before exiting,
    // otherwise the user's terminal will be stuck in a weird state.
    disable_raw_mode().unwrap();
}

// --- BORDER HELPERS ---

fn print_top_border(width: u16, title: &str) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
    print!("┌──");

    // Print Title in Bold
    execute!(
        stdout,
        SetForegroundColor(TITLE_COLOR),
        SetAttribute(Attribute::Bold)
    )
    .unwrap();
    print!(" {} ", title);

    // Resume Border Line
    execute!(
        stdout,
        SetForegroundColor(BORDER_COLOR),
        SetAttribute(Attribute::Reset)
    )
    .unwrap();

    // Calculate how much "dash" line is needed to fill the width
    let used_len = 3 + 1 + title.chars().count() + 1; // "┌──" + " " + TITLE + " "
    let remaining = (width as usize).saturating_sub(used_len + 1); // +1 for corner

    print!("{}", "─".repeat(remaining));
    execute!(stdout, MoveToColumn(width - 1)).unwrap();
    print!("┐\r\n");
}

fn print_bottom_border(width: u16) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).unwrap();
    print!("└{}", "─".repeat((width as usize).saturating_sub(2)));
    execute!(stdout, MoveToColumn(width - 1)).unwrap();
    print!("┘\r\n");
    execute!(stdout, ResetColor).unwrap();
}
