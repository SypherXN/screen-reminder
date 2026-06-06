import { useEffect, useState, type CSSProperties } from "react";
import {
  addLayerToComposition,
  createDefaultComposition,
  moveLayerInComposition,
  removeLayerFromComposition,
  sortedLayers,
  updateComposition,
  updateLayerInComposition,
} from "../lib/composition";
import { api } from "../lib/api";
import { previewStartTime } from "../lib/timeUntil";
import {
  BUILTIN_ICONS,
  LAYER_TYPE_LABELS,
  type AppSettings,
  type CompositionPreset,
  type LayerType,
  type OverlayLayer,
} from "../lib/types";
import { ReminderBubble, reminderBubbleStyles } from "./ReminderBubble";

interface Props {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}

export function AdvancedOptionsPanel({ settings, onChange }: Props) {
  const [selectedLayerId, setSelectedLayerId] = useState<string | null>(null);
  const [presets, setPresets] = useState<CompositionPreset[]>([]);
  const [presetName, setPresetName] = useState("");
  const composition = settings.composition;
  const selectedLayer =
    composition.layers.find((layer) => layer.id === selectedLayerId) ?? null;
  const orderedLayers = sortedLayers(composition);

  useEffect(() => {
    api.listCompositionPresets().then(setPresets).catch(() => setPresets([]));
  }, []);

  const refreshPresets = () => {
    api.listCompositionPresets().then(setPresets).catch(() => setPresets([]));
  };

  const updateComp = (next: typeof composition) => {
    onChange(updateComposition(settings, next));
  };

  const updateLayer = (layerId: string, patch: Partial<OverlayLayer>) => {
    updateComp(updateLayerInComposition(composition, layerId, patch));
  };

  const addLayer = (type: LayerType) => {
    const next = addLayerToComposition(composition, type);
    updateComp(next);
    const layers = sortedLayers(next);
    const created = layers[layers.length - 1];
    if (created) {
      setSelectedLayerId(created.id);
    }
  };

  return (
    <section className="space-y-6">
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-2 text-lg font-medium">Composition presets</h2>
        <p className="mb-4 text-sm text-slate-400">
          Save and load named layer layouts for quick switching.
        </p>
        <div className="flex flex-wrap gap-2">
          <input
            value={presetName}
            onChange={(e) => setPresetName(e.target.value)}
            placeholder="Preset name"
            className="min-w-[180px] flex-1 rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm"
          />
          <button
            type="button"
            disabled={!presetName.trim()}
            onClick={async () => {
              await api.saveCompositionPreset(presetName.trim());
              setPresetName("");
              refreshPresets();
            }}
            className="rounded-lg bg-indigo-600 px-3 py-2 text-sm hover:bg-indigo-500 disabled:opacity-50"
          >
            Save current
          </button>
        </div>
        {presets.length > 0 && (
          <ul className="mt-4 space-y-2">
            {presets.map((preset) => (
              <li
                key={preset.id}
                className="flex items-center justify-between gap-3 rounded-lg border border-slate-800 bg-slate-950 px-3 py-2"
              >
                <span className="text-sm">{preset.name}</span>
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={async () => {
                      const next = await api.loadCompositionPreset(preset.id);
                      onChange(next);
                    }}
                    className="rounded-lg bg-slate-800 px-2 py-1 text-xs hover:bg-slate-700"
                  >
                    Load
                  </button>
                  <button
                    type="button"
                    onClick={async () => {
                      await api.deleteCompositionPreset(preset.id);
                      refreshPresets();
                    }}
                    className="rounded-lg bg-red-900/40 px-2 py-1 text-xs text-red-200 hover:bg-red-900/70"
                  >
                    Delete
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-2 text-lg font-medium">Layer editor</h2>
        <p className="mb-4 text-sm text-slate-400">
          Build your reminder from layers — add multiple images, text, titles, and countdowns.
          The overlay has no background by default; only your layers are visible.
        </p>

        <div className="grid gap-6 lg:grid-cols-[240px_minmax(0,1fr)]">
          <aside className="space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-slate-300">Layers</h3>
              <span className="text-xs text-slate-500">{orderedLayers.length}</span>
            </div>

            <ul className="space-y-2">
              {[...orderedLayers].reverse().map((layer) => (
                <li
                  key={layer.id}
                  className={`rounded-lg border px-3 py-2 ${
                    selectedLayerId === layer.id
                      ? "border-indigo-500 bg-indigo-950/40"
                      : "border-slate-800 bg-slate-950"
                  }`}
                >
                  <button
                    type="button"
                    className="flex w-full items-center gap-2 text-left text-sm"
                    onClick={() => setSelectedLayerId(layer.id)}
                  >
                    <span
                      className={`h-2 w-2 rounded-full ${
                        layer.visible ? "bg-emerald-400" : "bg-slate-600"
                      }`}
                    />
                    <span className="flex-1 truncate">{layer.name}</span>
                    <span className="text-xs text-slate-500">{LAYER_TYPE_LABELS[layer.type]}</span>
                  </button>
                  <div className="mt-2 flex gap-1">
                    <button
                      type="button"
                      title="Toggle visibility"
                      className="rounded bg-slate-800 px-2 py-1 text-xs hover:bg-slate-700"
                      onClick={() => updateLayer(layer.id, { visible: !layer.visible })}
                    >
                      {layer.visible ? "Hide" : "Show"}
                    </button>
                    <button
                      type="button"
                      title="Move up (front)"
                      className="rounded bg-slate-800 px-2 py-1 text-xs hover:bg-slate-700"
                      onClick={() => updateComp(moveLayerInComposition(composition, layer.id, "up"))}
                    >
                      ↑
                    </button>
                    <button
                      type="button"
                      title="Move down (back)"
                      className="rounded bg-slate-800 px-2 py-1 text-xs hover:bg-slate-700"
                      onClick={() => updateComp(moveLayerInComposition(composition, layer.id, "down"))}
                    >
                      ↓
                    </button>
                    <button
                      type="button"
                      title="Delete layer"
                      className="rounded bg-red-950/50 px-2 py-1 text-xs text-red-200 hover:bg-red-900/50"
                      onClick={() => {
                        updateComp(removeLayerFromComposition(composition, layer.id));
                        if (selectedLayerId === layer.id) {
                          setSelectedLayerId(null);
                        }
                      }}
                    >
                      ✕
                    </button>
                  </div>
                </li>
              ))}
            </ul>

            <div className="grid grid-cols-2 gap-2">
              {(["image", "text", "title", "countdown"] as LayerType[]).map((type) => (
                <button
                  key={type}
                  type="button"
                  onClick={() => addLayer(type)}
                  className="rounded-lg bg-slate-800 px-2 py-2 text-xs hover:bg-slate-700"
                >
                  + {LAYER_TYPE_LABELS[type]}
                </button>
              ))}
            </div>
          </aside>

          <div className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <label className="block text-sm">
                <span className="mb-2 block text-slate-400">
                  Canvas width ({composition.canvas_width}px)
                </span>
                <input
                  type="range"
                  min={120}
                  max={640}
                  value={composition.canvas_width}
                  onChange={(e) =>
                    updateComp({ ...composition, canvas_width: Number(e.target.value) })
                  }
                  className="w-full"
                />
              </label>
              <label className="block text-sm">
                <span className="mb-2 block text-slate-400">
                  Canvas height ({composition.canvas_height}px)
                </span>
                <input
                  type="range"
                  min={48}
                  max={240}
                  value={composition.canvas_height}
                  onChange={(e) =>
                    updateComp({ ...composition, canvas_height: Number(e.target.value) })
                  }
                  className="w-full"
                />
              </label>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs text-slate-500">
                <span>Notification bounds — content outside the border is clipped on screen</span>
                <span className="font-mono text-slate-400">
                  {composition.canvas_width} × {composition.canvas_height}px
                </span>
              </div>

              <div
                className="layer-editor-workspace"
                style={
                  {
                    "--canvas-width": `${composition.canvas_width}px`,
                    "--canvas-height": `${composition.canvas_height}px`,
                  } as CSSProperties
                }
              >
                <ReminderBubble
                  settings={settings}
                  title="Team standup"
                  location="Zoom"
                  startTime={previewStartTime(15)}
                  interactive={false}
                  editMode
                  selectedLayerId={selectedLayerId}
                  onSelectLayer={setSelectedLayerId}
                  onCompositionChange={updateComp}
                />
              </div>
            </div>

            {selectedLayer && (
              <LayerProperties
                layer={selectedLayer}
                onChange={(patch) => updateLayer(selectedLayer.id, patch)}
              />
            )}

            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => {
                  updateComp(createDefaultComposition(settings));
                  setSelectedLayerId(null);
                }}
                className="rounded-lg bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
              >
                Reset layers
              </button>
            </div>
          </div>
        </div>
      </div>

      <style>{reminderBubbleStyles}</style>
    </section>
  );
}

function LayerProperties({
  layer,
  onChange,
}: {
  layer: OverlayLayer;
  onChange: (patch: Partial<OverlayLayer>) => void;
}) {
  return (
    <div className="rounded-xl border border-slate-800 bg-slate-950 p-4">
      <h3 className="mb-3 text-sm font-medium">Selected layer</h3>
      <div className="grid gap-3 md:grid-cols-2">
        <label className="block text-sm md:col-span-2">
          <span className="mb-1 block text-slate-400">Name</span>
          <input
            value={layer.name}
            onChange={(e) => onChange({ name: e.target.value })}
            className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
          />
        </label>

        <div className="text-xs text-slate-500 md:col-span-2">
          Position: {layer.x}%, {layer.y}% · Stack order: {layer.z_index + 1}
        </div>

        {layer.type === "image" && (
          <>
            <label className="block text-sm md:col-span-2">
              <span className="mb-1 block text-slate-400">Built-in icon</span>
              <select
                value={layer.icon_id ?? "bell"}
                onChange={(e) => onChange({ icon_id: e.target.value, image_path: null })}
                className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
              >
                {BUILTIN_ICONS.map((icon) => (
                  <option key={icon.id} value={icon.id}>
                    {icon.emoji} {icon.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block text-sm md:col-span-2">
              <span className="mb-1 block text-slate-400">Image path or upload</span>
              <input
                value={layer.image_path ?? ""}
                onChange={(e) =>
                  onChange({ image_path: e.target.value ? e.target.value : null })
                }
                placeholder="/path/to/image.png"
                className="mb-2 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
              />
              <input
                type="file"
                accept="image/*"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (file) {
                    onChange({ image_path: URL.createObjectURL(file) });
                  }
                }}
                className="block w-full text-xs text-slate-400"
              />
            </label>
            <label className="block text-sm">
              <span className="mb-1 block text-slate-400">Size ({layer.width ?? 48}px)</span>
              <input
                type="range"
                min={16}
                max={160}
                value={layer.width ?? 48}
                onChange={(e) => onChange({ width: Number(e.target.value) })}
                className="w-full"
              />
            </label>
          </>
        )}

        {layer.type === "text" && (
          <>
            <label className="block text-sm md:col-span-2">
              <span className="mb-1 block text-slate-400">Text</span>
              <textarea
                value={layer.text_content ?? ""}
                onChange={(e) => onChange({ text_content: e.target.value })}
                rows={3}
                className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
              />
            </label>
            <label className="block text-sm">
              <span className="mb-1 block text-slate-400">Box width ({layer.width ?? 140}px)</span>
              <input
                type="range"
                min={60}
                max={320}
                value={layer.width ?? 140}
                onChange={(e) => onChange({ width: Number(e.target.value) })}
                className="w-full"
              />
            </label>
            <label className="block text-sm">
              <span className="mb-1 block text-slate-400">Box height ({layer.height ?? 40}px)</span>
              <input
                type="range"
                min={24}
                max={120}
                value={layer.height ?? 40}
                onChange={(e) => onChange({ height: Number(e.target.value) })}
                className="w-full"
              />
            </label>
            <label className="block text-sm md:col-span-2">
              <span className="mb-1 block text-slate-400">Font size override</span>
              <input
                type="number"
                min={8}
                max={48}
                placeholder="Default"
                value={layer.font_size ?? ""}
                onChange={(e) =>
                  onChange({
                    font_size: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
              />
            </label>
          </>
        )}

        {layer.type === "title" && (
          <>
            <p className="text-sm text-slate-400 md:col-span-2">
              Shows the event title and location when a reminder fires.
            </p>
            <label className="block text-sm">
              <span className="mb-1 block text-slate-400">Box width ({layer.width ?? 220}px)</span>
              <input
                type="range"
                min={80}
                max={320}
                value={layer.width ?? 220}
                onChange={(e) => onChange({ width: Number(e.target.value) })}
                className="w-full"
              />
            </label>
            <label className="block text-sm">
              <span className="mb-1 block text-slate-400">Box height ({layer.height ?? 52}px)</span>
              <input
                type="range"
                min={28}
                max={120}
                value={layer.height ?? 52}
                onChange={(e) => onChange({ height: Number(e.target.value) })}
                className="w-full"
              />
            </label>
          </>
        )}

        {layer.type === "countdown" && (
          <p className="text-sm text-slate-400 md:col-span-2">
            Shows a live countdown until the event start time.
          </p>
        )}
      </div>
    </div>
  );
}
