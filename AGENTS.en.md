[Русский](AGENTS.md) | [English](AGENTS.en.md) | [简体中文](AGENTS.zh-CN.md)

# Agent instructions

This is the English translation of the canonical Russian `AGENTS.md`. If wording
diverges, preserve the stricter safety rule and update all translations together.

## Entry points

- `README.md` — repository overview and verified commands.
- `CHANGES.md` — changes, before/after metrics, known issues.
- `MEMORY.md` — durable, secret-free decisions and continuation point.
- `docs/` — requirements, architecture, lifecycle, security, packaging, licensing,
  roadmap, research references, and localization policy.

## Working style

- Read `MEMORY.md` before work. Act independently and minimize interruptions.
- Inspect similar roles/components when a repeated-pattern bug is found.
- Ask before destructive actions: `rm`, playbook deletion, inventory replacement, or
  broad irreversible edits.
- If commits are requested, keep logical phases separate. Verify before commit/push.
- After refactoring, update README, AGENTS, CHANGES, and MEMORY as applicable.
- Summaries for large work include before/after metrics and pre-existing problems.

## Legacy repository prohibitions

- Never commit vault passwords (`.ansible.vault`, `vault_pass*`, or equivalents).
- Do not introduce deprecated Ansible syntax: `with_items`, ambiguous `include:`,
  flat module arguments, or `become: yes/no`.
- Do not mechanically replace existing `shell:` tasks with native modules.

## JiveFetch context

JiveFetch is a local-first cross-platform media-download manager targeting Tauri 2,
Rust, React/TypeScript, `yt-dlp`, FFmpeg/ffprobe, and optional aria2.

## Development rules

- Keep queue, task lifecycle, and process ownership in Rust. React is a projection.
- Persist scheduler transitions transactionally in SQLite before publishing UI events.
- A PID is not ownership. Stop only the owned Unix process group/session or Windows
  Job Object. Never use executable-name-wide kill commands.
- Portable Pause means resumable controlled stop plus a new attempt. OS suspension is
  only an optional optimization.
- UI input must become typed Rust arguments, never a shell command string.
- Never log or commit cookies, tokens, auth headers, browser profiles, decrypted temp
  files, or signed URLs.
- Store keys in the OS credential service; encrypt large cookie blobs with a random
  key held there. Never fall back to plaintext or machine-derived keys.
- SQLite migrations are forward-safe and recovery-tested.
- Queue features require state-transition and crash/restart tests. Process control uses
  deterministic helper processes, not bulk real downloads.
- Packaging claims require native Windows, macOS, and Linux tests.
- A release tag must match the application/package versions. Keep releases marked as
  pre-release until signing, notarization, sidecar licensing, and release gates pass.
- MediaHarbor and FlowGrab are research references only. Do not copy code, assets,
  UI text, or internal structure.
- The private `origin` is configured. Commit or push only when the user explicitly
  requests it and the relevant checks pass.

## Localization and documentation

- Every Markdown document has complete EN/RU/Simplified Chinese versions with links
  to the other translations at the top.
- Every user-visible string goes through i18n. First run defaults to EN; an explicit
  RU or Simplified Chinese choice is remembered locally.
- Update all three languages in one phase. Missing translations mean work is incomplete.
- Document only commands that exist and have been verified; mark plans as future.
- Before handoff run relevant formatting, lint, tests, `git diff --check`, relative
  Markdown-link validation, and secret checks.
