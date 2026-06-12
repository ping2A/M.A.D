import { useMemo, useState } from "react";
import { ComparisonView } from "./components/ComparisonView";
import { CriteriaTab } from "./components/CriteriaTab";
import { Header } from "./components/Header";
import { PillarCard } from "./components/PillarCard";
import { ScoreMatrix } from "./components/ScoreMatrix";
import { ScoreOverview } from "./components/ScoreOverview";
import { ProcurementPanel } from "./components/ProcurementPanel";
import { WorkspaceDataPanel } from "./components/WorkspaceDataPanel";
import { ScoringPanel } from "./components/ScoringPanel";
import { StatsBar } from "./components/StatsBar";
import { TechnicalReport } from "./components/TechnicalReport";
import { VendorEditor } from "./components/VendorEditor";
import { ValueStreamView } from "./components/ValueStreamView";
import { VendorDocView } from "./components/VendorDocView";
import { VendorScorecard } from "./components/VendorScorecard";
import { useEvaluation } from "./hooks/useEvaluation";
import { usePolicyContent } from "./hooks/usePolicyContent";
import { useLocale } from "./i18n/LocaleContext";
import { buildFilteredEvaluation } from "./utils/comparisonFilter";
import { rankVendors } from "./utils/ranking";

type Tab =
  | "overview"
  | "vendors"
  | "criteria"
  | "matrix"
  | "compare"
  | "scorecards"
  | "vsm"
  | "docs"
  | "report";

