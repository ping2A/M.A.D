import type { EvaluationReport, EvaluationResult } from "../types";

/** Score used for leaderboard / chip ordering. */
export function rankScore(result: EvaluationResult, evaluation: EvaluationReport): number {
  const procurement = evaluation.procurement;
  if (
    procurement?.use_price_in_ranking &&
    result.overall_score.composite_score_percent != null
  ) {
    return result.overall_score.composite_score_percent;
  }
  return result.overall_score.overall_score_percent;
}

export function rankVendors(evaluation: EvaluationReport): EvaluationResult[] {
  return [...evaluation.vendors].sort((a, b) => rankScore(b, evaluation) - rankScore(a, evaluation));
}

export function usesCompositeRanking(evaluation: EvaluationReport): boolean {
  return Boolean(evaluation.procurement?.use_price_in_ranking);
}
