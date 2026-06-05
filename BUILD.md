# Screen Reminder — Build Guide

This guide explains how to build installable **Screen Reminder** apps for Windows and macOS. You must build on each platform separately — a Mac build cannot run on Windows and vice versa.

For first-time machine setup (Node, Rust, OAuth keys), see [SETUP.md](SETUP.md).

---

## Overview

| Step | What happens |
|------|----------------|
| `npm install` | Installs frontend and Tauri CLI dependencies |
| `npm run tauri build` | Compiles Rust, builds the UI, creates installers |
| Output | `.msi` / `.exe` (Windows) or `.app` / `.dmg` (macOS) |

Bundling is already enabled in `src-tauri/tauri.conf.json` (`bundle.active: true`).

---

## Part 1 — Shared preparation

### 1. Get the project

```bash
git clone https://github.com/SypherXN/screen-reminder.git
cd screen-reminder
```

Or download the ZIP from GitHub, extract it, and open a terminal in that folder.

### 2. Configure OAuth (for Google / Outlook)

Google and Outlook sign-in read credentials from a `.env` file in the project root when the app starts (and when you build).

**macOS / Linux:**

```bash
cp .env.example .env
```

**Windows PowerShell:**

```powershell
Copy-Item .env.example .env
```

Edit `.env` and set the values you need:

```env
GOOGLE_CLIENT_ID=your-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-secret

MICROSOFT_CLIENT_ID=your-client-id
MICROSOFT_CLIENT_SECRET=your-client-secret
```

- **CalDAV only:** leave OAuth fields empty — no keys required.
- **Push sync:** optional; leave `PUSH_RELAY_URL` blank unless you host a relay.

See [SETUP.md](SETUP.md) for how to create Google Cloud and Azure credentials.

### 3. Install Node dependencies

```bash
npm install
```

---

## Part 2 — Build on Windows

Use a **native Windows** shell (PowerShell or Command Prompt). Building inside WSL is not recommended for producing Windows installers.

### Prerequisites

| Tool | Link / notes |
|------|----------------|
| Node.js 20+ | https://nodejs.org/ |
| Rust | https://rustup.rs/ |
| Visual Studio Build Tools | https://visualstudio.microsoft.com/visual-cpp-build-tools/ — install **Desktop development with C++** |
| WebView2 Runtime | Usually preinstalled on Windows 10/11; [download](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if the app fails to start |

Verify:

```powershell
node --version
rustc --version
```

### Optional: test before building

```powershell
npm run tauri dev
```

If the app opens and the tray icon appears, proceed to the release build. Exit the app when finished.

### Release build

```powershell
npm run tauri build
```

The first build often takes **10–20+ minutes** while Rust compiles dependencies. Later builds are faster.

### Output location

```
src-tauri\target\release\bundle\
```

| Artifact | Typical path | Use |
|----------|--------------|-----|
| MSI installer | `bundle\msi\*.msi` | **Best for sharing** with other Windows users |
| NSIS installer | `bundle\nsis\*.exe` | Alternative installer |
| Executable only | `target\release\screen-reminder.exe` | Direct run, no installer |

### Install and test

1. Run the `.msi` (or NSIS `.exe`) and complete setup.
2. If **SmartScreen** appears (unsigned app): choose **More info → Run anyway**.
3. Find **Screen Reminder** in the system tray (near the clock).
4. Open settings and connect a calendar.

---

## Part 3 — Build on macOS

Use a Mac running **macOS 10.15 or later**.

### Prerequisites

| Tool | Install |
|------|---------|
| Xcode Command Line Tools | `xcode-select --install` |
| Node.js 20+ | https://nodejs.org/ or [nvm](https://github.com/nvm-sh/nvm) |
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` then `source "$HOME/.cargo/env"` |

Verify:

```bash
node --version
rustc --version
```

### Optional: test before building

```bash
npm run tauri dev
```

### Release build

```bash
npm run tauri build
```

### Output location

```
src-tauri/target/release/bundle/macos/
```

| Artifact | Use |
|----------|-----|
| `Screen Reminder.app` | macOS application bundle |
| `Screen Reminder_*.dmg` | Disk image for distribution (when generated) |

Share the **`.dmg`**, or zip **`Screen Reminder.app`** for others to copy into Applications.

### Install and test

1. Open the `.dmg` and drag the app to **Applications**, or run the `.app` directly.
2. If macOS blocks an unsigned app: **Right-click → Open → Open**, or **System Settings → Privacy & Security → Open Anyway**.
3. Grant **Calendars** access if using Apple Calendar.
4. Use the **menu bar** tray icon to open settings.

**Apple Developer ID:** Not required to run on your own Mac. Optional for notarization when distributing to many users without security warnings.

---

## Part 4 — Build checklist

```
[ ] Clone or download the repository
[ ] Copy .env.example → .env and add OAuth keys (if using Google/Outlook)
[ ] npm install
[ ] npm run tauri dev          (optional smoke test)
[ ] npm run tauri build
[ ] Locate artifacts under src-tauri/target/release/bundle/
[ ] Install on a test machine and connect a calendar
```

---

## Command reference

```bash
npm install              # Install dependencies (once per clone)
npm run tauri dev        # Development mode with hot reload
npm run tauri build      # Production build + installers
npm run build            # Frontend only (no desktop app)
```

---

## Troubleshooting

| Problem | What to try |
|---------|-------------|
| `GOOGLE_CLIENT_ID not set` when connecting Google | Create/fix `.env`, then **rebuild** |
| Windows: `link.exe` not found | Install Visual Studio Build Tools with C++ workload |
| Windows: WebView2 error | Install [WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| macOS: compile errors | Re-run `xcode-select --install`, restart terminal |
| macOS: “unidentified developer” | Right-click app → Open (expected without notarization) |
| Build very slow | Normal on first run; Rust compiles many crates |
| Built on WSL | Use native Windows or Mac for release installers |

---

## Distributing to others

1. Build on **Windows** → upload the `.msi` (or NSIS `.exe`).
2. Build on **macOS** → upload the `.dmg` or `.app`.
3. Host on **GitHub Releases**, your website, or file share.
4. Tell users they may see a **one-time security prompt** unless you sign the app:
   - **macOS:** Apple Developer ID + notarization — [Tauri docs](https://v2.tauri.app/distribute/sign/macos/)
   - **Windows:** Authenticode certificate — [Tauri docs](https://v2.tauri.app/distribute/sign/windows/)

End users do **not** need Node.js or Rust — only the installer you provide. They still connect their own Google/Microsoft/CalDAV accounts inside the app.

---

## CI builds

GitHub Actions runs `npm run tauri build` on Ubuntu, Windows, and macOS (see `.github/workflows/ci.yml`). You can extend that workflow to attach `bundle/` artifacts to GitHub Releases for automated builds.
