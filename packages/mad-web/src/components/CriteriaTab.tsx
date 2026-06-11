import { useState } from "react";
import { CriteriaEditor } from "./CriteriaEditor";
import { PillarGroupEditor } from "./PillarGroupEditor";
import { useLocale } from "../i18n/LocaleContext";
import type { Pillar, PillarId, Requirement } from "../types";

interface CriteriaTabProps {
  pillars: Pillar[];
  onAddPillar: (id: string, name: string, description: string) => Promise<void>;
  onUpdatePillar: (id: string, name: string, description: string) => Promise<void>;
  onDeletePillar: (id: string) => Promise<void>;
  onAddRequirement: (pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onUpdateRequirement: (id: string, pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onDeleteRequirement: (id: string) => Promise<void>;
}

export function CriteriaTab({
  pillars,
  onAddPillar,
  onUpdatePillar,
  onDeletePillar,
  onAddRequirement,
  onUpdateRequirement,
  onDeleteRequirement,
}: CriteriaTabProps) {
  const { t } = useLocale();
  const [filterPillar, setFilterPillar] = useState<PillarId | "all">("all");

  const totalRequirements = pillars.reduce((n, p) => n + p.requirements.length, 0);

  return (
    <section className="criteria-page">
      <header className="criteria-page-header card">
        <div>
          <h2 className="section-title">{t.criteriaPage.title}</h2>
          <p className="section-intro">{t.criteriaPage.intro}</p>
        </div>
        <div className="criteria-page-stats">
          <span className="stat-pill">
            <strong>{pillars.length}</strong> {t.criteriaPage.groups}
          </span>
          <span className="stat-pill">
            <strong>{totalRequirements}</strong> {t.criteriaPage.requirements}
          </span>
        </div>
      </header>

      <div className="criteria-step">
        <div className="criteria-step-header">
          <span className="step-badge" aria-hidden>1</span>
          <div>
            <h3>{t.criteriaPage.step1Title}</h3>
            <p>{t.criteriaPage.step1Hint}</p>
          </div>
        </div>
        <PillarGroupEditor
          pillars={pillars}
          selectedPillar={filterPillar}
          onSelectPillar={setFilterPillar}
          onAdd={onAddPillar}
          onUpdate={onUpdatePillar}
          onDelete={onDeletePillar}
        />
      </div>

      <div className="criteria-step">
        <div className="criteria-step-header">
          <span className="step-badge" aria-hidden>2</span>
          <div>
            <h3>{t.criteriaPage.step2Title}</h3>
            <p>{t.criteriaPage.step2Hint}</p>
          </div>
        </div>
        <CriteriaEditor
          pillars={pillars}
          filterPillar={filterPillar}
          onFilterPillarChange={setFilterPillar}
          onAdd={onAddRequirement}
          onUpdate={onUpdateRequirement}
          onDelete={onDeleteRequirement}
        />
      </div>
    </section>
  );
}
