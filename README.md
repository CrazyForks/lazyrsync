<div align="center">

  <img src="assets/logo.svg" alt="lazyrsync" width="600" />

  A terminal UI for rsync 🔄

  <img src="assets/demo.gif" alt="lazyrsync demo" width="800" />

  [![Built With Ratatui](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=FF6B6B&labelColor=222322&color=E23636)](https://ratatui.rs)
  [![crates.io](https://img.shields.io/crates/v/lazyrsync.svg?color=E23636&labelColor=222322)](https://crates.io/crates/lazyrsync)
  [![License](https://img.shields.io/badge/license-MIT-E23636.svg?labelColor=222322)](LICENSE)
  [![Docs](https://img.shields.io/badge/docs-lazyrsync.westpoint.io-E23636?labelColor=222322)](https://lazyrsync.westpoint.io)

  <a title="This tool is Tool of The Week on Terminal Trove, The $HOME of all things in the terminal" href="https://terminaltrove.com/lazyrsync"><img src="https://cdn.terminaltrove.com/media/badges/tool_of_the_week/png/terminal_trove_tool_of_the_week_gold_transparent.png" alt="Terminal Trove Tool of The Week" width="150" /></a>

</div>

A terminal UI for `rsync` — manage reusable profiles, preview a transfer as a
structured diff **before** running it, and watch a live run with progress and
cancellation. All from the terminal, including over SSH where a desktop GUI
can't reach.

## Contents

- [Why](#why)
- [Features](#features)
- [Install](#install)
- [Quickstart](#quickstart)
- [Keybindings](#keybindings)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Why

`rsync` is the right tool for backups and syncs, but its flags are easy to get
wrong and a single mistake can delete data. lazyrsync keeps you in the terminal
while giving you the safety of a GUI: save your transfers once, see exactly what
a run will change before it runs, and keep destructive flags behind a gate.

## Features

### Profiles & tasks

Save a Source → Destination pair once and rerun it with a keystroke.

![Profiles & tasks](assets/profiles.gif)

### Dry-run preview

Press `p` and watch the transfer resolve into a `+`/`~`/`-` diff with stats.
Nothing is written until you say so.

![Dry-run preview](assets/preview.gif)

### Live run & cancel

`r` runs it — a progress bar fills with byte and file counts. Press `c` to
stop mid-transfer.

![Live run & cancel](assets/run.gif)

### Flags & `--delete` gating

Toggle rsync's options as checkboxes. Flip on `--delete` and it makes you
confirm before anything can be removed.

![Flags & --delete gating](assets/flags.gif)

### Over SSH

Put a `user@host:/path` on either side of a task and it runs over SSH — remote
source downloads, remote destination uploads.

![Over SSH](assets/ssh.gif)

### Snapshots

Keep numbered, hardlinked versions with `--link-dest` — each run writes the
next directory (`1/`, `2/`, …).

![Snapshots](assets/snapshot.gif)

## Install

`rsync` must be on your `$PATH`.

```bash
cargo install lazyrsync                          # crates.io
cargo binstall lazyrsync                          # prebuilt release binary
brew install westpoint-io/lazyrsync/lazyrsync     # Homebrew
yay -S lazyrsync                                   # AUR (Arch)
```

Or build from source:

```bash
cargo install --path .
```

## Quickstart

```bash
lazyrsync            # launch the TUI
```

1. Press `]` to switch to the **Profiles** sub-tab, then `a` to add a profile.
2. Back on **Tasks** (`]`), press `a` to add a task: an **ID**, an **Action**
   (Sync ⇄ Snapshot with `←/→`), a **Source**, and a **Destination**. Either
   path may be local or a remote `user@host:/path`.
3. Press `p` to **preview** (dry-run) — you'll see the exact `+`/`~`/`-`
   changes and stats, and nothing is written.
4. Press `r` to **run** it. Watch progress in the **Runs** panel; press `c` to
   cancel.

A task is just **Source → Destination**, exactly like the rsync command line —
no push/pull, no separate "remote" field. A trailing `/` on the Source copies
its _contents_; without it, the folder itself is copied.

### Resolve / inspect (headless)

lazyrsync can print the exact rsync command a profile resolves to — handy for
review or dropping into a script. Running transfers headlessly isn't wired up
yet; use the TUI to actually run them.

```bash
lazyrsync list            # list profiles and their resolved rsync commands
lazyrsync run NAME        # print the resolved command(s) for a profile
lazyrsync run NAME -n     # print the dry-run form (with -n)
```

## Keybindings

Press `?` in the app for the full, context-aware list. The essentials:

| Key | Action |
|-----|--------|
| `1`–`4`, `Tab` | Focus a rail panel (Runs / Tasks · Profiles / Flags / Filters) |
| `]` | Toggle the Tasks / Profiles sub-tab |
| `j`/`k`, `↑`/`↓` | Move the cursor |
| `space` / `enter` | Select the task (or toggle the highlighted flag) |
| `a` | Add a task (or profile, on the Profiles sub-tab) |
| `p` | Preview (dry-run) the selected task |
| `r` / `R` | Run the selected task / run every task in the profile |
| `e` / `s` / `i` / `x` | Edit Basics / SSH / Filters / Advanced |
| `d` | Delete (confirm first) |
| `V` | Visual range (multi-select), then `r`/`d` acts on the block |
| `c` | Cancel the running job |
| `/` | Filter the list, or search the run output |
| `q` / `Esc` | Quit |

## Configuration

Profiles and settings live under `$XDG_CONFIG_HOME/lazyrsync/` (typically
`~/.config/lazyrsync/`):

- `profiles.toml` — your profiles and tasks
- `settings.toml` — preferences (theme, hints, `skip_delete_warning`)

### Dynamic paths

Source and destination paths can contain placeholders, resolved every time the
task runs — so one saved task can write to a new dated folder each night:

| Placeholder | Expands to |
|-------------|------------|
| `{now}` | today's date, `2026-07-27` |
| `{now:FORMAT}` | any [strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) format, e.g. `{now:%Y/%m/%d}` or `{now:%H%M}` |
| `{utcnow}`, `{utcnow:FORMAT}` | the same in UTC |
| `{hostname}` | this machine's hostname |
| `{user}` | the current user |
| `$VAR`, `${VAR}` | an environment variable |
| `~` | your home directory |

```toml
dest = "~/backups/{hostname}/{now:%Y-%m-%d}/"
```

Unknown placeholders, unset variables and a bare `%` are left exactly as typed,
and the dry-run preview always shows the resolved path before anything runs.
Write `{{` for a literal brace and `$$` for a literal `$`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the build/test/lint commands, the
module map, and the code + UI conventions.

## Acknowledgements

- [lazygit](https://github.com/jesseduffield/lazygit) — the TUI whose
  keyboard-driven, panel-based workflow inspired this one.
- [ratatui](https://ratatui.rs) — the Rust TUI library lazyrsync is built on.

## License

[MIT](LICENSE).
