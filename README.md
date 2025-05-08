# 🦀 Rusty Tools

**Rusty Tools** is a lightweight, powerful CLI toolkit written in Rust for automating interactions with the [BlueSky](https://bsky.app) API. Built with speed, safety, and precision in mind, this project helps you analyze, manage, and grow your presence on the AT Protocol with full control.

---

## 🔧 Features

- 🔐 Authenticate with your BlueSky account using app passwords
- 🚪 Logout and securely erase stored credentials
- 👤 Fetch and display your profile info
- 📥 Save a snapshot of your current followers in a local SQLite database
- 🔍 Compare follower snapshots to detect new followers and unfollowers
- 🕵️‍♂️ Look up any handle and retrieve public profile data
- 🤝 Follow all accounts that a given handle follows (mirror follows)
- 🧠 Intelligent CLI prompts and built-in safety checks
- 🔒 Local-first, no third-party dependencies for storage

---

## 🚀 Getting Started

### Prerequisites

- [Rust (latest stable)](https://www.rust-lang.org/tools/install)
- [SQLite3](https://www.sqlite.org/)
- A BlueSky account
- An app password for your BlueSky account (create one in your account settings)

### Install

```bash
git clone https://github.com/antoniwan/rusty-tools.git
cd rusty-tools
cargo build --release
```

### First Time Setup

1. Create an app password in your BlueSky account settings
2. Run the tool and use the app password when prompted
3. Your credentials will be securely stored locally
