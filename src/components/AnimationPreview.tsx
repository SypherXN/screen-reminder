import { ReminderBubble, reminderBubbleStyles } from "./ReminderBubble";
import type { AppSettings } from "../lib/types";
import { previewStartTime } from "../lib/timeUntil";

interface Props {
  settings: AppSettings;
  onPreview: () => void;
}

export function AnimationPreview({ settings, onPreview }: Props) {
  const duration = 6 / settings.animation_speed;
  const animationName =
    settings.animation_path === "bounce"
      ? "preview-bounce"
      : settings.animation_path === "figure_eight"
        ? "preview-figure-eight"
        : settings.animation_path === "random"
          ? "preview-random"
          : "preview-slide";

  return (
    <section className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-lg font-medium">Live preview</h2>
        <button
          onClick={onPreview}
          className="rounded-lg bg-indigo-600 px-3 py-2 text-sm hover:bg-indigo-500"
        >
          Show on screen
        </button>
      </div>

      <div className="relative h-44 overflow-hidden rounded-lg bg-slate-950">
        <div
          className="absolute"
          style={{
            animation: `${animationName} ${duration}s linear infinite`,
          }}
        >
          <ReminderBubble
            settings={settings}
            title="Upcoming meeting"
            startTime={previewStartTime(15)}
            interactive={false}
          />
        </div>
      </div>

      <style>{`
        ${reminderBubbleStyles}
        @keyframes preview-slide {
          0% { left: -20%; top: 50%; transform: translateY(-50%); }
          100% { left: 100%; top: 50%; transform: translateY(-50%); }
        }
        @keyframes preview-bounce {
          0%, 100% { left: 10%; top: 20%; }
          50% { left: 70%; top: 70%; }
        }
        @keyframes preview-figure-eight {
          0% { left: 50%; top: 10%; transform: translateX(-50%); }
          25% { left: 80%; top: 50%; }
          50% { left: 50%; top: 80%; transform: translateX(-50%); }
          75% { left: 20%; top: 50%; }
          100% { left: 50%; top: 10%; transform: translateX(-50%); }
        }
        @keyframes preview-random {
          0% { left: 5%; top: 30%; }
          33% { left: 60%; top: 10%; }
          66% { left: 30%; top: 70%; }
          100% { left: 85%; top: 40%; }
        }
      `}</style>
    </section>
  );
}
