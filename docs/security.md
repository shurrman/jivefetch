[English](security.md) | [Русский](security.ru.md) | [简体中文](security.zh-CN.md)

# Security and authentication

## 1. Security goals

JiveFetch runs third-party tools against untrusted URLs and may temporarily access
browser authentication. Its security goal is to minimize privilege and secret
exposure while ensuring that a compromised or malformed media page cannot gain Tauri
capabilities or execute arbitrary local commands.

JiveFetch does not promise to bypass DRM, paywalls, access controls, or platform
policy. Authentication only reuses access the user already has and is authorized to
use.

## 2. Threat model

Protect against:

- malicious URL strings, titles, filenames, metadata, thumbnails, and engine output;
- command/shell injection;
- path traversal and output overwrite outside the approved destination;
- secret leakage through logs, SQLite, UI state, process previews, crash reports, or
  support bundles;
- another local user reading temporary cookie material;
- tampered sidecar updates;
- a remote page attempting to invoke privileged Tauri commands;
- stopping unrelated system processes due to PID reuse or name matching.

The MVP does not defend against a fully compromised user account, kernel, browser,
or OS credential store.

## 3. Authentication modes

### 3.1 Browser reference

Preferred mode stores only:

- browser kind;
- optional profile identifier/path after validation;
- source scope and display label.

The engine receives the corresponding `--cookies-from-browser` style argument. Raw
cookie values are not copied into JiveFetch storage. The UI explains that the browser
may need to be closed or unlocked depending on the platform and engine behavior.

### 3.2 Imported cookie file

For user-approved Netscape cookie files:

1. read and validate the selected file locally;
2. encrypt its bytes with a random per-installation data encryption key;
3. store the encrypted blob and non-secret metadata in app data;
4. store the data encryption key in the OS credential service;
5. decrypt only for a short-lived credential lease;
6. create a user-only temporary file with restrictive permissions/ACL;
7. pass only that path as an argument;
8. remove the file after the owned process tree exits;
9. clean stale app-owned files during startup recovery.

If the OS credential service is unavailable, JiveFetch fails closed for persistent
imports. It may offer session-only use; it must not fall back to plaintext storage or
a predictable machine-derived key.

### 3.3 Future credentials

Passwords, tokens, proxy credentials, and service-specific OAuth flows are separate
credential types. They reuse the credential broker but require their own threat model
and cannot be smuggled into generic extra arguments.

## 4. Secret data rules

The following are always treated as secret:

- raw cookies and cookie database copies;
- passwords, tokens, Authorization/Proxy-Authorization headers;
- signed URL query parameters and fragment data where applicable;
- decrypted temp file content and its path when the path reveals a credential ID;
- keychain item identifiers that could enable unauthorized retrieval.

Redaction happens before persistence. Redactors operate on structured fields first
and conservative text patterns second. A log field that cannot be safely classified
is omitted.

Support bundles are allowlist-based and show a manifest before export. They never
include the database wholesale, credential blobs, browser profiles, raw command
arguments, environment dumps, or download artifacts.

## 5. Process execution boundary

- Use direct executable invocation with an argument vector; no shell.
- Resolve executables through the verified engine registry, not `$PATH` alone.
- Provide a minimal explicit environment. Remove known secret-bearing inherited
  variables unless required.
- Use an app-owned working directory and canonical approved output directory.
- Treat stdout/stderr as untrusted data; cap line/event size and total retained logs.
- Escape all metadata before rendering; never inject engine HTML into the webview.
- Do not expose arbitrary environment, headers, postprocessor arguments, or executable
  paths through the normal UI.

## 6. Filesystem boundary

Before creating or deleting a path:

1. resolve the selected destination through platform-safe canonicalization;
2. apply a sanitized output template with component length limits;
3. reject traversal, device names, alternate streams, control characters, and paths
   escaping the approved root;
4. choose an explicit collision policy;
5. track every created artifact by task and attempt.

Remove-with-files deletes only tracked canonical files. It never recursively deletes
the destination root, follows an untrusted symlink outside the root, or expands an
unresolved glob.

## 7. Tauri and webview boundary

- Define a narrow Tauri capability set per window.
- Validate every command payload in Rust, including size and URL scheme limits.
- Serve the application UI from packaged local assets.
- Use a restrictive Content Security Policy; remote scripts are forbidden.
- Fetch thumbnails through a constrained path that strips credentials, limits size
  and content type, and grants no Tauri IPC capability to remote origins.
- Disable navigation to arbitrary remote pages inside the privileged webview; open
  approved external links through the OS after confirmation.
- Keep secrets out of React state, browser storage, clipboard, and frontend logs.

## 8. Engine and application updates

- Pin an engine compatibility range and source artifacts only from declared upstreams.
- Distribute a signed manifest containing version, target triple, size, hash, source,
  and license metadata.
- Verify manifest signature and artifact hash before installation.
- Download to a staging path, fsync as appropriate, set permissions, run a bounded
  version check, then activate atomically.
- Retain a known-good prior version and roll back after failed health checks.
- Never execute a partially downloaded or unverified file.
- Application updates use the platform/Tauri signing mechanism and cannot be replaced
  by the engine updater.

## 9. Privacy defaults

- No telemetry or analytics in the initial release.
- Local history can be disabled or cleared without deleting downloaded media.
- Clipboard monitoring is off by default and ignores non-URL clipboard content.
- Deep links always show a review screen before a download starts.
- Authentication is scoped per task/source where practical, not applied globally to
  every URL without visibility.

## 10. Security verification

- Injection tests for URLs, titles, format labels, templates, and engine output.
- Property/fuzz tests for path sanitization and redaction.
- Permission/ACL tests for imported-cookie temp files on every target OS.
- Database/log/support-bundle scans using synthetic sentinel secrets.
- Tauri capability tests proving remote content cannot invoke commands.
- Tampered manifest, wrong hash, downgrade, interrupted update, and rollback tests.
- Process ownership tests proving Stop does not affect an unrelated same-name process.

Security-sensitive failures block release rather than degrading to an insecure mode.

## 11. Tracked upstream advisory

The Linux dependency graph currently inherits `glib 0.18.5` through Tauri and GTK
0.18. It is affected by `GHSA-wrw7-89jp-8q8g` in `VariantStrIter`; JiveFetch does not
call that API. The patched `glib 0.20` is incompatible with GTK's current `^0.18`
constraint, so this is an explicit temporary exception for the private `v0.1.0`
pre-release only. Updating the Tauri/GTK chain or otherwise removing the advisory is
a stable-release gate.
