import { useEffect, useState } from "react";
import { BUILTIN_ICONS, ANIMATION_PATHS, type AccountStyleOverrides, type AppSettings } from "../lib/types";
import { createDefaultComposition, normalizeSettings } from "../lib/composition";

interface Props {
  accountId: string;
  accountName: string;
  globalSettings: AppSettings;
  onClose: () => void;
  onSaved: () => void;
}

const emptyStyle = (): AccountStyleOverrides => ({
  enabled: false,
  icon_id: null,
  icon_size: null,
  animation_path: null,
  font_color: null,
  composition: null,
});

export function AccountStylePanel({
  accountId,
  accountName,
  globalSettings,
  onClose,
  onSaved,
}: Props) {
  const [style, setStyle] = useState<AccountStyleOverrides>(emptyStyle());
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    import("../lib/api").then(({ api }) => {
      api
        .getAccountStyle(accountId)
        .then((loaded) => setStyle(loaded ?? emptyStyle()))
        .catch(() => setStyle(emptyStyle()));
    });
  }, [accountId]);

  const save = async () => {
    setBusy(true);
    try {
      const { api } = await import("../lib/api");
      await api.saveAccountStyle(accountId, style);
      onSaved();
      onClose();
    } finally {
      setBusy(false);
    }
  };

  const useGlobalComposition = () => {
    setStyle((prev) => ({ ...prev, composition: null }));
  };

  const copyGlobalComposition = () => {
    setStyle((prev) => ({
      ...prev,
      composition: normalizeSettings(globalSettings).composition,
    }));
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-xl border border-slate-700 bg-slate-950 p-5">
        <h2 className="mb-1 text-lg font-medium">Style for {accountName}</h2>
        <p className="mb-4 text-sm text-slate-400">
          Override the global appearance for reminders from this account only.
        </p>

        <label className="mb-4 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={style.enabled}
            onChange={(e) => setStyle({ ...style, enabled: e.target.checked })}
          />
          Enable per-account styling
        </label>

        <div className="space-y-4">
          <label className="block text-sm">
            <span className="mb-2 block text-slate-400">Icon</span>
            <select
              value={style.icon_id ?? globalSettings.icon_id}
              onChange={(e) => setStyle({ ...style, icon_id: e.target.value })}
              className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
            >
              {BUILTIN_ICONS.map((icon) => (
                <option key={icon.id} value={icon.id}>
                  {icon.emoji} {icon.label}
                </option>
              ))}
            </select>
          </label>

          <label className="block text-sm">
            <span className="mb-2 block text-slate-400">Animation</span>
            <select
              value={style.animation_path ?? globalSettings.animation_path}
              onChange={(e) => setStyle({ ...style, animation_path: e.target.value })}
              className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
            >
              {ANIMATION_PATHS.map((path) => (
                <option key={path.id} value={path.id}>
                  {path.label}
                </option>
              ))}
            </select>
          </label>

          <label className="block text-sm">
            <span className="mb-2 block text-slate-400">Text color</span>
            <input
              type="color"
              value={style.font_color ?? globalSettings.font_color}
              onChange={(e) => setStyle({ ...style, font_color: e.target.value })}
              className="h-10 w-full rounded-lg border border-slate-700 bg-slate-900"
            />
          </label>

          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={copyGlobalComposition}
              className="rounded-lg bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
            >
              Copy current global layout
            </button>
            <button
              type="button"
              onClick={useGlobalComposition}
              className="rounded-lg bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
            >
              Use global layout
            </button>
            <button
              type="button"
              onClick={() =>
                setStyle({ ...style, composition: createDefaultComposition(globalSettings) })
              }
              className="rounded-lg bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
            >
              Reset to default layout
            </button>
          </div>

          {style.composition && (
            <p className="text-xs text-slate-500">
              Custom layout saved ({style.composition.layers.length} layers). Edit layers in
              Advanced after copying from global.
            </p>
          )}
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg bg-slate-800 px-4 py-2 text-sm hover:bg-slate-700"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={save}
            className="rounded-lg bg-indigo-600 px-4 py-2 text-sm hover:bg-indigo-500 disabled:opacity-50"
          >
            Save style
          </button>
        </div>
      </div>
    </div>
  );
}
