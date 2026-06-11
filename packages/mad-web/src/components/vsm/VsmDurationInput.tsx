import { useEffect, useState } from "react";
import { useLocale } from "../../i18n/LocaleContext";
import {
  bestDurationUnit,
  minutesToUnitValue,
  parseDurationInput,
  type DurationUnit,
} from "../../utils/valueStream";

interface VsmDurationInputProps {
  minutes: number | null | undefined;
  onChange: (minutes: number | null) => void;
  inputKey?: string;
}

export function VsmDurationInput({ minutes, onChange, inputKey }: VsmDurationInputProps) {
  const { t } = useLocale();
  const [unit, setUnit] = useState<DurationUnit>(() => bestDurationUnit(minutes));

  useEffect(() => {
    setUnit(bestDurationUnit(minutes));
  }, [inputKey]);

  const unitLabel = (value: DurationUnit) => {
    switch (value) {
      case "hours":
        return t.vsm.durationUnitHours;
      case "days":
        return t.vsm.durationUnitDays;
      case "weeks":
        return t.vsm.durationUnitWeeks;
      default:
        return t.vsm.durationUnitMinutes;
    }
  };

  return (
    <div className="vsm-duration-input">
      <input
        type="number"
        min={0}
        step={unit === "minutes" ? 1 : 0.25}
        value={minutesToUnitValue(minutes, unit)}
        onChange={(e) => onChange(parseDurationInput(e.target.value, unit))}
        placeholder="0"
      />
      <select
        value={unit}
        onChange={(e) => setUnit(e.target.value as DurationUnit)}
        aria-label={t.vsm.durationUnit}
      >
        {(["minutes", "hours", "days", "weeks"] as DurationUnit[]).map((option) => (
          <option key={option} value={option}>
            {unitLabel(option)}
          </option>
        ))}
      </select>
    </div>
  );
}
