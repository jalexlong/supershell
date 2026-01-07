# Creating Curriculums

The engine is entirely data-driven. The order of chapters in the `quests.yaml` file determines the order of the course.

## The Chapter Object
| Property | Description |
| :--- | :--- |
| `id` | Unique identifier for the chapter. |
| `title` | The display name shown in the status command. |
| `briefing` | Text played when the student first enters the chapter. |
| `debriefing` | Text played after the final checkpoint is cleared. |
| `next_chapter_id` | The ID of the next chapter (Set to `null` to end the game). |

## The Checkpoint Object
Checkpoints are the individual tasks within a chapter.
- `instruction`: The "In-Universe" reason for the task.
- `objective`: The actual command the user should try to learn.
- `success`: The confirmation message played upon completion.

## Condition Types
You can combine multiple conditions into a single checkpoint.
1. `CommandMatches`: Regex pattern to check the user's input.
2. `FileExists`: Checks if a specific file path exists.
3. `FileContains`: Regex pattern check against the contents of a file.
4. `FileMissing`: Ensures a file has been deleted.
