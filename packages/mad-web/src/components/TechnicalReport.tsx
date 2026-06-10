import type { EvaluationReport, PolicySummary } from "../types";
import { RequirementList } from "./RequirementList";
import { VendorScorecard } from "./VendorScorecard";

interface TechnicalReportProps {
  policy: PolicySummary;
  evaluation: EvaluationReport;
}

export function TechnicalReport({ policy, evaluation }: TechnicalReportProps) {
  const rankedVendors = [...evaluation.vendors].sort(
    (a, b) =>
      b.overall_score.overall_score_percent -
      a.overall_score.overall_score_percent,
  );

  return (
    <section className="tech-report">
      <h2>Technical Evaluation Report</h2>
      <div className="report-toolbar">
        <p className="report-meta">
          Policy v{policy.version} · {policy.total_requirements} requirements ·{" "}
          {evaluation.vendors.length} vendors assessed · Mobile MDM (iOS & Android) only
        </p>
        <a className="download-btn" href="/api/report.html" download="mad-evaluation-report.html">
          Download HTML Report
        </a>
      </div>

      <article className="report-block">
        <h3>1. Purpose and Scope</h3>
        <p>
          Operation M.A.D. is an <strong>evaluation-only</strong> platform. It assesses
          whether candidate MDM vendors meet a corporate mobile security standard before
          procurement. It does not enroll devices, push profiles, or enforce compliance.
        </p>
        <div className="scope-grid">
          <div className="scope-in">
            <h4>In scope</h4>
            <ul>
              <li>iOS MDM (ABM, supervised mode)</li>
              <li>Android Enterprise (Work Profile, COBO, kiosk)</li>
              <li>Vendor capability assessment & scoring</li>
            </ul>
          </div>
          <div className="scope-out">
            <h4>Out of scope</h4>
            <ul>
              <li>Desktop / laptop management</li>
              <li>Post-selection policy enforcement</li>
              <li>Device deployment or ongoing management</li>
            </ul>
          </div>
        </div>
      </article>

      <article className="report-block">
        <h3>2. Evaluation Methodology</h3>
        <p>
          Requirements are defined in Policy-as-Code YAML and mapped to a compliance
          status per vendor. Each status carries a scoring weight used to compute
          pillar and overall scores.
        </p>
        <table className="report-table">
          <thead>
            <tr>
              <th>Status</th>
              <th>Weight</th>
              <th>Meaning</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><code>compliant</code></td>
              <td>1.0</td>
              <td>Native capability, no workarounds</td>
            </tr>
            <tr>
              <td><code>partial</code></td>
              <td>0.5</td>
              <td>Limited, platform-specific, or manual</td>
            </tr>
            <tr>
              <td><code>non_compliant</code></td>
              <td>0.0</td>
              <td>Cannot be met with current capabilities</td>
            </tr>
            <tr>
              <td><code>untested</code></td>
              <td>0.0</td>
              <td>No assessment data recorded</td>
            </tr>
          </tbody>
        </table>

        <h4>Scoring formula</h4>
        <pre className="code-block">{`pillar_score = ((compliant × 1.0) + (partial × 0.5)) / total × 100
overall_score  = mean(cybersecurity, dfir, platform_os)

critical_gap   = critical requirement AND (non_compliant OR untested)`}</pre>

        <h4>Data flow</h4>
        <pre className="code-block flow">{`policies/*.yaml → PolicyBundle → Evaluator
                                      ↓
VendorAssessment (per-requirement status)
                                      ↓
EvaluationReport → CLI / API / Dashboard`}</pre>
      </article>

      <article className="report-block">
        <h3>3. Requirements and Technical Criteria</h3>
        <p>
          Each requirement includes an evaluation method (how to test) and technical
          criteria (APIs, payloads, protocols). Production evaluations replace sample
          data with lab tests and API probes.
        </p>
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
        <h3>4. Vendor Assessment Results</h3>
        <p>
          Sample assessments demonstrating the scoring engine. Ranked by overall score;
          critical gaps are flagged independently of percentage.
        </p>
        <div className="vendor-grid">
          {rankedVendors.map((result, i) => (
            <VendorScorecard key={result.vendor.name} result={result} rank={i + 1} detailed />
          ))}
        </div>
      </article>

      <style>{`
        .tech-report h2 {
          margin: 0 0 0.25rem;
          color: var(--mad-navy);
        }
        .report-toolbar {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          justify-content: space-between;
          gap: 1rem;
          margin-bottom: 1.5rem;
        }
        .report-meta {
          font-family: var(--font-mono);
          font-size: 0.85rem;
          color: var(--mad-text-muted);
          margin: 0;
        }
        .download-btn {
          display: inline-flex;
          align-items: center;
          gap: 0.4rem;
          background: var(--mad-navy);
          color: white;
          text-decoration: none;
          padding: 0.6rem 1rem;
          border-radius: 6px;
          font-size: 0.85rem;
          font-weight: 600;
          border: 2px solid var(--mad-cyan);
          transition: background 0.2s;
        }
        .download-btn:hover {
          background: var(--mad-navy-light);
        }
        .report-block {
          background: white;
          border-radius: 10px;
          padding: 1.5rem;
          margin-bottom: 1.25rem;
          box-shadow: 0 2px 8px rgba(10, 22, 40, 0.08);
        }
        .report-block h3 {
          margin: 0 0 0.75rem;
          color: var(--mad-navy);
          font-size: 1.1rem;
          border-bottom: 2px solid var(--mad-cyan);
          padding-bottom: 0.5rem;
        }
        .report-block h4 {
          margin: 1rem 0 0.5rem;
          color: var(--mad-navy);
          font-size: 0.95rem;
        }
        .report-block p {
          margin: 0 0 0.75rem;
          color: var(--mad-text-muted);
          line-height: 1.6;
        }
        .scope-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 1rem;
          margin-top: 1rem;
        }
        @media (max-width: 640px) {
          .scope-grid { grid-template-columns: 1fr; }
        }
        .scope-in, .scope-out {
          padding: 1rem;
          border-radius: 8px;
          font-size: 0.9rem;
        }
        .scope-in {
          background: #e8f5e9;
          border-left: 4px solid var(--mad-compliant);
        }
        .scope-out {
          background: #fde8ea;
          border-left: 4px solid var(--mad-critical);
        }
        .scope-in h4, .scope-out h4 {
          margin: 0 0 0.5rem;
          font-size: 0.85rem;
          text-transform: uppercase;
          letter-spacing: 0.04em;
        }
        .scope-in ul, .scope-out ul {
          margin: 0;
          padding-left: 1.25rem;
        }
        .scope-in li, .scope-out li {
          margin-bottom: 0.25rem;
        }
        .report-table {
          width: 100%;
          border-collapse: collapse;
          font-size: 0.9rem;
          margin: 1rem 0;
        }
        .report-table th, .report-table td {
          padding: 0.6rem 0.75rem;
          text-align: left;
          border-bottom: 1px solid #e2e6ea;
        }
        .report-table th {
          background: var(--mad-navy-light);
          color: white;
          font-weight: 600;
        }
        .report-table code {
          font-family: var(--font-mono);
          font-size: 0.8rem;
          background: #f0f2f5;
          padding: 0.1rem 0.35rem;
          border-radius: 3px;
        }
        .code-block {
          background: var(--mad-navy);
          color: var(--mad-cyan);
          padding: 1rem;
          border-radius: 8px;
          font-family: var(--font-mono);
          font-size: 0.8rem;
          line-height: 1.6;
          overflow-x: auto;
          margin: 0.75rem 0;
        }
        .code-block.flow {
          color: var(--mad-silver);
        }
        .pillar-section {
          margin-top: 1.5rem;
        }
        .pillar-desc {
          font-size: 0.85rem;
          margin-bottom: 0.75rem !important;
        }
        .req-panel {
          border: 1px solid #e2e6ea;
          border-radius: 8px;
          overflow: hidden;
        }
        .vendor-grid {
          display: flex;
          flex-direction: column;
          gap: 1rem;
          margin-top: 1rem;
        }
      `}</style>
    </section>
  );
}
