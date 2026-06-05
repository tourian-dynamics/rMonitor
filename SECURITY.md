# Security Policy

We take the security of `rMonitor` seriously. If you discover a vulnerability, please report it following this policy.

## Reporting a Vulnerability

**Please do not open a public issue for security-related bugs.**

Instead, please report security vulnerabilities privately by emailing the maintainers or project leads directly.

We will acknowledge receipt of your report within 48 hours and work with you to analyze and resolve the issue before making any public disclosure.

## Scope & Philosophy

*   **Registry & System Access:** `rMonitor` reads system metrics directly using standard APIs (or winreg under Windows). It does not execute raw command strings or start shell contexts with user input, protecting the host system from injection vectors.
*   **Minimal Permissions:** `rMonitor` does not require Administrator/UAC elevation to run. While running elevated grants access to additional system stats (such as restricted process-specific disk transfer rates), standard standard-user execution is fully supported and secure.
