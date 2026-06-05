import { BUILTIN_ICONS } from "../lib/types";

interface Props {
  selectedId: string;
  onSelect: (iconId: string) => void;
}

export function IconPicker({ selectedId, onSelect }: Props) {
  return (
    <div>
      <div className="mb-2 text-sm text-slate-400">Reminder icon</div>
      <div className="grid grid-cols-3 gap-2 sm:grid-cols-6">
        {BUILTIN_ICONS.map((icon) => (
          <button
            key={icon.id}
            type="button"
            onClick={() => onSelect(icon.id)}
            className={`flex flex-col items-center rounded-lg border px-2 py-3 text-sm ${
              selectedId === icon.id
                ? "border-indigo-500 bg-indigo-950"
                : "border-slate-700 bg-slate-950 hover:border-slate-500"
            }`}
          >
            <span className="text-2xl">{icon.emoji}</span>
            <span className="mt-1 text-xs text-slate-400">{icon.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
