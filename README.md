# Operation M.A.D.

**Mobile MDM Vendor Evaluation Platform**

![Operation M.A.D. logo](assets/logo.png)

Operation M.A.D. (**M**obile **A**ssessment & **D**ecision) evaluates mobile Device Management (MDM) solutions for **iOS and Android** only. Using **Policy-as-Code**, it tests vendors (Intune, Jamf, Workspace ONE, etc.) against Cybersecurity, DFIR, and Platform requirements — producing detailed technical reports to support procurement decisions.

> **This is an evaluation tool.** It does not deploy policies, manage devices, or enforce compliance in production.

## Documentation

- **[Technical Report](docs/TECHNICAL_REPORT.md)** — methodology, scoring, architecture, and how evaluation works
- **[Policy standard](policies/mad-standard.yaml)** — requirements with evaluation methods and technical criteria

## Evaluation Pillars

| Pillar | What it tests |
|--------|---------------|
| **Cybersecurity & DLP** | Containerization, IdP conditional access, jailbreak/root remediation |
| **Digital Forensics & IR** | Network isolation, silent triage, SIEM audit streaming |
| **Platform & OS Native Support** | ABM/Supervised iOS, Android Enterprise modes, OEMConfig |

## Architecture

```
M.A.D/
├── policies/           # Policy-as-Code YAML (requirements + test criteria)
├── docs/               # Technical report
├── crates/
│   ├── mad-core/       # Evaluation engine (Rust)
│   ├── mad-cli/        # CLI: policy, evaluate, report
│   └── mad-server/     # REST API
└── packages/
    └── mad-web/        # Evaluation dashboard (TypeScript / React)
```

## Quick Start

```bash
npm install && cargo build

# API server
cargo run -p mad-server

# Web dashboard (separate terminal)
npm run dev
```

Open `http://localhost:5173` and use the evaluation workflow:

1. **Criteria** — add or remove evaluation requirements
2. **Score Matrix** — click cells to set vendor compliance per criterion
3. **Comparison** — radar chart, leaderboard, pillar bars, heatmap
4. **Report** — download shareable HTML for stakeholders

Scores use **severity weighting** (critical ×3, high ×2, medium ×1) so the most important mobile security requirements drive the ranking.

## CLI

```bash
cargo run -p mad-cli -- policy              # list requirements
cargo run -p mad-cli -- evaluate            # vendor scorecard

# Shareable HTML report (self-contained, logo embedded)
cargo run -p mad-cli -- report --format html -o report.html
npm run report   # writes reports/mad-evaluation-report.html

# Markdown report
cargo run -p mad-cli -- report --format md -o report.md
```

The HTML report is a **single file** with inline CSS and an embedded logo — open in any browser, attach to email, or upload to SharePoint with no dependencies.

From the web dashboard, use **Download HTML Report** on the Technical Report tab (requires `mad-server` running).

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).
