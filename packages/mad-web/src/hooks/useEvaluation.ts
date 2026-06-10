import { useCallback, useEffect, useState } from "react";
import {
  addRequirement,
  deleteRequirement,
  getEvaluation,
  getPolicy,
  setAssessment,
  updateScoring,
} from "../api/client";
import type {
  ComplianceStatus,
  EvaluationReport,
  PillarId,
  PolicySummary,
  Requirement,
  ScoringConfig,
} from "../types";
import { STATUS_CYCLE } from "../types";

export function useEvaluation() {
  const [policy, setPolicy] = useState<PolicySummary | null>(null);
  const [evaluation, setEvaluation] = useState<EvaluationReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [p, e] = await Promise.all([getPolicy(), getEvaluation()]);
    setPolicy(p);
    setEvaluation(e);
  }, []);

  useEffect(() => {
    refresh()
      .catch((err: Error) => setError(err.message))
      .finally(() => setLoading(false));
  }, [refresh]);

  const handleAddRequirement = async (pillarId: PillarId, requirement: Requirement) => {
    await addRequirement(pillarId, requirement);
    await refresh();
  };

  const handleDeleteRequirement = async (id: string) => {
    await deleteRequirement(id);
    await refresh();
  };

  const handleSetAssessment = async (
    vendorId: string,
    requirementId: string,
    status: ComplianceStatus,
  ) => {
    await setAssessment(vendorId, requirementId, status);
    await refresh();
  };

  const handleCycleAssessment = async (vendorId: string, requirementId: string) => {
    if (!evaluation) return;
    const vendor = evaluation.vendors.find((v) => v.vendor.id === vendorId);
    if (!vendor) return;
    let current: ComplianceStatus = "untested";
    for (const pillar of vendor.pillars) {
      const req = pillar.requirements.find((r) => r.requirement_id === requirementId);
      if (req) {
        current = req.status;
        break;
      }
    }
    const idx = STATUS_CYCLE.indexOf(current);
    const next = STATUS_CYCLE[(idx + 1) % STATUS_CYCLE.length];
    await handleSetAssessment(vendorId, requirementId, next);
  };

  const handleUpdateScoring = async (scoring: ScoringConfig) => {
    await updateScoring(scoring);
    await refresh();
  };

  return {
    policy,
    evaluation,
    loading,
    error,
    refresh,
    addRequirement: handleAddRequirement,
    deleteRequirement: handleDeleteRequirement,
    setAssessment: handleSetAssessment,
    cycleAssessment: handleCycleAssessment,
    updateScoring: handleUpdateScoring,
  };
}
