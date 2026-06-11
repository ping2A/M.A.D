import { useCallback, useEffect, useState } from "react";
import {
  addPillar,
  addRequirement,
  addVendor,
  deletePillar,
  deleteRequirement,
  deleteVendor,
  exportVendorsJson,
  exportWorkspaceJson,
  getEvaluation,
  getPolicy,
  importVendorsJson,
  importWorkspaceJson,
  setAssessment,
  updatePillar,
  updateProcurement,
  updateValueStream,
  updateRequirement,
  updateScoring,
  updateVendor,
} from "../api/client";
import type {
  ComplianceStatus,
  EvaluationReport,
  PillarId,
  PolicySummary,
  ProcurementConfig,
  Requirement,
  ScoringConfig,
  Vendor,
  VendorImportMode,
  VendorImportResult,
  ValueStreamMap,
  VendorSetFile,
} from "../types";
import { STATUS_CYCLE } from "../types";

export function useEvaluation() {
  const [policy, setPolicy] = useState<PolicySummary | null>(null);
  const [evaluation, setEvaluation] = useState<EvaluationReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
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

  const withSave = async (fn: () => Promise<void>) => {
    setSaving(true);
    try {
      await fn();
      await refresh();
    } finally {
      setSaving(false);
    }
  };

  const handleAddPillar = async (id: string, name: string, description: string) => {
    await withSave(() => addPillar(id, name, description).then(() => undefined));
  };

  const handleUpdatePillar = async (id: string, name: string, description: string) => {
    await withSave(() => updatePillar(id, name, description).then(() => undefined));
  };

  const handleDeletePillar = async (id: string) => {
    await withSave(() => deletePillar(id));
  };

  const handleAddRequirement = async (pillarId: PillarId, requirement: Requirement) => {
    await withSave(() => addRequirement(pillarId, requirement).then(() => undefined));
  };

  const handleUpdateRequirement = async (
    id: string,
    pillarId: PillarId,
    requirement: Requirement,
  ) => {
    await withSave(() => updateRequirement(id, pillarId, requirement).then(() => undefined));
  };

  const handleDeleteRequirement = async (id: string) => {
    await withSave(() => deleteRequirement(id));
  };

  const handleAddVendor = async (vendor: Vendor) => {
    await withSave(() => addVendor(vendor).then(() => undefined));
  };

  const handleUpdateVendor = async (id: string, vendor: Vendor) => {
    await withSave(() => updateVendor(id, vendor).then(() => undefined));
  };

  const handleDeleteVendor = async (id: string) => {
    await withSave(() => deleteVendor(id));
  };

  const handleSetAssessment = async (
    vendorId: string,
    requirementId: string,
    status: ComplianceStatus,
    notes?: string | null,
  ) => {
    await withSave(() =>
      setAssessment(vendorId, requirementId, status, notes ?? undefined).then(() => undefined),
    );
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
    await withSave(() => updateScoring(scoring).then(() => undefined));
  };

  const handleUpdateProcurement = async (procurement: ProcurementConfig) => {
    await withSave(() => updateProcurement(procurement).then(() => undefined));
  };

  const handleUpdateValueStream = async (vendorId: string, map: ValueStreamMap) => {
    setSaving(true);
    try {
      await updateValueStream(vendorId, map);
      setPolicy((prev) =>
        prev
          ? {
              ...prev,
              value_streams: { ...prev.value_streams, [vendorId]: map },
            }
          : prev,
      );
    } catch (err) {
      await refresh();
      throw err;
    } finally {
      setSaving(false);
    }
  };

  const handleExportWorkspace = async () => {
    await exportWorkspaceJson();
  };

  const handleExportVendors = async () => {
    await exportVendorsJson();
  };

  const handleImportWorkspace = async (json: string, vendorMode: VendorImportMode) => {
    setSaving(true);
    try {
      const { result } = await importWorkspaceJson(json, vendorMode);
      await refresh();
      return {
        kind: result.kind,
        pillars: result.pillars,
        requirements: result.requirements,
        vendors: result.vendors,
      };
    } finally {
      setSaving(false);
    }
  };

  const handleImportVendors = async (
    file: VendorSetFile,
    mode: VendorImportMode,
  ): Promise<VendorImportResult> => {
    setSaving(true);
    try {
      const { result } = await importVendorsJson(file, mode);
      await refresh();
      return result;
    } finally {
      setSaving(false);
    }
  };

  return {
    policy,
    evaluation,
    loading,
    saving,
    error,
    refresh,
    addPillar: handleAddPillar,
    updatePillar: handleUpdatePillar,
    deletePillar: handleDeletePillar,
    addRequirement: handleAddRequirement,
    updateRequirement: handleUpdateRequirement,
    deleteRequirement: handleDeleteRequirement,
    addVendor: handleAddVendor,
    updateVendor: handleUpdateVendor,
    deleteVendor: handleDeleteVendor,
    setAssessment: handleSetAssessment,
    cycleAssessment: handleCycleAssessment,
    updateScoring: handleUpdateScoring,
    updateProcurement: handleUpdateProcurement,
    updateValueStream: handleUpdateValueStream,
    valueStreams: policy?.value_streams ?? {},
    exportWorkspace: handleExportWorkspace,
    exportVendors: handleExportVendors,
    importWorkspace: handleImportWorkspace,
    importVendors: handleImportVendors,
  };
}
