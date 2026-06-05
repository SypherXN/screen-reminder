interface Props {
  value: string;
  options: string[];
  onChange: (value: string) => void;
}

export function FontPicker({ value, options, onChange }: Props) {
  return (
    <label className="block text-sm">
      <span className="mb-2 block text-slate-400">Font family</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
        style={{ fontFamily: value }}
      >
        {options.map((font) => (
          <option key={font} value={font} style={{ fontFamily: font }}>
            {font.split(",")[0]}
          </option>
        ))}
      </select>
    </label>
  );
}
