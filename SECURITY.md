# Security Policy

## Supported Versions

Security updates are provided for the latest released version. The table below
shows which versions are currently receiving security patches.

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Do not open a public issue for security vulnerabilities.**

Instead, please report security vulnerabilities privately by email to:

**logic.yan@me.com**

Please include the following in your report:

- A clear description of the vulnerability
- Steps to reproduce, including code examples if applicable
- The affected version(s)
- Any potential impact or exploit scenario

### What to Expect

| Stage | Timeline |
|-------|----------|
| Initial acknowledgment | Within 48 hours |
| Confirmation of vulnerability | Within 5 business days |
| Patch release | Within 30 days (critical: 7 days) |

We will keep you informed of progress throughout the process and will
coordinate the public disclosure timeline with you.

## Security Considerations for Quantum Computing

MyQuat is a quantum computing simulation library. Security-relevant areas
include:

- **Serialization boundaries**: QASM import/export and serde deserialization
  should not panic or exhibit undefined behavior on untrusted input.
- **Cloud backend credentials**: API keys and cloud configuration files
  (`apikey.json`, `cloud_config.toml`) should never be committed to version
  control.
- **Numerical safety**: Floating-point operations in state vector simulation
  should be free of NaN propagation and division-by-zero panics.
- **Unsafe code**: Any `unsafe` blocks (SIMD, CUDA FFI) should be audited for
  memory safety and undefined behavior.

## Disclosure Policy

After a fix is released, we will publish a security advisory describing:

- The vulnerability and its impact
- The fix version
- Credit to the reporter (unless anonymity is requested)
- Any workarounds available before upgrading
