import { setLayerVisibilityByType, syncPrimaryImageLayer } from "../lib/composition";
import { BUILTIN_ICONS, ANIMATION_PATHS, FONT_OPTIONS, type AppSettings } from "../lib/types";
import { IconPicker } from "./IconPicker";
import { FontPicker } from "./FontPicker";

interface Props {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}

export function AppearancePanel({ settings, onChange }: Props) {
  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    onChange({ ...settings, [key]: value });
  };

  return (
    <section className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
      <h2 className="mb-4 text-lg font-medium">Customization</h2>

      <div className="space-y-5">
        <IconPicker
          selectedId={settings.icon_id}
          onSelect={(iconId) => {
            onChange(
              syncPrimaryImageLayer({ ...settings, icon_id: iconId }, {
                icon_id: iconId,
                image_path: null,
              }),
            );
          }}
        />

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Custom icon path (PNG/SVG, optional)</span>
          <input
            value={settings.custom_icon_path ?? ""}
            onChange={(e) => {
              const image_path = e.target.value ? e.target.value : null;
              onChange(
                syncPrimaryImageLayer(
                  { ...settings, custom_icon_path: image_path },
                  { image_path },
                ),
              );
            }}
            placeholder="/path/to/icon.png"
            className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
          />
        </label>

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Icon size ({settings.icon_size}px)</span>
          <input
            type="range"
            min={24}
            max={128}
            value={settings.icon_size}
            onChange={(e) => {
              const width = Number(e.target.value);
              onChange(
                syncPrimaryImageLayer({ ...settings, icon_size: width }, { width }),
              );
            }}
            className="w-full"
          />
        </label>

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Animation</span>
          <select
            value={settings.animation_path}
            onChange={(e) => update("animation_path", e.target.value)}
            className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
          >
            {ANIMATION_PATHS.map((path) => (
              <option key={path.id} value={path.id}>
                {path.label}
              </option>
            ))}
          </select>
        </label>

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">
            Animation speed ({settings.animation_speed.toFixed(1)}x)
          </span>
          <input
            type="range"
            min={0.5}
            max={3}
            step={0.1}
            value={settings.animation_speed}
            onChange={(e) => update("animation_speed", Number(e.target.value))}
            className="w-full"
          />
        </label>

        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.show_title}
            onChange={(e) =>
              onChange(setLayerVisibilityByType(settings, "title", e.target.checked))
            }
          />
          Show event title on overlay
        </label>

        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.show_countdown}
            onChange={(e) =>
              onChange(setLayerVisibilityByType(settings, "countdown", e.target.checked))
            }
          />
          Show time until event
        </label>

        <FontPicker
          value={settings.font_family}
          options={FONT_OPTIONS}
          onChange={(font) => update("font_family", font)}
        />

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Font size ({settings.font_size}px)</span>
          <input
            type="range"
            min={12}
            max={32}
            value={settings.font_size}
            onChange={(e) => update("font_size", Number(e.target.value))}
            className="w-full"
          />
        </label>

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Font color</span>
          <input
            type="color"
            value={settings.font_color}
            onChange={(e) => update("font_color", e.target.value)}
            className="h-10 w-full cursor-pointer rounded-lg border border-slate-700 bg-slate-950"
          />
        </label>

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Snooze durations (minutes, comma-separated)</span>
          <input
            value={settings.snooze_durations.join(", ")}
            onChange={(e) =>
              update(
                "snooze_durations",
                e.target.value
                  .split(",")
                  .map((v) => Number(v.trim()))
                  .filter((n) => !Number.isNaN(n) && n > 0),
              )
            }
            className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
          />
        </label>

        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Auto-dismiss after (seconds, optional)</span>
          <input
            type="number"
            min={0}
            placeholder="Never"
            value={settings.auto_dismiss_seconds ?? ""}
            onChange={(e) =>
              update(
                "auto_dismiss_seconds",
                e.target.value ? Number(e.target.value) : null,
              )
            }
            className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
          />
        </label>

        <p className="text-xs text-slate-500">
          For multiple images and precise positioning, use the Advanced options layer editor.
        </p>
      </div>
    </section>
  );
}

export function iconEmoji(iconId: string): string {
  return BUILTIN_ICONS.find((icon) => icon.id === iconId)?.emoji ?? "🔔";
}

export function iconLabel(settings: AppSettings): string {
  if (settings.custom_icon_path) {
    return settings.custom_icon_path.split(/[/\\]/).pop() ?? "Custom";
  }
  return iconEmoji(settings.icon_id);
}