export default function App() {
  const { t } = useLocale();
  const { localizePillars, localizeEvaluation } = usePolicyContent();
  const {
    policy,
    evaluation,
    loading,
    saving,
    error,
    addPillar,
    updatePillar,
    deletePillar,
    addRequirement,
    updateRequirement,
    deleteRequirement,
    addVendor,
    updateVendor,
    deleteVendor,
    setAssessment,
    cycleAssessment,
    updateScoring,
    updateProcurement,
    updateValueStream,
    createValueStream,
    deleteValueStream,
    updateVendorDoc,
    createVendorDoc,
    deleteVendorDoc,
    valueStreams,
    vendorDocs,
    exportWorkspace,
    exportVendors,
    importWorkspace,
    loadExampleVendors,
  } = useEvaluation();

  const [tab, setTab] = useState<Tab>("matrix");
  const [activeTags, setActiveTags] = useState<Set<string>>(() => new Set());

  const displayPillars = useMemo(
    () => (policy ? localizePillars(policy.pillars) : []),
    [policy, localizePillars],
  );

  const displayEvaluation = useMemo(
    () => (evaluation ? localizeEvaluation(evaluation) : null),
    [evaluation, localizeEvaluation],
  );

  const allVendorIds = useMemo(
    () => new Set(displayEvaluation?.vendors.map((v) => v.vendor.id) ?? []),
    [displayEvaluation],
  );

  const tagFilteredEvaluation = useMemo(() => {
    if (!displayEvaluation) return null;
    return buildFilteredEvaluation(displayEvaluation, allVendorIds, activeTags);
  }, [displayEvaluation, allVendorIds, activeTags]);

  const rankedVendors = tagFilteredEvaluation ? rankVendors(tagFilteredEvaluation) : [];

  const tabItems: [Tab, string][] = [
    ["matrix", t.tabs.matrix],
    ["vendors", t.tabs.vendors],
    ["criteria", t.tabs.criteria],
    ["compare", t.tabs.compare],
    ["scorecards", t.tabs.scorecards],
    ["vsm", t.tabs.vsm],
    ["docs", t.tabs.docs],
    ["overview", t.tabs.overview],
    ["report", t.tabs.report],
  ];

  return (
    <div className="app">
      <Header />

      <main className="main">
        {loading && <p className="loading">{t.loading}</p>}
        {error && (
          <div className="error-banner">
            <strong>{t.error.title}</strong>
            <p>
              {t.error.hintBefore}
              <code>cargo run -p mad-server</code>
              {t.error.hintAfter}
            </p>
            <p className="error-detail">{error}</p>
          </div>
        )}

        {policy && evaluation && (
          <>
            <ScoreOverview
              evaluation={displayEvaluation!}
              activeTags={activeTags}
              onActiveTagsChange={setActiveTags}
              onSelectVendor={() => setTab("matrix")}
            />

            <StatsBar
              version={policy.version}
              pillars={policy.pillar_count}
              requirements={policy.total_requirements}
              critical={policy.critical_requirements}
              vendors={evaluation.vendors.length}
            />

            {policy.scoring && (
              <ScoringPanel
                key={policy.scoring.use_severity_weighting ? "sw-on" : "sw-off"}
                scoring={policy.scoring}
                onChange={updateScoring}
                collapsed
              />
            )}

            {policy.procurement && (
              <ProcurementPanel
                procurement={policy.procurement}
                onChange={updateProcurement}
                collapsed
              />
            )}

            <WorkspaceDataPanel
              onExportWorkspace={exportWorkspace}
              onExportVendors={exportVendors}
              onImportWorkspace={importWorkspace}
              stats={{
                pillars: policy.pillar_count,
                requirements: policy.total_requirements,
                vendors: evaluation.vendors.length,
              }}
            />

            <nav className="tabs" role="tablist">
              {tabItems.map(([tKey, label]) => (
                <button
                  key={tKey}
                  type="button"
                  role="tab"
                  aria-selected={tab === tKey}
                  className={`tab ${tab === tKey ? "active" : ""}`}
                  onClick={() => setTab(tKey)}
                >
                  {label}
                </button>
              ))}
            </nav>

            {tab === "overview" && (
              <section className="section">
                <h2 className="section-title">{t.overview.title}</h2>
                <p className="section-intro">{t.overview.intro}</p>
                <div className="pillar-grid">
                  {displayPillars.map((pillar) => (
                    <PillarCard key={pillar.id} pillar={pillar} />
                  ))}
                </div>
                <div className="workflow-card">
                  <h3>{t.overview.workflowTitle}</h3>
                  <ol>
                    <li><strong>{t.tabs.vendors}</strong> — {t.overview.workflowVendors}</li>
                    <li><strong>{t.tabs.criteria}</strong> — {t.overview.workflowCriteria}</li>
                    <li><strong>{t.tabs.matrix}</strong> — {t.overview.workflowMatrix}</li>
                    <li><strong>{t.tabs.compare}</strong> — {t.overview.workflowCompare}</li>
                    <li><strong>{t.tabs.report}</strong> — {t.overview.workflowReport}</li>
                  </ol>
                </div>
              </section>
            )}

            {tab === "vendors" && (
              <VendorEditor
                evaluation={evaluation}
                onAdd={addVendor}
                onUpdate={updateVendor}
                onDelete={deleteVendor}
                onLoadExample={loadExampleVendors}
              />
            )}

            {tab === "criteria" && policy && (
              <CriteriaTab
                pillars={policy.pillars}
                onAddPillar={addPillar}
                onUpdatePillar={updatePillar}
                onDeletePillar={deletePillar}
                onAddRequirement={addRequirement}
                onUpdateRequirement={updateRequirement}
                onDeleteRequirement={deleteRequirement}
              />
            )}

            {tab === "matrix" && (
              <ScoreMatrix
                pillars={displayPillars}
                evaluation={displayEvaluation!}
                onSetStatus={setAssessment}
                onCycle={cycleAssessment}
                saving={saving}
              />
            )}

            {tab === "compare" && (
              <ComparisonView evaluation={displayEvaluation!} pillars={displayPillars} />
            )}

            {tab === "scorecards" && displayEvaluation && (
              <section className="section">
                <h2 className="section-title">{t.scorecards.title}</h2>
                <p className="section-intro">{t.scorecards.intro}</p>
                <div className="vendor-grid">
                  {rankedVendors.map((result, i) => (
                    <VendorScorecard key={result.vendor.name} result={result} rank={i + 1} detailed />
                  ))}
                </div>
              </section>
            )}

            {tab === "vsm" && displayEvaluation && (
              <ValueStreamView
                evaluation={displayEvaluation}
                valueStreams={valueStreams}
                saving={saving}
                onSave={updateValueStream}
                onCreate={createValueStream}
                onDelete={deleteValueStream}
              />
            )}

            {tab === "docs" && displayEvaluation && (
              <VendorDocView
                evaluation={displayEvaluation}
                vendorDocs={vendorDocs}
                saving={saving}
                onSave={updateVendorDoc}
                onCreate={createVendorDoc}
                onDelete={deleteVendorDoc}
              />
            )}

            {tab === "report" && tagFilteredEvaluation && (
              <TechnicalReport
                policy={{ ...policy, pillars: displayPillars }}
                evaluation={tagFilteredEvaluation}
                activeTags={activeTags}
              />
            )}
          </>
        )}
      </main>

      <footer className="footer">{t.footer}</footer>

      <style>{`
        .app { min-height: 100vh; display: flex; flex-direction: column; }
        .main {
          flex: 1;
          max-width: 1400px;
          margin: 0 auto;
          padding: 1.5rem 2rem 2.5rem;
          width: 100%;
        }
        .loading { text-align: center; color: var(--mad-text-muted); padding: 3rem; }
        .error-banner {
          background: #fde8ea;
          border: 1px solid var(--mad-critical);
          border-radius: var(--mad-radius);
          padding: 1.25rem;
          margin-bottom: 1.5rem;
        }
        .error-banner code {
          background: rgba(0,0,0,0.06);
          padding: 0.15rem 0.4rem;
          border-radius: 4px;
          font-family: var(--font-mono);
        }
        .error-detail { font-size: 0.85rem; color: var(--mad-text-muted); }
        .tabs {
          display: flex;
          gap: 0.15rem;
          margin: 1.5rem 0 1.25rem;
          border-bottom: 2px solid var(--mad-border);
          overflow-x: auto;
          padding-bottom: 0;
        }
        .tab {
          background: none;
          border: none;
          padding: 0.75rem 1rem;
          font-size: 0.85rem;
          font-weight: 600;
          color: var(--mad-text-muted);
          cursor: pointer;
          border-bottom: 3px solid transparent;
          margin-bottom: -2px;
          white-space: nowrap;
          font-family: inherit;
        }
        .tab:hover { color: var(--mad-navy); }
        .tab.active {
          color: var(--mad-cyan-dim);
          border-bottom-color: var(--mad-cyan);
        }
        .pillar-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 1rem;
          margin-bottom: 1.5rem;
        }
        .workflow-card {
          background: white;
          border-radius: var(--mad-radius);
          padding: 1.5rem;
          border-left: 4px solid var(--mad-cyan);
          box-shadow: var(--mad-shadow);
        }
        .workflow-card h3 { margin: 0 0 0.75rem; color: var(--mad-navy); }
        .workflow-card ol {
          margin: 0;
          padding-left: 1.25rem;
          color: var(--mad-text-muted);
          line-height: 1.8;
        }
        .vendor-grid { display: flex; flex-direction: column; gap: 1rem; }
        .footer {
          text-align: center;
          padding: 1.5rem;
          font-size: 0.8rem;
          color: var(--mad-text-muted);
          border-top: 1px solid var(--mad-border);
        }
      `}</style>
    </div>
  );
}
