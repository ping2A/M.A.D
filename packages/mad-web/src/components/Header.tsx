import { LanguageSwitcher } from "./LanguageSwitcher";
import { useLocale } from "../i18n/LocaleContext";

export function Header() {
  const { t } = useLocale();

  return (
    <header className="header">
      <div className="header-inner">
        <div className="header-brand">
          <img src="/logo.png" alt={t.header.logoAlt} className="header-logo" />
          <div>
            <h1 className="header-title">{t.header.title}</h1>
            <p className="header-subtitle">{t.header.subtitle}</p>
          </div>
        </div>
        <div className="header-actions">
          <p className="header-tagline">{t.header.tagline}</p>
          <LanguageSwitcher />
        </div>
      </div>
      <style>{`
        .header {
          background: linear-gradient(135deg, var(--mad-navy) 0%, var(--mad-blue) 100%);
          color: white;
          padding: 1.25rem 2rem;
          border-bottom: 3px solid var(--mad-cyan);
          box-shadow: 0 4px 20px rgba(10, 22, 40, 0.3);
        }
        .header-inner {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 1.5rem;
          max-width: 1400px;
          margin: 0 auto;
          flex-wrap: wrap;
        }
        .header-brand {
          display: flex;
          align-items: center;
          gap: 1.25rem;
        }
        .header-actions {
          display: flex;
          align-items: center;
          gap: 1rem;
          flex-wrap: wrap;
        }
        .header-logo {
          width: 64px;
          height: 64px;
          border-radius: 8px;
          object-fit: cover;
          border: 2px solid var(--mad-cyan);
          box-shadow: 0 0 16px rgba(0, 180, 216, 0.4);
        }
        .header-title {
          margin: 0;
          font-size: 1.6rem;
          font-weight: 700;
          letter-spacing: 0.02em;
        }
        .header-subtitle {
          margin: 0.2rem 0 0;
          font-size: 0.9rem;
          color: var(--mad-silver);
          opacity: 0.9;
        }
        .header-tagline {
          margin: 0;
          font-size: 0.82rem;
          color: var(--mad-silver);
          opacity: 0.75;
          text-align: right;
        }
        @media (max-width: 640px) {
          .header-tagline { display: none; }
        }
      `}</style>
    </header>
  );
}
