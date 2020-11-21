# Task Board

Simple & fast Taskwarrior web frontend

## Rationale

There are seveval third party Taskwarrior web interfaces, but they fell into one of these categories:

- They feel bloated. Installing a docker container for a frontend to Taskwarrior feels in total opposite to the spirit of the original tool.
- They are commercial "Software As A Service" applications. I have nothing against paying for good software, but hosting my tasks lists unencrypted on an third party server is a big no no for me.

## Goals

### Usability

- Be fast, **any perceived latency is a bug**
- Be easily usable on mobile (this is a priority target compared to desktop).
- Support the whole `task` workflow: add, delete, edit, undo, etc. (no less, no more).

### Design choices

- Do one thing, and do it well: get configuration from the usual `~/.taskrc` and offer a convenient UI to interact with tasks though the `task` binary, everything else is out of scope. In other words, **this is a frontend to the `task` Taskwarrior command line tool, rather than a frontend to Taskwarrior tasks using a different model.**
  **This means all your customization in `~/.taskrc` that affects report output, priorities, etc. is respected.**
- Application is a single binary, with everything (including assets) in it. This siplify deployment and avoids systemd calls to open external files.
- Race free task edition: tasks are always manipulated with their UUID, which avoids many pitfalls when tasks are renumbered
- Minimalist presentation layer for simplicity and performance:
  - Use plain HTML as much as possible, with semantic elements.
  - Use JavaScript only when absolutely necessary
  - Minimal use of CSS, with a ["drop in" stylesheet](https://github.com/dohliam/dropin-minimal-css#list-of-frameworks)
  - Use font icons, instead of images
- Improve my Rust skills (which tend to get rusty for lack of practice :smirk:).

## Install

_TODO_

## Deployement

_TODO reverse proxy_

## Progress

### Done

- basic report display

### TODO

- command line interface
- change task attributes from report
- remove jQuery dependency
- suggested values from column/type
- web shell
- web shell completion
- "the remaining 80%"

## Coding style

- No `unsafe` usage
- Passes `cargo clippy` without warnings
- Passes `cargo fmt`
- Minimal `use` usage, prefer explicit namespace usage inline, I find it more readable most of the time.

## License

_TODO_
