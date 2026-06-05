import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ReminderBubble, reminderBubbleStyles } from "../components/ReminderBubble";
import { api } from "../lib/api";
import { playReminderChime } from "../lib/chime";
import type { OverlayPayload } from "../lib/types";

export function OverlayApp() {
  const [payload, setPayload] = useState<OverlayPayload | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const dismissTimer = useRef<number | null>(null);
  const payloadRef = useRef<OverlayPayload | null>(null);

  useEffect(() => {
    payloadRef.current = payload;
  }, [payload]);

  useEffect(() => {
    const unlistenPromise = listen<OverlayPayload>("show-reminder", (event) => {
      setPayload(event.payload);
      setMenuOpen(false);
      if (event.payload.play_sound) {
        playReminderChime();
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

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
    return <div className="overlay-root" />;
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

  const handleDismiss = async () => {
    if (payload.reminder_id === "preview") {
      await api.hideReminderOverlay();
    } else {
      await api.dismissReminder(payload.reminder_id);
    }
    setPayload(null);
    setMenuOpen(false);
  };

  const handleSnooze = async (minutes: number) => {
    if (payload.reminder_id === "preview") {
      await api.hideReminderOverlay();
    } else {
      await api.snoozeReminder(payload.reminder_id, minutes);
    }
    setPayload(null);
    setMenuOpen(false);
  };

  const handleSnoozeUntilStart = async () => {
    if (payload.reminder_id === "preview") {
      await api.hideReminderOverlay();
    } else {
      await api.snoozeReminderUntilStart(payload.reminder_id);
    }
    setPayload(null);
    setMenuOpen(false);
  };

  const handleOpen = async () => {
    if (payload.url) {
      await api.openReminderUrl(payload.url);
    }
    setMenuOpen(false);
  };

  return (
    <div className="overlay-root">
      <div
        className="overlay-track"
        style={{
          animation: `${animationName} ${duration}s linear infinite`,
        }}
      >
        <ReminderBubble
          settings={displaySettings}
          title={payload.title}
          location={payload.location}
          startTime={payload.start_time}
          onClick={() => setMenuOpen((open) => !open)}
        />
      </div>

      {menuOpen && (
        <div className="overlay-menu">
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
          <p className="mt-1 text-center text-xs text-slate-400">
            Esc dismiss · S snooze · O open
          </p>
        </div>
      )}

      <style>{`
        ${reminderBubbleStyles}
        .overlay-root {
          width: 100vw;
          height: 100vh;
          overflow: hidden;
          background: transparent;
          position: relative;
        }
        .overlay-track {
          position: absolute;
        }
        .overlay-menu {
          position: fixed;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 12px;
          background: rgba(15, 23, 42, 0.95);
          border: 1px solid rgba(148, 163, 184, 0.35);
          border-radius: 12px;
          z-index: 20;
        }
        .overlay-menu button {
          background: #1e293b;
          color: #e2e8f0;
          border: none;
          border-radius: 8px;
          padding: 8px 12px;
          cursor: pointer;
        }
        .overlay-menu button:hover {
          background: #334155;
        }
        @keyframes overlay-slide {
          0% { left: -20%; top: 50%; transform: translateY(-50%); }
          100% { left: 100%; top: 50%; transform: translateY(-50%); }
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
}
