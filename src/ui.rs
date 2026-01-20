use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
use std::env;
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;
use textwrap::fill;

// --- THE SAFETY NET ---
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Self {
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = stdout();
        execute!(stdout, Hide, MoveTo(0, 0)).unwrap();
        Self
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        disable_raw_mode().unwrap_or(());
        execute!(
            stdout,
            Show,
            ResetColor,
            SetAttribute(Attribute::Reset),
            Clear(ClearType::All),
            MoveTo(0, 0)
        )
        .unwrap_or(());
    }
}

// --- HELPER: DRAW THE BOX WITH TITLE ---
// Strategy #2: "Chunking" - We create a container with a labeled header
fn draw_box(x: u16, y: u16, width: u16, height: u16, title: &str) {
    let mut stdout = stdout();

    // Box Styling (Matrix/Hacker Theme)
    let border_color = Color::Green;
    let title_color = Color::White;

    let top_left = '┌';
    let top_right = '┐';
    let bottom_left = '└';
    let bottom_right = '┘';
    let horizontal = '─';
    let vertical = '│';

    execute!(stdout, SetForegroundColor(border_color)).unwrap();

    // 1. Draw Top Border with Title
    execute!(stdout, MoveTo(x, y)).unwrap();
    print!("{}", top_left);

    // Strategy #1: Visual Signposting
    // We print a short line, then the Title in a different color, then the rest of the line
    print!("{}{}", horizontal, horizontal); // Little dash before title

    execute!(
        stdout,
        SetForegroundColor(title_color),
        SetAttribute(Attribute::Bold)
    )
    .unwrap();
    print!(" {} ", title); // The Title (e.g., " 📜 MISSION BRIEF ")
    execute!(
        stdout,
        SetForegroundColor(border_color),
        SetAttribute(Attribute::Reset)
    )
    .unwrap();

    // Calculate remaining dash length
    // Width - corners(2) - dashes(2) - title_len - padding(2)
    let title_len = title.chars().count() as u16;
    let used_width = 2 + 2 + title_len + 2;
    let remaining_width = if width > used_width {
        width - used_width
    } else {
        0
    };

    print!(
        "{}",
        horizontal.to_string().repeat(remaining_width as usize)
    );
    print!("{}", top_right);

    // 2. Draw Side Borders
    for i in 1..height - 1 {
        execute!(stdout, MoveTo(x, y + i)).unwrap();
        print!("{}", vertical);
        execute!(stdout, MoveTo(x + width - 1, y + i)).unwrap();
        print!("{}", vertical);
    }

    // 3. Draw Bottom Border
    execute!(stdout, MoveTo(x, y + height - 1)).unwrap();
    print!(
        "{}{}{}",
        bottom_left,
        horizontal.to_string().repeat((width - 2) as usize),
        bottom_right
    );

    stdout.flush().unwrap();
}

// --- THE RENDERER ---
pub fn play_cutscene(text: &str) {
    // Testing Bypass
    if env::var("SUPERSHELL_TEST_MODE").is_ok() {
        println!("\n{}\n", text);
        return;
    }

    let _guard = TerminalGuard::new();
    let mut stdout = stdout();

    // 1. CALCULATE DIMENSIONS
    let (cols, rows) = size().unwrap_or((80, 24));

    // Box width: 80% of screen or max 80 chars
    let box_width = std::cmp::min((cols as f32 * 0.8) as u16, 80);
    // Inner width for text (Width - 4 for borders/padding)
    let text_width = box_width - 4;

    // Wrap text
    let wrapped_content = fill(text, text_width as usize);
    let content_lines: Vec<&str> = wrapped_content.split('\n').collect();

    // Height = content lines + 2 (top/bottom padding) + 2 (borders)
    let box_height = (content_lines.len() as u16) + 4;

    // Center logic
    let start_x = (cols - box_width) / 2;
    let start_y = (rows.saturating_sub(box_height)) / 2;

    // 2. CLEAR SCREEN
    execute!(
        stdout,
        Clear(ClearType::All),
        SetForegroundColor(Color::Green), // Default text color
    )
    .unwrap();

    // 3. DRAW THE HUD
    // We use a fixed title for the cutscene.
    // You can change "📜 MISSION BRIEF" to whatever fits your lore.
    draw_box(start_x, start_y, box_width, box_height, "📜 MISSION BRIEF");

    // 4. TYPEWRITER EFFECT
    let mut skipped = false;
    let mut cursor_x = start_x + 2;
    let mut cursor_y = start_y + 2;

    execute!(stdout, MoveTo(cursor_x, cursor_y)).unwrap();

    for char in wrapped_content.chars() {
        if char == '\n' {
            cursor_y += 1;
            execute!(stdout, MoveTo(cursor_x, cursor_y)).unwrap();
        } else {
            print!("{}", char);
            stdout.flush().unwrap();
        }

        if !skipped {
            if poll(Duration::from_secs(0)).unwrap() {
                if let Event::Key(key) = read().unwrap() {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                        skipped = true;
                    }
                }
            }
            // Strategy #4: Typewriter Speed (25ms is a good reading speed)
            thread::sleep(Duration::from_millis(25));
        }
    }

    // 5. PROMPT
    let prompt = ">> PRESS [ENTER] TO CONTINUE_";
    if cursor_y < start_y + box_height - 1 {
        cursor_y += 2;
    }

    // Position prompt centered-ish or aligned left inside box
    execute!(
        stdout,
        MoveTo(start_x + 2, cursor_y),
        SetForegroundColor(Color::DarkGrey),
        Print(prompt)
    )
    .unwrap();

    // Wait for input
    loop {
        if let Event::Key(key) = read().unwrap() {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                break;
            }
        }
    }
}
