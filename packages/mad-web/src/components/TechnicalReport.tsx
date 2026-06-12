import type { EvaluationReport, PolicySummary } from "../types";
import { useLocale } from "../i18n/LocaleContext";
import { rankVendors } from "../utils/ranking";
import { RequirementList } from "./RequirementList";
import { ReportDownloads } from "./ReportDownloads";
import { ReportEmbedPreview } from "./ReportEmbedPreview";
import { VendorScorecard } from "./VendorScorecard";

interface TechnicalReportProps {
  policy: PolicySummary;
  evaluation: EvaluationReport;
  activeTags?: Set<string>;
}

export function TechnicalReport({
  policy,
  evaluation,
  activeTags = new Set(),
}: TechnicalReportProps) {
  const { t, format } = useLocale();
  const rankedVendors = rankVendors(evaluation);
  const tagList = [...activeTags].join(", ");

  return (
    <section className="tech-report">
      <h2>{t.report.title}</h2>
      <div className="report-toolbar">
        <p className="report-meta">
          {format(t.report.meta, {
            version: policy.version,
            requirements: policy.total_requirements,
            vendors: evaluation.vendors.length,
          })}
        </p>
        <ReportDownloads activeTags={activeTags} />
      </div>

      {activeTags.size > 0 && (
        <p className="report-tag-filter-note">
          {format(t.report.tagsFilterActive, { tags: tagList })}
        </p>
      )}

      <ReportEmbedPreview activeTags={activeTags} />

      <article className="report-block">
        <h3>{t.report.purposeTitle}</h3>
        <p>{t.report.purposeBody}</p>
        <div className="scope-grid">
          <div className="scope-in">
            <h4>{t.report.inScope}</h4>
            <ul>
              {t.report.inScopeItems.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>
          <div className="scope-out">
            <h4>{t.report.outScope}</h4>
            <ul>
              {t.report.outScopeItems.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>
        </div>
      </article>

      <article className="report-block">
        <h3>{t.report.methodologyTitle}</h3>
        <p>{t.report.methodologyBody}</p>
        <table className="report-table">
          <thead>
            <tr>
              <th>{t.report.statusCol}</th>
              <th>{t.report.weightCol}</th>
              <th>{t.report.meaningCol}</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><code>compliant</code></td>
              <td>1.0</td>
              <td>{t.report.compliantMeaning}</td>
            </tr>
            <tr>
              <td><code>partial</code></td>
              <td>0.5</td>
              <td>{t.report.partialMeaning}</td>
            </tr>
            <tr>
              <td><code>non_compliant</code></td>
              <td>0.0</td>
              <td>{t.report.nonCompliantMeaning}</td>
            </tr>
            <tr>
              <td><code>untested</code></td>
              <td>0.0</td>
              <td>{t.report.untestedMeaning}</td>
            </tr>
          </tbody>
        </table>

        <h4>{t.report.formulaTitle}</h4>
        <pre className="code-block">{`pillar_score = ((compliant × 1.0) + (partial × 0.5)) / total × 100
overall_score  = mean(cybersecurity, dfir, platform_os)

critical_gap   = critical requirement AND (non_compliant OR untested)`}</pre>

        <h4>{t.report.dataFlowTitle}</h4>
        <pre className="code-block flow">{`policies/*.yaml → PolicyBundle → Evaluator
                                      ↓
VendorAssessment (per-requirement status)
                                      ↓
EvaluationReport → CLI / API / Dashboard`}</pre>
      </article>

      <article className="report-block">
        <h3>{t.report.requirementsTitle}</h3>
        <p>{t.report.requirementsBody}</p>
        {policy.pillars.map((pillar) => (
          <div key={pillar.id} className="pillar-section">
            <h4>{pillar.name}</h4>
            <p className="pillar-desc">{pillar.description}</p>
            <div className="req-panel">
              <RequirementList requirements={pillar.requirements} showTechnical />
            </div>
          </div>
        ))}
      </article>

      <article className="report-block">
        <h3>{t.report.resultsTitle}</h3>
        <p>{t.report.resultsBody}</p>
        <div className="vendor-grid">
          {rankedVendors.map((result, i) => (
            <VendorScorecard key={result.vendor.name} result={result} rank={i + 1} detailed />
          ))}
        </div>
      </article>

      <style>{`
        .tech-report h2 { margin: 0 0 0.25rem; color: var(--mad-navy); }
        .report-toolbar {
          display: flex; flex-wrap: wrap; align-items: center;
          justify-content: space-between; gap: 1rem; margin-bottom: 1.5rem;
        }
        .report-meta {
          font-family: var(--font-mono); font-size: 0.85rem;
          color: var(--mad-text-muted); margin: 0;
        }
        .report-tag-filter-note {
          margin: -0.5rem 0 1rem;
          padding: 0.65rem 0.9rem;
          border-radius: 8px;
          background: #f0fbff;
          border: 1px solid var(--mad-cyan);
          font-size: 0.85rem;
          color: var(--mad-navy);
          font-weight: 600;
        }
        .report-block {
          background: white; border-radius: 10px; padding: 1.5rem;
          margin-bottom: 1.25rem; box-shadow: 0 2px 8px rgba(10, 22, 40, 0.08);
        }
        .report-block h3 {
          margin: 0 0 0.75rem; color: var(--mad-navy); font-size: 1.1rem;
          border-bottom: 2px solid var(--mad-cyan); padding-bottom: 0.5rem;
        }
        .report-block h4 { margin: 1rem 0 0.5rem; color: var(--mad-navy); font-size: 0.95rem; }
        .report-block p { margin: 0 0 0.75rem; color: var(--mad-text-muted); line-height: 1.6; }
        .scope-grid {
          display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1rem;
        }
        @media (max-width: 640px) { .scope-grid { grid-template-columns: 1fr; } }
        .scope-in, .scope-out {
          padding: 1rem; border-radius: 8px; font-size: 0.9rem;
        }
        .scope-in { background: #e8f5e9; border-left: 4px solid var(--mad-compliant); }
        .scope-out { background: #fde8ea; border-left: 4px solid var(--mad-critical); }
        .scope-in h4, .scope-out h4 {
          margin: 0 0 0.5rem; font-size: 0.85rem;
          text-transform: uppercase; letter-spacing: 0.04em;
        }
        .scope-in ul, .scope-out ul { margin: 0; padding-left: 1.25rem; }
        .scope-in li, .scope-out li { margin-bottom: 0.25rem; }
        .report-table {
          width: 100%; border-collapse: collapse; font-size: 0.9rem; margin: 1rem 0;
        }
        .report-table th, .report-table td {
          padding: 0.6rem 0.75rem; text-align: left; border-bottom: 1px solid #e2e6ea;
        }
        .report-table th { background: var(--mad-navy-light); color: white; font-weight: 600; }
        .report-table code {
          font-family: var(--font-mono); font-size: 0.8rem;
          background: #f0f2f5; padding: 0.1rem 0.35rem; border-radius: 3px;
        }
        .code-block {
          background: var(--mad-navy); color: var(--mad-cyan); padding: 1rem;
          border-radius: 8px; font-family: var(--font-mono); font-size: 0.8rem;
          line-height: 1.6; overflow-x: auto; margin: 0.75rem 0;
        }
        .code-block.flow { color: var(--mad-silver); }
        .pillar-section { margin-top: 1.5rem; }
        .pillar-desc { font-size: 0.85rem; margin-bottom: 0.75rem !important; }
        .req-panel { border: 1px solid #e2e6ea; border-radius: 8px; overflow: hidden; }
        .vendor-grid {
          display: flex; flex-direction: column; gap: 1rem; margin-top: 1rem;
        }
      `}</style>
    </section>
  );
}
