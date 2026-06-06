# ⚙️ Valo-Core Webhook Relayer

The secure, high-throughput asynchronous backend engine for the Valo-Core ecosystem. Built with Rust and the Axum web framework, this relayer acts as the cryptographic gateway that listens for GitHub webhook event payloads, validates their signatures, and extracts contribution data to interface with the core protocol smart contracts.

## 🗺️ Ecosystem System Architecture

                   +---------------------------+
                   |   GitHub Webhook Events   |
                   +-------------+-------------+
                                 |  (Raw JSON Payload)
                                 v
                   +-------------+-------------+
                   |      valocore-relayer     |  <-- (You are here)
                   |  [HMAC-SHA256 Validation] |
                   +-------------+-------------+
                                 |
                                 v  (Parsed Milestone IDs)
+--------------------------+   +-----+-----+   +---------------------------+
|    valocore-contracts    |-->| Smart State |<--|     valocore-dashboard    |
| (Escrow & Checked Math)  |   +-----------+   |  (Obsidian & Gold Engine) |
+--------------------------+                   +---------------------------+


---

## 🔒 Security & Verification Layer

- **Cryptographic Guard:** HMAC-SHA256 signature verification utilizing constant-time comparison (`constant_time_eq`) to eliminate timing-attack vectors on incoming GitHub payloads.
- **Payload Parsing:** Asynchronous parsing using a tokio-backed Axum pipeline with robust regular expression token isolation for developer milestones.

## 🛠️ Local Development Setup

### Prerequisites
Ensure you have the latest stable Rust toolchain installed (`cargo`, `rustc`).

### Compilation & Checks
Verify the abstract syntax tree and ensure the codebase compiles cleanly:
```powershell
cargo check
