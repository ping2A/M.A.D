mod html;

use crate::evaluation::EvaluationReport;
use crate::policy::PolicyBundle;

pub use html::{load_logo_data_uri, render_html, HtmlReportOptions};

/// Renders a detailed technical evaluation report in Markdown.
pub fn render_markdown(bundle: &PolicyBundle, evaluation: &EvaluationReport) -> String {
    let mut out = String::new();

    out.push_str("# Operation M.A.D. — Technical Evaluation Report\n\n");
    out.push_str("**Scope:** Mobile Device Management (MDM) solutions for iOS and Android only.\n\n");
    out.push_str(&format!(
        "**Policy version:** {}  \n**Requirements evaluated:** {} ({} critical)\n\n",
        evaluation.policy_version,
        evaluation.total_requirements,
        evaluation.critical_requirements
    ));

    out.push_str("---\n\n");
    out.push_str("## 1. Purpose and Scope\n\n");
    out.push_str(
        "Operation M.A.D. is an **evaluation-only** platform. It does not deploy, manage, \
         or enforce policies on end-user devices. Its sole function is to assess whether \
         candidate MDM vendors can meet a corporate mobile security standard before procurement.\n\n\
         **In scope:** iOS and Android MDM/UEM platforms (e.g., Microsoft Intune, Jamf Pro, \
         VMware Workspace ONE).\n\n\
         **Out of scope:** Desktop/laptop management, SaaS CASB, network firewalls, \
         post-selection continuous compliance enforcement.\n\n",
    );

    out.push_str("## 2. Evaluation Methodology\n\n");
    out.push_str(
        "Vendors are assessed using **Policy-as-Code (PaC)**. Requirements are declared in \
         version-controlled YAML (`policies/mad-standard.yaml`). Each requirement maps to \
         a discrete compliance status:\n\n\
         | Status | Meaning |\n\
         |--------|----------|\n\
         | `compliant` | Vendor natively satisfies the requirement without third-party workarounds |\n\
         | `partial` | Capability exists but is limited, platform-specific, or requires manual steps |\n\
         | `non_compliant` | Requirement cannot be met with current vendor capabilities |\n\
         | `untested` | No assessment data recorded |\n\n",
    );

    out.push_str("### Scoring algorithm\n\n");
    out.push_str(
        "Per-pillar score:\n\n\
         ```\n\
         score = ((compliant × 1.0) + (partial × 0.5)) / total_requirements × 100\n\
         ```\n\n\
         Overall vendor score is the **arithmetic mean** of the three pillar scores. \
         Critical requirements that are `non_compliant` or `untested` are flagged as \
         **critical gaps** regardless of percentage score.\n\n",
    );

    out.push_str("### Data flow\n\n");
    out.push_str(
        "```\n\
policies/*.yaml  →  PolicyBundle (mad-core)  →  Evaluator\n\
                                                      ↓\n\
VendorAssessment (per requirement status)  →  EvaluationReport\n\
                                                      ↓\n\
                              CLI / REST API / Web dashboard\n\
```\n\n",
    );

    out.push_str("---\n\n");
    out.push_str("## 3. Evaluation Pillars and Technical Criteria\n\n");

    for pillar in &bundle.pillars {
        out.push_str(&format!("### {}\n\n", pillar.name));
        out.push_str(&format!("{}\n\n", pillar.description.trim()));

        for req in &pillar.requirements {
            let severity = match req.severity {
                crate::pillar::RequirementSeverity::Critical => "CRITICAL",
                crate::pillar::RequirementSeverity::High => "HIGH",
                crate::pillar::RequirementSeverity::Medium => "MEDIUM",
            };
            out.push_str(&format!(
                "#### [{severity}] `{}` — {}\n\n",
                req.id, req.title
            ));
            out.push_str(&format!("{}\n\n", req.description.trim()));
            out.push_str(&format!(
                "- **Platforms:** {}\n",
                req.platforms.join(", ")
            ));
            if let Some(method) = &req.evaluation_method {
                out.push_str(&format!("- **Evaluation method:** {method}\n"));
            }
            if let Some(criteria) = &req.technical_criteria {
                out.push_str(&format!("- **Technical criteria:** {criteria}\n"));
            }
            out.push('\n');
        }
    }

    out.push_str("---\n\n");
    out.push_str("## 4. Vendor Assessment Results\n\n");

    let mut ranked: Vec<_> = evaluation.vendors.iter().collect();
    ranked.sort_by(|a, b| {
        b.overall_score
            .overall_score_percent
            .partial_cmp(&a.overall_score.overall_score_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (rank, result) in ranked.iter().enumerate() {
        out.push_str(&format!(
            "### #{} {} — {:.1}% overall\n\n",
            rank + 1,
            result.vendor.name,
            result.overall_score.overall_score_percent
        ));
        out.push_str(&format!("{}\n\n", result.vendor.description));

        for pillar in &result.pillars {
            out.push_str(&format!(
                "#### {} — {:.1}%\n\n",
                pillar.pillar_name, pillar.score.score_percent
            ));
            out.push_str("| Requirement | Status | Notes |\n");
            out.push_str("|-------------|--------|-------|\n");
            for req in &pillar.requirements {
                let status = format!("{:?}", req.status).to_lowercase();
                let notes = req.notes.as_deref().unwrap_or("—");
                out.push_str(&format!(
                    "| `{}` {} | {} | {} |\n",
                    req.requirement_id, req.title, status, notes
                ));
            }
            out.push('\n');
        }

        if !result.overall_score.critical_gaps.is_empty() {
            out.push_str("**Critical gaps:**\n\n");
            for gap in &result.overall_score.critical_gaps {
                out.push_str(&format!("- {gap}\n"));
            }
            out.push('\n');
        }
    }

    out.push_str("---\n\n");
    out.push_str(
        "*Generated by Operation M.A.D. (mad-core). This report reflects sample assessments \
         for demonstration; production evaluations require live API probes and lab validation.*\n",
    );

    out
}

/// Builds evaluation report options with an optional embedded logo and current timestamp.
pub fn default_html_options(logo_path: Option<&std::path::Path>) -> HtmlReportOptions {
    HtmlReportOptions {
        logo_data_uri: logo_path.and_then(load_logo_data_uri),
        generated_at: Some(now_rfc3339()),
    }
}

fn now_rfc3339() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
}
