import { useCallback, useMemo } from "react";
import {
  formatDurationWithLabels,
  type DurationFormatLabels,
} from "../utils/valueStream";
import { useLocale } from "./LocaleContext";

export function useFormatDuration() {
  const { t } = useLocale();
  const labels = useMemo<DurationFormatLabels>(
    () => ({
      minute: t.vsm.durationShortMinute,
      hour: t.vsm.durationShortHour,
      day: t.vsm.durationShortDay,
      week: t.vsm.durationShortWeek,
      sep: t.vsm.durationShortSep,
    }),
    [t],
  );
  return useCallback(
    (minutes: number | null | undefined, style: "auto" | "compact" = "auto") =>
      formatDurationWithLabels(minutes, style, labels),
    [labels],
  );
}
