[English](licensing.md) | [Русский](licensing.ru.md) | [简体中文](licensing.zh-CN.md)

# Licensing and third-party engines

## JiveFetch source

JiveFetch source code and project documentation are licensed under the
[Apache License 2.0](../LICENSE). `package.json` and `src-tauri/Cargo.toml` carry
the matching SPDX identifier `Apache-2.0`.

Apache-2.0 was selected because it permits private and commercial use,
modification, and redistribution while providing an explicit patent grant and
requiring preservation of license and attribution notices. This document is an
engineering record, not legal advice.

## Current engine model

Version `0.1.1` discovers and invokes user-installed `yt-dlp` and FFmpeg by
absolute path. JiveFetch does not currently redistribute either executable.
The process supervisor passes a typed argument list directly, without a shell.

- `yt-dlp` publishes its source under [The Unlicense](https://github.com/yt-dlp/yt-dlp/blob/master/LICENSE).
- FFmpeg is normally LGPL 2.1-or-later, but optional GPL components make a
  particular build GPL 2-or-later; the exact configuration must be reviewed
  under the [FFmpeg legal guidance](https://ffmpeg.org/legal.html).

The locally validated Homebrew FFmpeg reports GPL components. It is a
development-machine dependency and is not included in JiveFetch artifacts.

## Distribution gate

Before any installer bundles or downloads an engine, the release owner must:

1. record the exact engine version, source, hash, build configuration, and license;
2. confirm compatibility with the intended form of distribution;
3. include every required license, attribution, corresponding-source offer, and notice;
4. expose exact application and engine versions/licenses in the application;
5. verify the final artifact rather than relying on the license of a different build.

Signed installers, managed engine updates, SBOMs, and third-party notice bundles
remain release work. Native CI in this repository validates source builds and
process ownership; it does not by itself approve redistribution.
