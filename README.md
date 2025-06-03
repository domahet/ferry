# Ferry 🛶

A ferry for your files. Grab them from a directory, move to somewhere else, and then drop them, instead of the traditional cp/mv workflow of specifying paths ahead of the time.


## Features

* **Select Files**:
    * Interactively choose files in a Text User Interface (TUI).
    * Select files by regular expression.
    * Select files by providing direct paths.
    * Option to specify a starting directory for interactive or regex selection.
* **Copy Files**: Copy selected files to the current directory.
* **Move Files**: Move selected files to the current directory.
* **List Selected Files**: View currently selected files, with options for absolute or relative paths.

## Installation

With [Rust](https://www.rust-lang.org/tools/install) (version 1.87.0 or higher) installed you can install the package straight from crates.io using cargo:

```bash
cargo install --locked ferry
```

## Usage

Ferry operates using subcommands: `select`, `copy`, `move`, and `list`.

### `ferry select`

Select files to be processed by `copy` or `move`. By default, `select` launches an an interactive TUI if no other arguments are provided.

* **Interactive Selection (TUI)**:
  ```bash
    ferry select
    # or specify a starting directory
    ferry select --interactive --path /path/to/start
  ```

    Navigate with arrow keys, toggle selection with `Space`, and confirm with `Enter`. Only direct files in the specified directory are listed.

* **Regular Expression Selection**:
  ```
    ferry select --regex ".*\.rs$" --path src
  ```

    Selects files matching the regex within the `src` directory.

* **Direct Selection**:
  ```bash
    ferry select file1.txt *.rs
  ```

Then in another directory execute ferry move or ferry copy to drop them off, depending on whether you want to emulate mv or cp.
