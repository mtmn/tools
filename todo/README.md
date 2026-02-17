## Prerequisites

*   [ne](https://github.com/vigna/ne)
*   [ripgrep](https://github.com/BurntSushi/ripgrep)

### Shell Commands

*   **`tt`**: Appends the current date/time to the todo file and opens it in `ne`.
*   **`te`**: Opens the todo file in `ne` without appending the date.
*   **`ts`**: Lists all lines starting with "todo:" using `ripgrep`.

### Shortcuts

*   **`Ctrl+T`**: Inserts the string `todo: ` at the cursor.
*   **`Ctrl+D`**: Replaces the first occurrence of `todo` with `done` on the current line.
*   **`Ctrl+W`**: Delete previous word.
*   **`Ctrl+P` / `Ctrl+N`**: Line up / Line down.
*   **`Ctrl+K`**: Delete line.
