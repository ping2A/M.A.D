# Operation M.A.D. — Technical Report

**Mobile MDM Vendor Evaluation Platform**

---

## 1. Purpose and Scope

Operation M.A.D. (**M**obile **A**ssessment & **D**ecision) is an **evaluation-only** system. It exists to help security and IT teams **select** a corporate Mobile Device Management (MDM) solution for **iOS and Android** devices.

| In scope | Out of scope |
|----------|--------------|
| iOS MDM (ABM, supervised mode) | Desktop/laptop UEM |
| Android Enterprise (Work Profile, COBO, kiosk) | Post-procurement policy enforcement |
| Vendor capability assessment | Device deployment or ongoing management |
| Policy-as-Code requirement definitions | Network security appliances, CASB |

The platform does **not** enroll devices, push configuration profiles, or enforce compliance in production. It produces structured evaluation reports that inform procurement decisions.

---

## 2. How Evaluation Works

### 2.1 Policy-as-Code pipeline

Requirements are declared in version-controlled YAML (`policies/mad-standard.yaml`). The Rust `mad-core` crate loads policies, applies vendor assessments, and produces scored reports consumed by the CLI, REST API, and web dashboard.

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│ policies/*.yaml │────▶│ PolicyBundle │────▶│  Evaluator  │
└─────────────────┘     └──────────────┘     └──────┬──────┘
                                                      │
┌─────────────────┐     ┌──────────────┐              │
│ VendorAssessment│────▶│ per-req      │◀─────────────┘
│ (compliance map)│     │ status       │
└─────────────────┘     └──────┬───────┘
                                 │
                    ┌────────────▼────────────┐
                    │   EvaluationReport      │
                    │  (scores + gap analysis)│
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
         mad-cli            mad-server           mad-web
      (table/json/md)       (REST API)         (dashboard)
```

### 2.2 Compliance statuses

Each requirement receives exactly one status per vendor:

| Status | Weight | Definition |
|--------|--------|------------|
| `compliant` | 1.0 | Native capability, no workarounds |
| `partial` | 0.5 | Limited, platform-specific, or manual |
| `non_compliant` | 0.0 | Cannot be met |
| `untested` | 0.0 | No data recorded |

### 2.3 Scoring

**Pillar score:**

```
score = ((compliant × 1.0) + (partial × 0.5)) / total × 100
```

**Overall vendor score:** arithmetic mean of the three pillar scores.

**Critical gap flag:** any `critical`-severity requirement that is `non_compliant` or `untested` is listed as a disqualifying gap, regardless of percentage.

### 2.4 Technical verification (production evaluations)

Sample data ships with the repository for demonstration. A production evaluation replaces sample assessments with:

1. **API probes** — query vendor REST/Graph APIs for capability flags
2. **Lab enrollment** — test devices enrolled per deployment model (ABM, Work Profile, COBO)
3. **Controlled tests** — jailbreak/root simulation, network isolation trigger, silent log pull
4. **Evidence capture** — screenshots, API responses, and timestamps stored per requirement

Each requirement in the policy file includes `evaluation_method` (how to test) and `technical_criteria` (APIs, payloads, protocols involved).

---

## 3. Evaluation Pillars

### 3.1 Cybersecurity & Data Loss Prevention

Assesses whether the MDM can enforce mobile data boundaries and respond to compromise.

| ID | Requirement | Key technical signals |
|----|-------------|----------------------|
| `dlp-001` | Hardware-backed containerization | Android Work Profile DPC, iOS Managed Apps, TEE key binding |
| `dlp-002` | IdP Zero Trust conditional access | Graph compliance API, Okta device trust, SAML device claims |
| `dlp-003` | Jailbreak/root detection + remediation | Compliance policy triggers, selective wipe, retire action |

### 3.2 Digital Forensics & Incident Response

Assesses IR capabilities without destroying forensic evidence.

| ID | Requirement | Key technical signals |
|----|-------------|----------------------|
| `dfir-001` | Network isolation preserving RAM | Per-app VPN block, no reboot/wipe commands |
| `dfir-002` | Silent non-destructive triage | Supervised log stream, background bugreport |
| `dfir-003` | SIEM audit streaming | REST/webhook audit feed, CEF/ECS mapping |

### 3.3 Platform & OS Native Support

Assesses depth of native mobile platform integration.

| ID | Requirement | Key technical signals |
|----|-------------|----------------------|
| `plat-001` | ABM + iOS Supervised Mode | DEP enrollment, `IsSupervised`, restricted payloads |
| `plat-002` | Android Enterprise modes | Work Profile, Device Owner, COSU kiosk |
| `plat-003` | OEMConfig | Knox/Zebra managed configurations via MDM |

---

## 4. Architecture

### 4.1 Rust crates

| Crate | Responsibility |
|-------|----------------|
| `mad-core` | Policy parsing, pillar models, scoring engine, Markdown report renderer |
| `mad-cli` | `policy`, `evaluate`, `report` commands |
| `mad-server` | Axum REST API (`/api/policy`, `/api/evaluation`) |

### 4.2 TypeScript package

| Package | Responsibility |
|---------|----------------|
| `mad-web` | React dashboard: pillar browser, vendor scorecard, technical report view |

### 4.3 API endpoints

```
GET /api/health       → { status, name }
GET /api/policy       → pillars, requirements, technical metadata
GET /api/evaluation   → vendor scores, per-requirement status, critical gaps
```

---

## 5. Generating Reports

### Shareable HTML (recommended)

```bash
# Single self-contained file — inline CSS + embedded logo
cargo run -p mad-cli -- report --format html -o mad-evaluation-report.html
npm run report   # → reports/mad-evaluation-report.html
```

The HTML file can be shared via email, Teams, or file share with no server or dependencies. It includes print-friendly styles.

### Markdown

```bash
cargo run -p mad-cli -- report --format md -o report.md
```

### API

```
GET /api/report.html   → self-contained HTML report
```

### Web dashboard

Start `mad-server` and `npm run dev`. Open the **Technical Report** tab and click **Download HTML Report**, or browse to `http://localhost:3001/api/report.html`.

---

## 6. Extending Evaluations

To add a new MDM vendor:

1. Register the vendor in assessment data (or a future `vendors/*.yaml` file)
2. Map each requirement ID to a `ComplianceStatus` with evidence notes
3. Re-run `mad evaluate` or `mad report`

To add a new requirement:

1. Add an entry under the appropriate pillar in `policies/mad-standard.yaml`
2. Include `evaluation_method` and `technical_criteria`
3. Update vendor assessments for the new requirement ID

---

*Operation M.A.D. v0.1.0 — evaluation platform for mobile MDM procurement decisions.*
