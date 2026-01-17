# Creating Curriculums (v0.4.0)

The engine is data-driven. You create content by adding YAML files to the `library/` directory.

## File Format
The engine uses a **Universal Loader**, meaning it is flexible about how you structure the file. You can use any of these formats:

1.  **Standard (Recommended):** A `quests` list wrapping the content.
2.  **List:** A plain list starting with `- id: ...`.
3.  **Single:** A single quest object (useful for small drops).

## The Quest Object
This is the top-level container for a module.

| Property | Type | Description |
| :--- | :--- | :--- |
| `id` | String | Unique ID (e.g., `01_awakening`). Used for save states. |
| `title` | String | The display name shown in the CLI. |
| `construct` | Bool | If `true`, the engine enforces "Simulation Mode" checks. |
| `chapters` | List | An ordered list of Chapter objects. |

## The Chapter Object
A chapter represents a distinct phase of the quest (e.g., "Orientation", "Extraction").

| Property | Type | Description |
| :--- | :--- | :--- |
| `title` | String | The chapter name. |
| `intro` | String | (Markdown) Text displayed when the chapter starts. |
| `outro` | String | (Markdown) Text displayed when all tasks are complete. |
| `setup_actions`| List | **(Optional)** Actions to modify the file system before the chapter begins. |
| `tasks` | List | An ordered list of Tasks the user must complete. |

### Setup Actions
These run automatically when a chapter loads to prepare the environment.

```yaml
setup_actions:
  - type: CreateFile
    path: "secret.txt"
    content: "Top Secret"
  - type: CreateDir
    path: "Vault"
  - type: RemovePath
    path: "old_logs.log"
  - type: ResetWorld
    # Clears the entire ~/Construct directory

```

## The Task Object

Tasks are the individual steps the user must perform.

| Property | Description |
| --- | --- |
| `description` | Short summary of the task (e.g., "Ping the sonar"). |
| `instruction` | In-universe explanation of *what* to do. |
| `objective` | Explicit command instruction (e.g., "Type 'ls'"). |
| `hint` | **(Optional)** Helpful tip shown if the user gets stuck. |
| `success_msg` | **(Optional)** Message displayed upon completion. |
| `conditions` | List of requirements to pass the task. |

## Condition Types

You can combine multiple conditions. All must be true for the task to pass.

### Environment Checks

| Type | Parameter | Description |
| --- | --- | --- |
| `WorkingDir` | `path` | Regex matching the current directory name (e.g., `Construct$`). |
| `IsFile` | `path` | checks if a file exists. |
| `PathMissing` | `path` | Checks if a file/directory has been deleted. |
| `EnvVar` | `name`, `value` | Checks if an environment variable matches a value. |

### File Content Checks

| Type | Parameter | Description |
| --- | --- | --- |
| `FileContains` | `path`, `pattern` | File must contain the regex pattern. |
| `FileNotContains` | `path`, `pattern` | File must NOT contain the regex pattern. |
| `FileEmpty` | `path` | File must be 0 bytes. |

### Input Checks

| Type | Parameter | Description |
| --- | --- | --- |
| `CommandMatches` | `pattern` | The user's last command must match this regex. |

## Example Structure

```yaml
quests:
  - id: "01_example"
    title: "The Example"
    chapters:
      - title: "The Beginning"
        intro: "Welcome to the test."
        tasks:
          - description: "Say Hello"
            instruction: "Greet the system."
            objective: "echo hello"
            conditions:
              - type: CommandMatches
                pattern: "^echo hello"
        outro: "Good job."

```
