import type { BillingPeriod, VendorPricing } from "../types";

export function formatMoney(amount: number, currency = "USD", compact = false): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      notation: compact ? "compact" : "standard",
      maximumFractionDigits: compact ? 1 : 0,
    }).format(amount);
  } catch {
    return `${currency} ${amount.toFixed(0)}`;
  }
}

export function billingPeriodLabel(period: BillingPeriod, t: { monthly: string; annual: string }): string {
  return period === "annual" ? t.annual : t.monthly;
}

export function hasPricingInput(pricing?: VendorPricing | null): boolean {
  if (!pricing) return false;
  return pricing.price_per_device != null || pricing.global_price != null;
}

export function pricingSummary(
  pricing: VendorPricing,
  labels: { perDevice: string; global: string; monthly: string; annual: string },
): string {
  const period = pricing.billing_period === "annual" ? labels.annual : labels.monthly;
  const parts: string[] = [];
  if (pricing.price_per_device != null) {
    parts.push(`${labels.perDevice}: ${formatMoney(pricing.price_per_device, pricing.currency)}/${period}`);
  }
  if (pricing.global_price != null) {
    parts.push(`${labels.global}: ${formatMoney(pricing.global_price, pricing.currency)}/${period}`);
  }
  return parts.join(" · ");
}
