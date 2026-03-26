# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in shrug, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, use [GitHub's private vulnerability reporting](https://github.com/mfassaie/shrug/security/advisories/new) or email **mehdi@falkonr.com**.

Please include:

- A description of the vulnerability
- Steps to reproduce
- The potential impact
- Any suggested fix (if you have one)

You should receive a response within 48 hours. We will work with you to understand the issue and coordinate a fix before any public disclosure.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.5.x   | Yes       |
| < 0.5   | No        |

## Credential Safety

shrug stores credentials in the OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) or in an AES-256-GCM encrypted file. Credentials are never stored in plaintext. The `--trace` flag automatically masks tokens and secrets in diagnostic output.
