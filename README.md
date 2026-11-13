# Ferry 🛶

A ferry for your files. Grab them from a directory, move to somewhere else, and then drop them, instead of the traditional cp/mv workflow of specifying paths ahead of the time.


## Features

* **Select files and directories**:
    * Interactively, in a terminal UI.
    * By regular expression, searching a directory recursively.
    * By passing paths directly.
* **Copy or move** the selection into the current directory. Directories are copied recursively, and moves work across filesystems.
* **Review and edit** the selection with `list`, `deselect` and `clear`.

Selections are additive: each `ferry select` adds to what is already selected, until a `copy` or `move` drops them off.

## Installation

With [Rust](https://www.rust-lang.org/tools/install) (version 1.85.0 or higher) installed you can install the package straight from crates.io using cargo:

```bash
cargo install --locked ferry
```

## Usage

```bash
cd ~/Downloads
ferry select report.pdf photos/    # pick up a file and a directory
cd ~/Documents
ferry move                         # drop them off here
```

### `ferry select`

* **Interactive selection** (the default when no items are given):
  ```bash
  ferry select
  # or browse a different directory
  ferry select --path /path/to/start
  ```

  Lists the files and directories directly inside the directory. Move with the arrow keys or `j`/`k` (also `PgUp`/`PgDn`, `Home`/`End`), toggle with `Space`, toggle everything with `a`, confirm with `Enter`. `q` or `Esc` cancels without changing the selection.

* **Regular expression selection**:
  ```bash
  ferry select --regex '\.rs$' --path src
  ferry select --regex '^docs/.*\.md$'
  ```

  Searches `--path` (default: the current directory) recursively and selects the files whose path, relative to `--path` and written with `/` separators, matches the pattern. Directories themselves are not selected.

* **Direct selection**:
  ```bash
  ferry select file1.txt *.rs some_directory/
  ```

Add `--dry-run` to see what would be selected without changing anything.

### `ferry copy` / `ferry move`

Run in the destination directory to copy or move the selected items there, emulating `cp -r` or `mv`.

Everything is checked before anything is touched: if an item is missing, two items share a name, or a destination already exists, ferry reports every problem and transfers nothing. Use `--force` to replace existing destinations. The replaced item is only removed once the new one is in place.

Items are removed from the selection as they are transferred, so if something fails midway, running the command again continues with what is left.

### `ferry list`

Shows the selected items, marking any that no longer exist. Use `--relative` for paths relative to the current directory.

### `ferry deselect` / `ferry clear`

```bash
ferry deselect file1.txt    # remove specific items
ferry deselect --missing    # remove items that no longer exist
ferry clear                 # remove everything
```

### Global options

`--verbose` prints more detail, and `--silent` prints only errors.
