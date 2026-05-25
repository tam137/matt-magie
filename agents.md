# AI Rules & Guidelines for Matt-Magie

## Strict English Policy
- **Primary Directive:** English is the mandatory language for all technical artifacts.
- **GUI Labels:** Use standard English terminology (e.g., "Settings" instead of "Einstellungen", "Submit" instead of "Absenden").
- **CLI Output:** Ensure all `console.log`, `print`, or logger statements use English.
- **Exception:** Only communicate in German within the chat window if the user speaks in German. Everything written into project files must be English.

## Rust Coding Standards
- **Edition:** Rust Edition 2024.
- **Formatting:** Adhere to standard Rust formatting conventions.
- **Documentation:** Write all docstrings and code comments in clear, technical English.

## Testing & Verification Policy
- **Simple Implementations:** Standard unit tests only (`cargo test`).
- **Release Verification Requirement:** Run `./run_verify.sh` ONLY if the USER explicitly requests or mentions it during a release cycle. Do NOT run tournament verifications by default for releases or simple development changes unless explicitly asked.

## Git & Version Control Policy
- **Strict Limit on Git Operations:** The AI must NEVER automatically or preemptively execute `git commit` or `git push` commands.
- **Commits Rule:** Only create a Git commit if the USER explicitly asks/instructs the AI to perform a commit.
- **Pushes Rule:** Only execute a Git push if the USER explicitly mentions push or explicitly tells the AI to perform a push.

