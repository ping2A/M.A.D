import { useState } from "react";
import { downloadReport, type ReportFormat } from "../api/client";
import { useLocale } from "../i18n/LocaleContext";

interface ReportDownloadsProps {
  activeTags?: Set<string>;
}

export function ReportDownloads({ activeTags = new Set() }: ReportDownloadsProps) {
  const { t, locale } = useLocale();
  const [downloading, setDownloading] = useState<ReportFormat | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleDownload = async (format: ReportFormat) => {
    setDownloading(format);
    setError(null);
    try {
      await downloadReport(format, locale, [...activeTags]);
    } catch (err) {
      setError(err instanceof Error ? err.message : t.report.downloadError);
    } finally {
      setDownloading(null);
    }
  };

  return (
    <div className="report-downloads">
      <button
        type="button"
        className="download-btn"
        disabled={downloading !== null}
        onClick={() => handleDownload("html")}
      >
        {downloading === "html" ? t.report.downloading : t.report.downloadHtml}
      </button>
      <button
        type="button"
        className="download-btn download-btn-secondary"
        disabled={downloading !== null}
        onClick={() => handleDownload("pdf")}
      >
        {downloading === "pdf" ? t.report.downloading : t.report.downloadPdf}
      </button>
      {error && <p className="download-error">{error}</p>}

      <style>{`
        .report-downloads {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          gap: 0.6rem;
        }
        .download-btn {
          display: inline-flex;
          align-items: center;
          gap: 0.4rem;
          background: var(--mad-navy);
          color: white;
          border: 2px solid var(--mad-cyan);
          padding: 0.6rem 1rem;
          border-radius: 6px;
          font-size: 0.85rem;
          font-weight: 600;
          font-family: inherit;
          cursor: pointer;
          transition: background 0.2s;
        }
        .download-btn:hover:not(:disabled) {
          background: var(--mad-navy-light);
        }
        .download-btn:disabled {
          opacity: 0.65;
          cursor: not-allowed;
        }
        .download-btn-secondary {
          background: white;
          color: var(--mad-navy);
        }
        .download-btn-secondary:hover:not(:disabled) {
          background: #f4f6f8;
        }
        .download-error {
          flex-basis: 100%;
          margin: 0;
          font-size: 0.82rem;
          color: var(--mad-critical);
        }
      `}</style>
    </div>
  );
}
