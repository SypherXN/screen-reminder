# Screen Reminder

Cross-platform desktop app (Windows + macOS) that connects to Google Calendar, Microsoft Outlook, CalDAV, and Apple Calendar (Mac), then shows customizable animated icons on your screen when event reminders are due.

Built with **Tauri 2**, **React**, **TypeScript**, **Tailwind CSS**, and **SQLite**.

## Features

- **Calendar sources**: Google Calendar (OAuth), Microsoft 365 / Outlook (OAuth), CalDAV (iCloud, Fastmail, Nextcloud, etc.), Apple Calendar (macOS), Google Tasks, Microsoft To Do
- **Multiple accounts per provider**: Connect several Google or Microsoft accounts; each account can have its own overlay styling
- **Animated overlay**: Always-on-top transparent window with moving reminder icon and optional event title
- **Click-through overlay**: Clicks pass through to your desktop and apps while the reminder animates; hover the notification bubble to interact with it
- **Interactive reminders**: Click the bubble for dismiss, snooze, or open event; keyboard shortcuts `Esc` (dismiss), `S` (snooze), `O` (open URL)
- **Upcoming view**: List of scheduled reminders and events in the settings window
- **Customization**: Icon, size, animation path/speed, fonts, colors, auto-dismiss, chime sound, auto-contrast text, composition presets, per-account style overrides
- **Layer editor**: Preview notification bounds at the configured canvas size while editing overlay composition
- **System tray**: Run in background; open settings, pause reminders, or quit from the tray

## Recent updates

- **Click-through reminders** — overlay windows ignore mouse clicks by default so you can keep working; the bubble and menu become clickable when the cursor is over them
- **Calendar sync fixes** — correct parsing of Google/Outlook timed events (`dateTime`), no more false “due now” popups or missing Upcoming entries
- **Multi-account OAuth** — account picker on connect; duplicate accounts detected by email
- **Upcoming tab** — browse reminders and events sorted by time
- **OAuth reliability** — Windows URL handling fix, Google scopes/consent flow, background sync after connect (no app close on OAuth)
- **Build-time OAuth config** — Google/Microsoft client IDs and secrets are read from `.env` at compile time and embedded in the binary (no `.env` needed at runtime)
- **UI polish** — updated app icon, layer editor canvas border, account grouping in settings

## Prerequisites

- Node.js 20+
- Rust stable (`rustup`)
- Platform dependencies:
  - **macOS**: Xcode command line tools
  - **Windows**: WebView2 (usually preinstalled on Windows 10/11), Visual Studio Build Tools with **Desktop development with C++**
  - **Linux (dev only)**: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libdbus-1-dev`, `pkg-config`

## OAuth setup

Copy `.env.example` to `.env` in the project root and fill in credentials **before building**. Values are embedded at compile time via `src-tauri/build.rs` — end users do not need a `.env` file.

### Google Calendar

1. Create a project in [Google Cloud Console](https://console.cloud.google.com/)
2. Enable the **Google Calendar API** (and **Google Tasks API** for task sync)
3. Create an **OAuth 2.0 Client ID** (Desktop app)
4. Add authorized redirect URI: `http://127.0.0.1:PORT/callback` (loopback; the app picks a random port)
5. Set `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` in `.env`
6. If the OAuth app is in **Testing** mode, add your Google account under **Test users**

### Microsoft Outlook / 365

1. Register an app in [Azure Portal](https://portal.azure.com/) → App registrations
2. Add redirect URI: `http://127.0.0.1` (Mobile and desktop applications)
3. API permissions: `Calendars.Read`, `Tasks.Read` (for To Do), `User.Read`, `offline_access`
4. Set `MICROSOFT_CLIENT_ID` and `MICROSOFT_CLIENT_SECRET` in `.env`

### CalDAV

No OAuth keys required. Connect from the app with server URL, username, and password (stored in the OS keychain on Windows/macOS, or a restricted file in the app data directory on Linux dev builds).

### Apple Calendar (macOS only)

Grant Calendar access when prompted (System Settings → Privacy → Calendars). Events are read via Calendar.app automation on macOS.

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

**Important:** OAuth credentials must be present in `.env` when you run `tauri build`. Rebuild after changing `.env`.

**Step-by-step build instructions:** see [BUILD.md](BUILD.md).

## Code signing (release)

- **macOS**: Apple Developer ID + notarization (`codesign`, `notarytool`)
- **Windows**: Authenticode certificate for `.msi` / `.exe`

See [Tauri documentation](https://v2.tauri.app/distribute/sign/) for platform-specific signing steps.

## Project structure

```
src/                 React settings UI + overlay frontend
  components/        Settings panels, ReminderBubble, UpcomingPanel
  overlay/           Full-screen reminder overlay (click-through)
src-tauri/           Rust backend (sync, scheduler, storage, OAuth)
  src/calendar/      Google, Outlook, CalDAV, Apple providers
  src/scheduler.rs   Reminder polling + overlay dispatch
  src/storage.rs     SQLite persistence
  build.rs           Embeds .env OAuth config at compile time
```

## CI

GitHub Actions builds the frontend and runs `tauri build` on Ubuntu, Windows, and macOS (see `.github/workflows/ci.yml`).

## License

MIT
