import { useLocale } from "../i18n/LocaleContext";
import type { Locale } from "../i18n/types";

const LOCALES: { code: Locale; label: string }[] = [
  { code: "en", label: "EN" },
  { code: "fr", label: "FR" },
];

export function LanguageSwitcher() {
  const { locale, setLocale, t } = useLocale();

  return (
    <div className="lang-switcher" role="group" aria-label={t.header.language}>
      {LOCALES.map(({ code, label }) => (
        <button
          key={code}
          type="button"
          className={`lang-btn ${locale === code ? "active" : ""}`}
          onClick={() => setLocale(code)}
          aria-pressed={locale === code}
        >
          {label}
        </button>
      ))}
      <style>{`
        .lang-switcher {
          display: flex;
          gap: 0.25rem;
          background: rgba(0, 0, 0, 0.2);
          border-radius: 6px;
          padding: 0.2rem;
          border: 1px solid rgba(255, 255, 255, 0.15);
        }
        .lang-btn {
          background: transparent;
          border: none;
          color: var(--mad-silver);
          font-size: 0.75rem;
          font-weight: 700;
          padding: 0.35rem 0.6rem;
          border-radius: 4px;
          cursor: pointer;
          font-family: inherit;
          letter-spacing: 0.04em;
        }
        .lang-btn:hover {
          color: white;
          background: rgba(255, 255, 255, 0.1);
        }
        .lang-btn.active {
          background: var(--mad-cyan);
          color: var(--mad-navy);
        }
      `}</style>
    </div>
  );
}
