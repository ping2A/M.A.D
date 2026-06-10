import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { ComplianceStatus, PillarId, RequirementSeverity } from "../types";
import { en } from "./en";
import { fr } from "./fr";
import { getPolicyContentCatalog } from "./policyContent";
import type { Locale, Translations } from "./types";

const STORAGE_KEY = "mad-locale";

const catalogs: Record<Locale, Translations> = { en, fr };

function detectLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "en" || stored === "fr") return stored;
  const lang = navigator.language.toLowerCase();
  return lang.startsWith("fr") ? "fr" : "en";
}

function interpolate(
  template: string,
  params?: Record<string, string | number>,
): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) =>
    params[key] !== undefined ? String(params[key]) : `{${key}}`,
  );
}

interface LocaleContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: Translations;
  format: (template: string, params?: Record<string, string | number>) => string;
  pillarLabel: (id: PillarId) => string;
  pillarShortLabel: (id: PillarId) => string;
  statusLabel: (status: ComplianceStatus) => string;
  severityLabel: (severity: RequirementSeverity) => string;
}

const LocaleContext = createContext<LocaleContextValue | null>(null);

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(detectLocale);

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
    localStorage.setItem(STORAGE_KEY, next);
  }, []);

  const t = catalogs[locale];

  useEffect(() => {
    document.documentElement.lang = t.meta.htmlLang;
    const meta = document.querySelector('meta[name="description"]');
    if (meta) meta.setAttribute("content", t.meta.pageDescription);
  }, [t]);

  const format = useCallback(
    (template: string, params?: Record<string, string | number>) =>
      interpolate(template, params),
    [],
  );

  const pillarLabel = useCallback(
    (id: PillarId) =>
      getPolicyContentCatalog(locale)?.pillars[id as keyof typeof t.pillar]?.name
      ?? (t.pillar as Record<string, string>)[id]
      ?? id,
    [locale, t],
  );

  const pillarShortLabel = useCallback(
    (id: PillarId) => {
      const full = getPolicyContentCatalog(locale)?.pillars[id as keyof typeof t.pillar]?.name
        ?? (t.pillar as Record<string, string>)[id];
      if (full) {
        return full.split(" ")[0];
      }
      return (t.pillarShort as Record<string, string>)[id] ?? id;
    },
    [locale, t],
  );

  const statusLabel = useCallback(
    (status: ComplianceStatus) => t.status[status],
    [t],
  );

  const severityLabel = useCallback(
    (severity: RequirementSeverity) => t.severity[severity],
    [t],
  );

  const value = useMemo(
    () => ({
      locale,
      setLocale,
      t,
      format,
      pillarLabel,
      pillarShortLabel,
      statusLabel,
      severityLabel,
    }),
    [locale, setLocale, t, format, pillarLabel, pillarShortLabel, statusLabel, severityLabel],
  );

  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocale() {
  const ctx = useContext(LocaleContext);
  if (!ctx) throw new Error("useLocale must be used within LocaleProvider");
  return ctx;
}
