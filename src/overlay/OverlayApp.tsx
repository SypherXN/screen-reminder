import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  cursorPosition,
  getCurrentWindow,
  type PhysicalPosition,
} from "@tauri-apps/api/window";
import { ReminderBubble, reminderBubbleStyles } from "../components/ReminderBubble";
import { api } from "../lib/api";
import { playReminderChime } from "../lib/chime";
import type { OverlayPayload } from "../lib/types";

export function OverlayApp() {
  const [payload, setPayload] = useState<OverlayPayload | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const dismissTimer = useRef<number | null>(null);
  const payloadRef = useRef<OverlayPayload | null>(null);
  const bubbleWrapRef = useRef<HTMLDivElement>(null);
  const clickThroughRef = useRef(true);

  useEffect(() => {
    payloadRef.current = payload;
  }, [payload]);

  useEffect(() => {
    const unlistenShow = listen<OverlayPayload>("show-reminder", (event) => {
      setPayload(event.payload);
      setMenuOpen(false);
      if (event.payload.play_sound) {
        playReminderChime();
      }
    });
    const unlistenHide = listen("hide-reminder", () => {
      setPayload(null);
      setMenuOpen(false);
    });
    return () => {
      unlistenShow.then((unlisten) => unlisten());
      unlistenHide.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    const win = getCurrentWindow();
    if (!payload) {
      win.hide().catch(console.error);
      return;
    }
    const frame = requestAnimationFrame(() => {
      win.setIgnoreCursorEvents(true).catch(console.error);
      clickThroughRef.current = true;
      win.show().catch(console.error);
    });
    return () => cancelAnimationFrame(frame);
  }, [payload]);

  useEffect(() => {
    if (!payload) {
      return;
    }

    const win = getCurrentWindow();
    let cancelled = false;
    let pollTimer: number | null = null;
    let windowPos: PhysicalPosition | null = null;
    let scaleFactor = 1;

    const setClickThrough = async (ignore: boolean) => {
      if (cancelled || clickThroughRef.current === ignore) {
        return;
      }
      clickThroughRef.current = ignore;
      await win.setIgnoreCursorEvents(ignore);
    };

    const syncHitTest = async () => {
      const wrap = bubbleWrapRef.current;
      if (!wrap || !windowPos || cancelled) {
        return;
      }

      const rect = wrap.getBoundingClientRect();
      const cursor = await cursorPosition();
      const left = windowPos.x + rect.left * scaleFactor;
      const top = windowPos.y + rect.top * scaleFactor;
      const right = left + rect.width * scaleFactor;
      const bottom = top + rect.height * scaleFactor;
      const overInteractive =
        menuOpen ||
        (cursor.x >= left &&
          cursor.x <= right &&
          cursor.y >= top &&
          cursor.y <= bottom);

      await setClickThrough(!overInteractive);
    };

    void (async () => {
      try {
        [windowPos, scaleFactor] = await Promise.all([
          win.outerPosition(),
          win.scaleFactor(),
        ]);
        await setClickThrough(true);
        pollTimer = window.setInterval(() => {
          syncHitTest().catch(console.error);
        }, 32);
      } catch (error) {
        console.error(error);
      }
    })();

    return () => {
      cancelled = true;
      if (pollTimer !== null) {
        window.clearInterval(pollTimer);
      }
      win.setIgnoreCursorEvents(false).catch(console.error);
      clickThroughRef.current = false;
    };
  }, [payload, menuOpen]);

  useEffect(() => {
    if (dismissTimer.current) {
      window.clearTimeout(dismissTimer.current);
      dismissTimer.current = null;
    }
    if (payload?.settings.auto_dismiss_seconds) {
      dismissTimer.current = window.setTimeout(() => {
        if (payload.reminder_id !== "preview") {
          api.dismissReminder(payload.reminder_id).catch(console.error);
        } else {
          api.hideReminderOverlay().catch(console.error);
        }
      }, payload.settings.auto_dismiss_seconds * 1000);
    }
  }, [payload]);

  useEffect(() => {
    const onKeyDown = async (event: KeyboardEvent) => {
      const current = payloadRef.current;
      if (!current) {
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        if (current.reminder_id === "preview") {
          await api.hideReminderOverlay();
        } else {
          await api.dismissReminder(current.reminder_id);
        }
        setPayload(null);
        setMenuOpen(false);
      } else if (event.key.toLowerCase() === "s") {
        event.preventDefault();
        const minutes = current.settings.snooze_durations[0] ?? 5;
        if (current.reminder_id === "preview") {
          await api.hideReminderOverlay();
        } else {
          await api.snoozeReminder(current.reminder_id, minutes);
        }
        setPayload(null);
        setMenuOpen(false);
      } else if (event.key.toLowerCase() === "o" && current.url) {
        event.preventDefault();
        await api.openReminderUrl(current.url);
        setMenuOpen(false);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  if (!payload) {
    return null;
  }

  const { settings } = payload;
  const displaySettings = payload.effective_font_color
    ? { ...settings, font_color: payload.effective_font_color }
    : settings;
  const duration = 8 / settings.animation_speed;
  const animationName =
    settings.animation_path === "bounce"
      ? "overlay-bounce"
      : settings.animation_path === "figure_eight"
        ? "overlay-figure-eight"
        : settings.animation_path === "random"
          ? "overlay-random"
          : "overlay-slide";

  const closeMenu = () => setMenuOpen(false);

  const handleDismiss = async () => {
    if (payload.reminder_id === "preview") {
      await api.hideReminderOverlay();
    } else {
      await api.dismissReminder(payload.reminder_id);
    }
    setPayload(null);
    closeMenu();
  };

  const handleSnooze = async (minutes: number) => {
    if (payload.reminder_id === "preview") {
      await api.hideReminderOverlay();
    } else {
      await api.snoozeReminder(payload.reminder_id, minutes);
    }
    setPayload(null);
    closeMenu();
  };

  const handleSnoozeUntilStart = async () => {
    if (payload.reminder_id === "preview") {
      await api.hideReminderOverlay();
    } else {
      await api.snoozeReminderUntilStart(payload.reminder_id);
    }
    setPayload(null);
    closeMenu();
  };

  const handleOpen = async () => {
    if (payload.url) {
      await api.openReminderUrl(payload.url);
    }
    closeMenu();
  };

  return (
    <div className="overlay-root">
      <div
        className="overlay-track"
        style={{
          animation: `${animationName} ${duration}s linear infinite`,
        }}
      >
        <div className="overlay-bubble-wrap" ref={bubbleWrapRef}>
          <ReminderBubble
            settings={displaySettings}
            title={payload.title}
            location={payload.location}
            startTime={payload.start_time}
            allowOverflow
            onClick={() => setMenuOpen((open) => !open)}
          />

          {menuOpen && (
            <div className="overlay-menu" onClick={(event) => event.stopPropagation()}>
              <div className="overlay-menu__actions">
                <button type="button" onClick={handleDismiss}>
                  Dismiss
                </button>
                <button type="button" onClick={handleSnoozeUntilStart}>
                  At event start
                </button>
                {settings.snooze_durations.map((minutes) => (
                  <button key={minutes} type="button" onClick={() => handleSnooze(minutes)}>
                    Snooze {minutes}m
                  </button>
                ))}
                {payload.url && (
                  <button type="button" onClick={handleOpen}>
                    Open event
                  </button>
                )}
              </div>
              <div className="overlay-menu__shortcuts">
                <span>
                  <kbd>Esc</kbd> dismiss
                </span>
                <span>
                  <kbd>S</kbd> snooze
                </span>
                {payload.url && (
                  <span>
                    <kbd>O</kbd> open
                  </span>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      <style>{`
        ${reminderBubbleStyles}
        html, body, #root {
          margin: 0;
          width: 100%;
          height: 100%;
          overflow: hidden;
          background: transparent !important;
        }
        .overlay-root {
          width: 100vw;
          height: 100vh;
          overflow: hidden;
          background: transparent;
          position: relative;
          pointer-events: none;
        }
        .overlay-track {
          position: absolute;
          pointer-events: none;
        }
        .overlay-bubble-wrap {
          position: relative;
          pointer-events: auto;
        }
        .overlay-menu {
          position: absolute;
          top: calc(100% + 8px);
          left: 50%;
          transform: translateX(-50%);
          min-width: 180px;
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 10px;
          background: rgba(15, 23, 42, 0.94);
          border: 1px solid rgba(148, 163, 184, 0.3);
          border-radius: 10px;
          box-shadow: 0 8px 24px rgba(0, 0, 0, 0.28);
        }
        .overlay-menu__actions {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }
        .overlay-menu button {
          background: #1e293b;
          color: #e2e8f0;
          border: 1px solid rgba(148, 163, 184, 0.12);
          border-radius: 6px;
          padding: 6px 8px;
          font-size: 12px;
          text-align: left;
          cursor: pointer;
          white-space: nowrap;
        }
        .overlay-menu button:hover {
          background: #334155;
        }
        .overlay-menu__shortcuts {
          display: flex;
          flex-wrap: wrap;
          gap: 6px 10px;
          padding-top: 6px;
          border-top: 1px solid rgba(148, 163, 184, 0.18);
          font-size: 11px;
          color: #94a3b8;
        }
        .overlay-menu__shortcuts span {
          display: inline-flex;
          align-items: center;
          gap: 4px;
        }
        .overlay-menu kbd {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          min-width: 22px;
          padding: 1px 5px;
          border-radius: 4px;
          border: 1px solid rgba(148, 163, 184, 0.35);
          background: #0f172a;
          color: #f8fafc;
          font-size: 10px;
          font-family: inherit;
        }
        @keyframes overlay-slide {
          0% { left: 0%; top: 50%; transform: translateY(-50%); }
          100% { left: 100%; top: 50%; transform: translate(-100%, -50%); }
        }
        @keyframes overlay-bounce {
          0%, 100% { left: 8%; top: 18%; }
          50% { left: 72%; top: 72%; }
        }
        @keyframes overlay-figure-eight {
          0% { left: 50%; top: 12%; transform: translateX(-50%); }
          25% { left: 82%; top: 50%; }
          50% { left: 50%; top: 78%; transform: translateX(-50%); }
          75% { left: 18%; top: 50%; }
          100% { left: 50%; top: 12%; transform: translateX(-50%); }
        }
        @keyframes overlay-random {
          0% { left: 6%; top: 28%; }
          33% { left: 62%; top: 12%; }
          66% { left: 28%; top: 68%; }
          100% { left: 84%; top: 38%; }
        }
      `}</style>
    </div>
  );
}

export async function initOverlayWindow() {
  const window = getCurrentWindow();
  await window.setDecorations(false);
  await window.setSkipTaskbar(true);
  await window.setAlwaysOnTop(true);
  await window.setIgnoreCursorEvents(true);
  await window.hide();
}
