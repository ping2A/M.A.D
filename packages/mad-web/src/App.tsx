import { useState } from "react";
import { ComparisonView } from "./components/ComparisonView";
import { CriteriaEditor } from "./components/CriteriaEditor";
import { Header } from "./components/Header";
import { PillarCard } from "./components/PillarCard";
import { ScoreMatrix } from "./components/ScoreMatrix";
import { ScoringPanel } from "./components/ScoringPanel";
import { StatsBar } from "./components/StatsBar";
import { TechnicalReport } from "./components/TechnicalReport";
import { VendorScorecard } from "./components/VendorScorecard";
import { useEvaluation } from "./hooks/useEvaluation";

type Tab = "overview" | "criteria" | "matrix" | "compare" | "vendors" | "report";

export default function App() {
  const {
    policy,
    evaluation,
    loading,
    error,
    addRequirement,
    deleteRequirement,
    cycleAssessment,
    updateScoring,
  } = useEvaluation();

  const [tab, setTab] = useState<Tab>("compare");

  const rankedVendors = evaluation
    ? [...evaluation.vendors].sort(
        (a, b) =>
          b.overall_score.overall_score_percent -
          a.overall_score.overall_score_percent,
      )
    : [];

  return (
    <div className="app">
      <Header />

      <main className="main">
        {loading && <p className="loading">Loading evaluation data…</p>}
        {error && (
          <div className="error-banner">
            <strong>Unable to connect to mad-server.</strong>
            <p>
              Start the API with <code>cargo run -p mad-server</code> from the
              project root, then refresh.
            </p>
            <p className="error-detail">{error}</p>
          </div>
        )}

        {policy && evaluation && (
          <>
            {policy.scoring && (
              <ScoringPanel scoring={policy.scoring} onChange={updateScoring} />
            )}

            <StatsBar
              version={policy.version}
              pillars={policy.pillar_count}
              requirements={policy.total_requirements}
              critical={policy.critical_requirements}
              vendors={evaluation.vendors.length}
            />

            <nav className="tabs">
              {(
                [
                  ["overview", "Overview"],
                  ["criteria", "Criteria"],
                  ["matrix", "Score Matrix"],
                  ["compare", "Comparison"],
                  ["vendors", "Scorecards"],
                  ["report", "Report"],
                ] as [Tab, string][]
              ).map(([t, label]) => (
                <button
                  key={t}
                  type="button"
                  className={`tab ${tab === t ? "active" : ""}`}
                  onClick={() => setTab(t)}
                >
                  {label}
                </button>
              ))}
            </nav>

            {tab === "overview" && (
              <section className="section">
                <h2>Mobile MDM Vendor Evaluation</h2>
                <p className="section-intro">
                  Define criteria, score vendors in the matrix, and compare results.
                  Scores use severity weighting — critical requirements count more toward
                  the final ranking.
                </p>
                <div className="pillar-grid">
                  {policy.pillars.map((pillar) => (
                    <PillarCard key={pillar.id} pillar={pillar} />
                  ))}
                </div>
                <div className="workflow-card">
                  <h3>Evaluation workflow</h3>
                  <ol>
                    <li><strong>Criteria</strong> — review or add requirements per pillar</li>
                    <li><strong>Score Matrix</strong> — click cells to set compliance per vendor</li>
                    <li><strong>Comparison</strong> — radar chart, leaderboard, heatmap</li>
                    <li><strong>Report</strong> — download shareable HTML for stakeholders</li>
                  </ol>
                </div>
              </section>
            )}

            {tab === "criteria" && (
              <CriteriaEditor
                pillars={policy.pillars}
                onAdd={addRequirement}
                onDelete={deleteRequirement}
              />
            )}

            {tab === "matrix" && (
              <ScoreMatrix
                pillars={policy.pillars}
                evaluation={evaluation}
                onCycle={cycleAssessment}
              />
            )}

            {tab === "compare" && (
              <ComparisonView evaluation={evaluation} pillars={policy.pillars} />
            )}

            {tab === "vendors" && (
              <section className="section">
                <h2>Vendor Scorecards</h2>
                <p className="section-intro">
                  Severity-weighted scores. Critical gaps disqualify regardless of percentage.
                </p>
                <div className="vendor-grid">
                  {rankedVendors.map((result, i) => (
                    <VendorScorecard key={result.vendor.name} result={result} rank={i + 1} detailed />
                  ))}
                </div>
              </section>
            )}

            {tab === "report" && (
              <TechnicalReport policy={policy} evaluation={evaluation} />
            )}
          </>
        )}
      </main>

      <footer className="footer">
        Operation M.A.D. — Mobile MDM Vendor Evaluation (evaluation only)
      </footer>

      <style>{`
        .app { min-height: 100vh; display: flex; flex-direction: column; }
        .main { flex: 1; max-width: 1200px; margin: 0 auto; padding: 2rem; width: 100%; }
        .loading { text-align: center; color: var(--mad-text-muted); padding: 3rem; }
        .error-banner {
          background: #fde8ea; border: 1px solid var(--mad-critical);
          border-radius: 8px; padding: 1.25rem; margin-bottom: 1.5rem;
        }
        .error-banner code {
          background: rgba(0,0,0,0.06); padding: 0.15rem 0.4rem;
          border-radius: 4px; font-family: var(--font-mono);
        }
        .error-detail { font-size: 0.85rem; color: var(--mad-text-muted); }
        .tabs {
          display: flex; gap: 0.25rem; margin: 1.5rem 0; border-bottom: 2px solid #dde1e6;
          overflow-x: auto; padding-bottom: 0;
        }
        .tab {
          background: none; border: none; padding: 0.75rem 1rem; font-size: 0.85rem;
          font-weight: 600; color: var(--mad-text-muted); cursor: pointer;
          border-bottom: 3px solid transparent; margin-bottom: -2px; white-space: nowrap;
        }
        .tab:hover { color: var(--mad-navy); }
        .tab.active { color: var(--mad-cyan-dim); border-bottom-color: var(--mad-cyan); }
        .section h2 { margin: 0 0 0.5rem; color: var(--mad-navy); }
        .section-intro { color: var(--mad-text-muted); margin: 0 0 1.5rem; max-width: 72ch; }
        .pillar-grid {
          display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 1rem; margin-bottom: 1.5rem;
        }
        .workflow-card {
          background: white; border-radius: 10px; padding: 1.5rem;
          border-left: 4px solid var(--mad-cyan); box-shadow: 0 2px 8px rgba(10,22,40,0.08);
        }
        .workflow-card h3 { margin: 0 0 0.75rem; color: var(--mad-navy); }
        .workflow-card ol { margin: 0; padding-left: 1.25rem; color: var(--mad-text-muted); line-height: 1.8; }
        .vendor-grid { display: flex; flex-direction: column; gap: 1rem; }
        .footer {
          text-align: center; padding: 1.5rem; font-size: 0.8rem;
          color: var(--mad-text-muted); border-top: 1px solid #dde1e6;
        }
      `}</style>
    </div>
  );
}
