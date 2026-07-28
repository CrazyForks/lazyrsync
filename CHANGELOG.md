# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-07-28

### Added

- Headless runs: `lazyrsync run PROFILE` runs every task in a profile without
  the TUI, and `PROFILE/TASK` runs a single task by the id `list` prints. Suited
  to cron and systemd timers — Snapshot tasks resolve their `--link-dest` chain
  fresh on every run, which no static crontab line can express.
- `run -n` for a real dry run, `--yes` to allow tasks that delete at the
  destination, and `-v` to include rsync's own output per task.
- Documented exit codes: `0` success, `1` refused for want of `--yes`, `2`
  unknown target or unloadable config, `3` a task that could not start or was
  killed, otherwise the first failing task's rsync exit code.
- Dynamic paths: `{now}`, `{now:FORMAT}`, `{utcnow}`, `{hostname}`, `{user}`,
  `$VAR`/`${VAR}` and `~` expand every time a task runs, so one saved task can
  write to a new dated folder each night. Unknown placeholders and unset
  variables are left as typed; `{{` and `$$` escape.
- A separate opt-out per confirmation prompt in `settings.toml`
  (`skip_delete_warning`, `skip_run_confirm`, `skip_remove_confirm`), all
  `false` by default.

### Changed

- A headless real run passes rsync `-q` and reports one line per task; a dry run
  is never quiet, and neither carries the TUI-only `--info=progress2` or
  `--stats` flags.
- `profiles.toml` rejects unknown keys instead of ignoring them, so a typo in a
  field name fails loudly at load time rather than silently dropping a setting.

### Fixed

- A task that enables `--delete-excluded` on its own now warns, matching the
  existing `--delete` warning.
- Placeholders resolve before a path is classified as remote, so a placeholder
  that expands to contain a colon no longer looks like an SSH target.

## [0.1.1] - 2026-07-13

### Changed

- Feature GIFs are committed in the repo and referenced relatively, so they
  render on both GitHub and the crates.io page.

## [0.1.0] - 2026-07-10

### Added

- Initial release: an rsync terminal UI with reusable profiles and tasks,
  dry-run diff preview, live run progress with cancellation, a filters/flags
  editor, SSH remotes (`user@host:/path`), and local snapshots.

[Unreleased]: https://github.com/westpoint-io/lazyrsync/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/westpoint-io/lazyrsync/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/westpoint-io/lazyrsync/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/westpoint-io/lazyrsync/releases/tag/v0.1.0
