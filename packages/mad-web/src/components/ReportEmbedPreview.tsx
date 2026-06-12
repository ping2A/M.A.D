import { useEffect, useMemo, useRef, useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import { reportTagsQuery } from "../utils/comparisonFilter";

interface ReportEmbedPreviewProps {
  activeTags?: Set<string>;
}

export function ReportEmbedPreview({ activeTags = new Set() }: ReportEmbedPreviewProps) {
  const { t, locale } = useLocale();
  const tagQuery = useMemo(() => reportTagsQuery(activeTags), [activeTags]);
  const tagSuffix = tagQuery ? `&${tagQuery}` : "";
  const embedSrc = `/api/report.html?embed=1&lang=${locale}${tagSuffix}`;
  const openHref = `/api/report.html?lang=${locale}${tagSuffix}`;
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [height, setHeight] = useState(480);

  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (event.data?.type !== "mad-report-resize") return;
      const next = Number(event.data.height);
      if (Number.isFinite(next) && next > 0) {
        setHeight(Math.min(Math.max(next, 320), 2400));
      }
    };
    window.addEventListener("message", onMessage);
    return () => window.removeEventListener("message", onMessage);
  }, []);

  return (
    <article className="report-embed-block">
      <div className="report-embed-header">
        <h3>{t.report.embedPreviewTitle}</h3>
        <a
          className="report-embed-open"
          href={openHref}
          target="_blank"
          rel="noopener noreferrer"
        >
          {t.report.openLiveReport}
        </a>
      </div>
      <p className="report-embed-hint">{t.report.embedPreviewHint}</p>
      <div className="report-embed-frame-wrap">
        <iframe
          ref={iframeRef}
          title={t.report.embedPreviewTitle}
          src={embedSrc}
          className="report-embed-frame"
          style={{ height: `${height}px` }}
        />
      </div>

      <style>{`
        .report-embed-block {
          background: white;
          border-radius: 10px;
          padding: 1.25rem 1.5rem;
          margin-bottom: 1.25rem;
          box-shadow: 0 2px 8px rgba(10, 22, 40, 0.08);
        }
        .report-embed-header {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          justify-content: space-between;
          gap: 0.75rem;
          margin-bottom: 0.5rem;
        }
        .report-embed-header h3 {
          margin: 0;
          color: var(--mad-navy);
          font-size: 1.05rem;
          border-bottom: 2px solid var(--mad-cyan);
          padding-bottom: 0.35rem;
        }
        .report-embed-open {
          font-size: 0.82rem;
          font-weight: 600;
          color: var(--mad-navy);
          text-decoration: none;
          border: 1px solid var(--mad-cyan);
          padding: 0.35rem 0.75rem;
          border-radius: 6px;
        }
        .report-embed-open:hover {
          background: #f0fbff;
        }
        .report-embed-hint {
          margin: 0 0 0.75rem;
          font-size: 0.85rem;
          color: var(--mad-text-muted);
        }
        .report-embed-frame-wrap {
          border: 1px solid #dde1e6;
          border-radius: 8px;
          overflow: hidden;
          background: #e8eaed;
        }
        .report-embed-frame {
          display: block;
          width: 100%;
          border: none;
          background: white;
        }
      `}</style>
    </article>
  );
}
