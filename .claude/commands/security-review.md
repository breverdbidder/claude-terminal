# Security Review

Run a security audit on the codebase or recent changes.

## Instructions

Perform a security review of: $ARGUMENTS (or entire codebase if no arguments)

### Scan Areas

#### 1. Command Injection
- Search `src-tauri/src/` for `cmd /C`, `Command::new`, process spawning
- Verify all user-controlled strings are properly escaped
- Check `write_to_terminal` for injection vectors

#### 2. Path Traversal
- Search for file operations using user-provided paths
- Check workspace loading/saving for path validation
- Ensure no `..` traversal is possible

#### 3. Secrets Detection
- Search entire codebase for patterns: API keys, tokens, passwords, secrets
- Check `.env` files, config files, hardcoded strings
- Verify `.gitignore` covers sensitive files

#### 4. IPC Security
- Review `src-tauri/capabilities/default.json` for overly broad permissions
- Check that all IPC commands validate their inputs
- Verify error messages don't leak internal paths or state

#### 5. Dependency Audit
- Check for known vulnerabilities: `npm audit` and `cargo audit` (if available)
- Flag outdated dependencies with known CVEs

#### 6. Frontend Security
- Search for `eval()`, `innerHTML`, `dangerouslySetInnerHTML`
- Check CSP configuration in `tauri.conf.json`
- Verify no sensitive data in localStorage (Zustand persist)

#### 7. Update Security
- Verify auto-updater uses signed releases
- Check update endpoint configuration

### Output

```
## Security Audit Report

### CRITICAL 🔴 (must fix immediately)
[findings]

### HIGH 🟠 (should fix before release)
[findings]

### MEDIUM 🟡 (fix when convenient)
[findings]

### LOW 🔵 (informational)
[findings]

### Summary
- Total: N findings (C critical, H high, M medium, L low)
- Risk level: LOW / MEDIUM / HIGH / CRITICAL
- Recommended actions: [prioritized list]
```
