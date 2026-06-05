# Screen Reminder — Setup Guide

This guide walks you through installing and running Screen Reminder on your own computer. The app is a background tray utility that syncs calendar reminders and shows animated overlays when events are due.

**Supported platforms for daily use:** Windows and macOS  
**Linux:** development and testing only (not a release target)

**Building installers for distribution:** see [BUILD.md](BUILD.md).

---

## Before you start (all platforms)

### 1. Get the project

```bash
git clone https://github.com/SypherXN/screen-reminder.git
cd screen-reminder
```

Or download and extract the repository ZIP, then open a terminal in that folder.

### 2. Calendar OAuth credentials (recommended)

Most people connect **Google Calendar** and/or **Outlook**. You need free API credentials — not an Apple Developer account.

Copy the example env file:

```bash
cp .env.example .env
```

Edit `.env` in the project root.

#### Google Calendar (+ optional Google Tasks)

1. Open [Google Cloud Console](https://console.cloud.google.com/)
2. Create a project (or pick an existing one)
3. Enable **Google Calendar API** (and **Google Tasks API** if you want tasks)
4. Go to **APIs & Services → Credentials → Create credentials → OAuth client ID**
5. Application type: **Desktop app**
6. Copy the **Client ID** and **Client secret** into `.env`:

   ```
   GOOGLE_CLIENT_ID=your-id.apps.googleusercontent.com
   GOOGLE_CLIENT_SECRET=your-secret
   ```

The app uses a **loopback redirect** (`http://127.0.0.1:<random-port>/callback`). You do not need to register a fixed port in Google Cloud for desktop OAuth.

#### Microsoft Outlook / To Do

1. Open [Azure Portal](https://portal.azure.com/) → **App registrations → New registration**
2. Supported account types: **Accounts in any organizational directory and personal Microsoft accounts**
3. Redirect URI: platform **Mobile and desktop applications**, URI `http://127.0.0.1`
4. Under **API permissions**, add delegated permissions:
   - `Calendars.Read`
   - `Tasks.Read` (only if you use Microsoft To Do)
   - `User.Read`
   - `offline_access`
5. Create a **client secret** under **Certificates & secrets**
6. Add to `.env`:

   ```
   MICROSOFT_CLIENT_ID=your-client-id
   MICROSOFT_CLIENT_SECRET=your-client-secret
   ```

#### CalDAV (no OAuth keys)

Use CalDAV for iCloud, Fastmail, Nextcloud, etc. Connect from the app with server URL, username, and password. Nothing to add in `.env`.

#### Push sync (optional, advanced)

Only needed for near-instant sync via webhooks. Set `PUSH_RELAY_URL` to your own HTTPS relay. If unset, the app uses normal polling (every 5 minutes, or every 1 minute when an event is soon). Most users can leave this blank.

---

## macOS

### Prerequisites

1. **macOS 10.15 or later** (see `tauri.conf.json` for the configured minimum)
2. **Xcode Command Line Tools**

   ```bash
   xcode-select --install
   ```

3. **Node.js 20+** — [nodejs.org](https://nodejs.org/) or [nvm](https://github.com/nvm-sh/nvm)

   ```bash
   node --version   # should be v20 or higher
   ```

4. **Rust**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   rustc --version
   ```

### Install and run (development)

```bash
npm install
npm run tauri dev
```

The settings window opens and the app sits in the **menu bar tray**. Connect calendars from the **Calendar accounts** tab.

### Build a standalone app

See **[BUILD.md](BUILD.md)** for full step-by-step instructions (Windows and macOS).

Quick command:

```bash
npm run tauri build
```

### First launch without an Apple Developer ID

You **do not** need a paid Apple Developer account to run the app on **your own Mac**.

If macOS says the app is from an “unidentified developer”:

1. **Right-click** `Screen Reminder.app` → **Open** → **Open**, or  
2. **System Settings → Privacy & Security → Open Anyway**

This is expected for locally built apps that are not notarized.

### macOS permissions

When prompted, allow:

| Permission | Why |
|------------|-----|
| **Calendars** | Apple Calendar integration (optional) |
| **Automation → Calendar** | Reading events via Calendar.app |
| **Accessibility** | May be requested for overlay / screen features depending on OS version |

Connect **Apple Calendar** only on Mac. Other sources (Google, Outlook, CalDAV) work the same as on Windows.

### Launch at login

Enable **Launch Screen Reminder at login** under **General** in settings. Works on macOS without a Developer ID for personal use; macOS may show a one-time approval for login items.

### macOS troubleshooting

| Problem | What to try |
|---------|-------------|
| `tauri dev` fails to compile | Run `xcode-select --install` again; restart terminal |
| Apple Calendar empty / errors | System Settings → Privacy → Calendars → enable for Screen Reminder; ensure Calendar.app has events |
| OAuth browser doesn’t return | Allow local network / localhost; disable VPN temporarily |
| Overlay not visible | Check **General → Show reminders on** monitor setting; ensure reminders aren’t paused |

---

## Windows

### Prerequisites

1. **Windows 10 or 11**
2. **WebView2** — usually preinstalled on Windows 11 and recent Windows 10. If the app fails to start, install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
3. **Node.js 20+** — [nodejs.org](https://nodejs.org/) (LTS installer is fine)
4. **Rust** — install from [rustup.rs](https://rustup.rs/) (use the default `x86_64-pc-windows-msvc` toolchain)
5. **Microsoft C++ Build Tools** — required for Rust on Windows. Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload

Open **PowerShell** or **Command Prompt** in the project folder (not WSL — build the Windows app natively on Windows).

### Install and run (development)

```powershell
npm install
npm run tauri dev
```

Use the system tray icon (near the clock) to open settings.

### Build a standalone installer

```powershell
npm run tauri build
```

Output:

- Executable: `src-tauri\target\release\screen-reminder.exe`
- Installer: `src-tauri\target\release\bundle\msi\` or `nsis\` (depends on Tauri bundle config)

### Windows permissions

- **OAuth**: A browser window opens for Google/Microsoft sign-in; allow the redirect back to `127.0.0.1`
- **Startup**: Enable **Launch at login** under **General** — adds a startup entry via the autostart plugin
- **SmartScreen**: Unsigned local builds may show “Windows protected your PC” → **More info → Run anyway**

### Windows troubleshooting

| Problem | What to try |
|---------|-------------|
| `link.exe` not found | Install Visual Studio Build Tools with C++ workload |
| WebView2 error | Install WebView2 Evergreen Runtime |
| Tray icon missing | Click the `^` arrow in the taskbar to show hidden icons |
| OAuth fails | Confirm `.env` values; redirect URI in Azure is `http://127.0.0.1` |

---

## Linux (development only)

Linux is useful for contributing or previewing the UI, but **overlay, tray, and some integrations are intended for Windows/macOS production use**. Build on a real Linux machine or VM — WSL often lacks the GUI libraries Tauri needs unless configured carefully.

### Prerequisites (Debian / Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libdbus-1-dev \
  pkg-config \
  build-essential \
  curl
```

Install **Node.js 20+** and **Rust** (`rustup`) as on macOS.

### Install and run

```bash
npm install
npm run tauri dev
```

### Limitations on Linux

- **Apple Calendar** — not available
- **Autostart plugin** — macOS/Windows only in this project
- **Keychain** — tokens may use file-based storage instead of the system keychain
- **Screen capture (auto-contrast)** — depends on `xcap` and your display server

For a daily-driver reminder app, use **Windows or macOS**.

---

## Using the app (all platforms)

After the app is running:

1. **Connect accounts** — **Calendar accounts** tab (Google, Outlook, Tasks, To Do, CalDAV, Apple on Mac)
2. **Customize appearance** — **Appearance** and **Advanced options** (layers, presets)
3. **General settings** — quiet hours, dedupe, monitor choice, sound, autostart
4. **Sync** — automatic after connect, on wake, every 5 min (or 1 min when an event is within 30 min); **Sync now** in the header
5. **Tray / menu bar** — Settings, Pause/Resume reminders, Quit
6. **Reminders** — animated overlay; click for dismiss, snooze, or open event; **Esc** / **S** / **O** keyboard shortcuts

Data is stored locally in SQLite under your user data directory (not in the repo).

---

## Quick reference

| Task | Command |
|------|---------|
| Install dependencies | `npm install` |
| Run in dev mode | `npm run tauri dev` |
| Build release | `npm run tauri build` |
| Frontend only (browser preview, mock data) | `npm run dev` → open http://127.0.0.1:1420 |

| Platform | Best for |
|----------|----------|
| macOS | Full features including Apple Calendar |
| Windows | Full features; simplest path without Apple accounts |
| Linux | Development / UI testing |

---

## Getting help

- OAuth errors: double-check `.env` and API permissions in Google Cloud / Azure
- Sync issues: **Accounts** tab shows per-account sync status and errors
- Build errors: confirm Node 20+, Rust stable, and platform prerequisites above

For signing and distributing to other users’ Macs, see [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/) (requires Apple Developer Program). For personal use, that is optional.
