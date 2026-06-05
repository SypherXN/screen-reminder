# Screen Reminder

Cross-platform desktop app (Windows + macOS) that connects to Google Calendar, Microsoft Outlook, CalDAV, and Apple Calendar (Mac), then shows customizable animated icons on your screen when event reminders are due.

Built with **Tauri 2**, **React**, **TypeScript**, **Tailwind CSS**, and **SQLite**.

## Features

- **Calendar sources**: Google Calendar (OAuth), Microsoft 365 / Outlook (OAuth), CalDAV (iCloud, Fastmail, Nextcloud, etc.), Apple Calendar (macOS)
- **Animated overlay**: Always-on-top transparent window with moving reminder icon and optional event title
- **Interactive reminders**: Click to dismiss, snooze (configurable durations), or open the event URL
- **Customization**: Icon, size, animation path/speed, fonts, colors, auto-dismiss
- **System tray**: Run in background; open settings, pause reminders, or quit from the tray

## Prerequisites

- Node.js 20+
- Rust stable (`rustup`)
- Platform dependencies:
  - **macOS**: Xcode command line tools
  - **Windows**: WebView2 (usually preinstalled on Windows 10/11)
  - **Linux (dev only)**: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libdbus-1-dev`, `pkg-config`

## OAuth setup

Copy `.env.example` to `.env` in the project root and fill in credentials.

### Google Calendar

1. Create a project in [Google Cloud Console](https://console.cloud.google.com/)
2. Enable the **Google Calendar API**
3. Create an **OAuth 2.0 Client ID** (Desktop app)
4. Add authorized redirect URI: `http://127.0.0.1:PORT/callback` (loopback; the app picks a random port)
5. Set `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` in `.env`

### Microsoft Outlook / 365

1. Register an app in [Azure Portal](https://portal.azure.com/) → App registrations
2. Add redirect URI: `http://127.0.0.1` (Mobile and desktop applications)
3. API permissions: `Calendars.Read`, `Tasks.Read` (for To Do), `User.Read`, `offline_access`
4. Set `MICROSOFT_CLIENT_ID` and `MICROSOFT_CLIENT_SECRET` in `.env`

### CalDAV

No OAuth keys required. Connect from the app with server URL, username, and password (stored in the OS keychain on Windows/macOS, or a restricted file in the app data directory on Linux dev builds).

### Apple Calendar (macOS only)

Grant Calendar access when prompted (System Settings → Privacy → Calendars). Events are read via Calendar.app automation on macOS.

### Google Tasks / Microsoft To Do

Enable **Google Tasks API** in Google Cloud for task sync. For Microsoft To Do, add **`Tasks.Read`** to Azure API permissions (in addition to `Calendars.Read`).

### Push sync (optional)

Set `PUSH_RELAY_URL` in `.env` to an HTTPS relay that receives Google/Microsoft webhooks and exposes `GET /poll/{device_id}`. Enable in **General** settings.

## Development

```bash
npm install
cp .env.example .env   # add your OAuth credentials
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Installers are written to `src-tauri/target/release/bundle/`.

**Step-by-step build instructions:** see [BUILD.md](BUILD.md).

## Code signing (release)

- **macOS**: Apple Developer ID + notarization (`codesign`, `notarytool`)
- **Windows**: Authenticode certificate for `.msi` / `.exe`

See [Tauri documentation](https://v2.tauri.app/distribute/sign/) for platform-specific signing steps.

## Project structure

```
src/                 React settings UI + overlay frontend
src-tauri/           Rust backend (sync, scheduler, storage, OAuth)
  src/calendar/      Google, Outlook, CalDAV, Apple providers
  src/scheduler.rs   Reminder polling + overlay dispatch
  src/storage.rs     SQLite persistence
```

## CI

GitHub Actions builds the frontend and runs `tauri build` on Ubuntu, Windows, and macOS (see `.github/workflows/ci.yml`).

## License

MIT
