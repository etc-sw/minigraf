# Security Policy

## Supported Versions

Vicia DB is pre-1.0 and under active development. Nothing is published yet —
the `vicia-db` crate has not shipped a release — so only the current `main`
branch receives security fixes. Once `vicia-db` is on crates.io, the latest
published version joins `main`; older versions are not patched.

| Version | Supported |
| ------- | --------- |
| `main` | ✅ |
| latest published 0.x (once published) | ✅ |
| older 0.x    | ❌ |

Vulnerabilities in the published `minigraf` crate belong to
[upstream](https://github.com/project-minigraf/minigraf), not to this fork.

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Use GitHub's private vulnerability reporting instead:

> **[Report a vulnerability](https://github.com/etc-sw/vicia-db/security/advisories/new)**

Include as much of the following as possible:

- A description of the vulnerability and its potential impact
- The affected version(s)
- Steps to reproduce or a minimal proof-of-concept
- Any suggested mitigations, if known

## Response Timeline

This is a solo hobby project. I will make a best-effort response:

- **Acknowledgement**: within 7 days
- **Assessment**: within 14 days
- **Fix or mitigation**: timeline depends on severity and complexity

If a fix is warranted, it will be released as a patch version and a
[GitHub Security Advisory](https://github.com/etc-sw/vicia-db/security/advisories)
will be published.

## Scope

Vicia DB is an **embedded, single-file library** with no network surface.
Relevant security concerns include:

- Memory safety issues (e.g. unsound `unsafe` usage)
- Data corruption or loss due to incorrect ACID/WAL behaviour
- Malicious input via the Datalog query parser leading to unexpected behaviour
- File format vulnerabilities that could affect portability or integrity

Out of scope:

- Issues requiring physical access to the machine running the library
- Vulnerabilities in dependencies (report those upstream; we track them via
  `cargo audit` and Dependabot)
- Performance issues that do not have a security impact
