import { useLocale } from "../i18n/LocaleContext";
import {
  DOC_COLOR_PRESETS,
  isCustomDocColor,
  normalizeDocColor,
  resolveDocColorHex,
} from "../utils/vendorDoc";

interface VendorDocColorPickerProps {
  value?: string | null;
  onChange: (color: string | null) => void;
  compact?: boolean;
}

export function VendorDocColorPicker({ value, onChange, compact }: VendorDocColorPickerProps) {
  const { t } = useLocale();
  const normalized = normalizeDocColor(value);
  const isCustom = isCustomDocColor(normalized);
  const customHex = isCustom ? resolveDocColorHex(normalized)! : "#00b4d8";

  return (
    <div className={`vendor-doc-color-picker${compact ? " compact" : ""}`}>
      <span className="vendor-doc-color-label">{t.vendorDocs.itemColor}</span>
      <div className="vendor-doc-color-swatches" role="radiogroup" aria-label={t.vendorDocs.itemColor}>
        {DOC_COLOR_PRESETS.map((preset) => {
          const active = (normalized ?? "") === preset.id;
          return (
            <button
              key={preset.id || "none"}
              type="button"
              role="radio"
              aria-checked={active}
              title={preset.label}
              className={`vendor-doc-color-swatch${active ? " active" : ""}${preset.id ? "" : " none"}`}
              style={
                preset.hex
                  ? ({ "--swatch-color": preset.hex } as React.CSSProperties)
                  : undefined
              }
              onClick={() => onChange(preset.id || null)}
            />
          );
        })}
        <label
          className={`vendor-doc-color-custom${isCustom ? " active" : ""}`}
          title={t.vendorDocs.customColor}
        >
          <input
            type="color"
            value={customHex}
            onChange={(e) => onChange(e.target.value)}
          />
          <span
            className="vendor-doc-color-custom-icon"
            style={{ background: isCustom ? customHex : undefined }}
            aria-hidden
          />
        </label>
      </div>
    </div>
  );
}
