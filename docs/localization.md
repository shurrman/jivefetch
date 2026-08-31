[English](localization.md) | [Русский](localization.ru.md) | [简体中文](localization.zh-CN.md)

# Localization policy

## Supported languages

- `en` — English, mandatory first-run default and source locale.
- `ru` — Russian.
- `zh-CN` — Simplified Chinese.

The application never silently replaces the first-run default with the OS language.
After a user explicitly selects a language, that choice is stored locally and restored.

## Application strings

- Every user-visible string is a stable key in `src/i18n.tsx`; components do not embed
  English, Russian, or Chinese prose.
- All three dictionaries must expose the same keys. TypeScript enforces the key set.
- Dates and numbers use the selected locale; persisted domain values remain neutral.
- Task states, errors, notifications, accessibility labels, menu items, installer text,
  and updater messages are part of localization scope.
- Engine output is never shown as if translated. JiveFetch maps known structured errors
  to localized explanations and labels unknown redacted output as engine diagnostics.
- EN is the fallback for a missing runtime key, but missing committed translations fail
  validation and are not release-ready.

## Documentation

English files use the unsuffixed name. Russian files use `.ru.md`; Simplified Chinese
files use `.zh-CN.md`. `AGENTS.md` is the legacy Russian canonical instruction file,
with `AGENTS.en.md` and `AGENTS.zh-CN.md` translations.

Every Markdown file begins with direct links to its three language variants. Links
inside a translation should lead to the same-language target where one exists.

Translation is semantic, not word-for-word: commands, identifiers, code, state names,
paths, and security invariants remain exact. No translation may weaken a safety rule,
claim an unimplemented feature, or omit a release gate.

## Change workflow

1. Change the English/source meaning and application keys.
2. Update RU and Simplified Chinese in the same logical phase.
3. Run key-parity and Markdown-translation/link checks.
4. Review layout with long Russian labels and CJK glyphs.
5. Commit only when all three variants are complete.

## Acceptance checks

- Fresh storage opens the app in English.
- Each language can be selected and survives restart.
- No user-visible key is missing in any dictionary.
- HTML `lang` follows the selected locale.
- Queue controls remain usable with the longest labels.
- Every tracked Markdown base has EN/RU/zh-CN variants and mutual links.
- README commands and feature status have the same meaning in every language.
