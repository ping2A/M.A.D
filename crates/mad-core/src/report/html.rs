use crate::evaluation::EvaluationReport;
use crate::policy::PolicyBundle;
use crate::vendor::ComplianceStatus;

/// Options for HTML report generation.
#[derive(Debug, Clone, Default)]
pub struct HtmlReportOptions {
    /// `data:image/png;base64,...` URI embedded in the document.
    pub logo_data_uri: Option<String>,
    /// ISO-8601 timestamp shown in the report footer.
    pub generated_at: Option<String>,
}

pub fn load_logo_data_uri(path: &std::path::Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Some(format!("data:image/png;base64,{encoded}"))
}

/// Renders a self-contained HTML report (inline CSS, optional embedded logo).
/// Suitable for sharing as a single file via email, SharePoint, or file share.
pub fn render_html(
    bundle: &PolicyBundle,
    evaluation: &EvaluationReport,
    options: &HtmlReportOptions,
) -> String {
    let generated_at = options
        .generated_at
        .as_deref()
        .unwrap_or("—");

    let mut ranked: Vec<_> = evaluation.vendors.iter().collect();
    ranked.sort_by(|a, b| {
        b.overall_score
            .overall_score_percent
            .partial_cmp(&a.overall_score.overall_score_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut out = String::with_capacity(64 * 1024);
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"UTF-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    out.push_str("<title>Operation M.A.D. — MDM Evaluation Report</title>\n");
    out.push_str("<style>\n");
    out.push_str(HTML_STYLES);
    out.push_str("</style>\n</head>\n<body>\n");

    // Header
    out.push_str("<header class=\"header\">\n  <div class=\"header-inner\">\n");
    if let Some(logo) = &options.logo_data_uri {
        out.push_str("    <img class=\"logo\" src=\"");
        out.push_str(logo);
        out.push_str("\" alt=\"Operation M.A.D. logo\">\n");
    }
    out.push_str("    <div>\n");
    out.push_str("      <h1>Operation M.A.D.</h1>\n");
    out.push_str("      <p class=\"subtitle\">Mobile MDM Vendor Evaluation Report</p>\n");
    out.push_str("    </div>\n  </div>\n</header>\n");

    out.push_str("<main class=\"container\">\n");

    // Meta bar
    out.push_str("<div class=\"meta-bar\">\n");
    out.push_str(&format!(
        "  <span><strong>Policy</strong> v{}</span>\n",
        escape_html(&evaluation.policy_version)
    ));
    out.push_str(&format!(
        "  <span><strong>Requirements</strong> {} ({} critical)</span>\n",
        evaluation.total_requirements, evaluation.critical_requirements
    ));
    out.push_str(&format!(
        "  <span><strong>Vendors</strong> {}</span>\n",
        evaluation.vendors.len()
    ));
    out.push_str(&format!(
        "  <span><strong>Generated</strong> {}</span>\n",
        escape_html(generated_at)
    ));
    out.push_str("  <span class=\"scope-tag\">iOS &amp; Android MDM only</span>\n");
    out.push_str("</div>\n");

    // Section 1: Purpose
    out.push_str("<section class=\"card\">\n");
    out.push_str("  <h2>1. Purpose and Scope</h2>\n");
    out.push_str("  <p>Operation M.A.D. is an <strong>evaluation-only</strong> platform. \
        It assesses whether candidate MDM vendors meet a corporate mobile security standard \
        before procurement. It does not enroll devices or enforce policies.</p>\n");
    out.push_str("  <div class=\"scope-grid\">\n");
    out.push_str("    <div class=\"scope-in\"><h3>In scope</h3><ul>\
        <li>iOS MDM (ABM, supervised mode)</li>\
        <li>Android Enterprise (Work Profile, COBO, kiosk)</li>\
        <li>Vendor capability assessment</li></ul></div>\n");
    out.push_str("    <div class=\"scope-out\"><h3>Out of scope</h3><ul>\
        <li>Desktop / laptop management</li>\
        <li>Post-selection policy enforcement</li>\
        <li>Device deployment</li></ul></div>\n");
    out.push_str("  </div>\n</section>\n");

    // Section 2: Methodology
    out.push_str("<section class=\"card\">\n");
    out.push_str("  <h2>2. Evaluation Methodology</h2>\n");
    out.push_str("  <table class=\"data-table\">\n");
    out.push_str("    <thead><tr><th>Status</th><th>Weight</th><th>Meaning</th></tr></thead>\n");
    out.push_str("    <tbody>\n");
    for (status, weight, meaning) in [
        ("compliant", "1.0", "Native capability, no workarounds"),
        ("partial", "0.5", "Limited, platform-specific, or manual"),
        ("non_compliant", "0.0", "Cannot be met"),
        ("untested", "0.0", "No assessment data"),
    ] {
        out.push_str(&format!(
            "      <tr><td><span class=\"badge status-{status}\">{status}</span></td>\
             <td>{weight}</td><td>{meaning}</td></tr>\n"
        ));
    }
    out.push_str("    </tbody>\n  </table>\n");
    out.push_str("  <pre class=\"code\">pillar_score = ((compliant × 1.0) + (partial × 0.5)) / total × 100\n\
overall_score  = mean(cybersecurity, dfir, platform_os)</pre>\n");
    out.push_str("</section>\n");

    // Section 3: Requirements
    out.push_str("<section class=\"card\">\n");
    out.push_str("  <h2>3. Requirements and Technical Criteria</h2>\n");
    for pillar in &bundle.pillars {
        out.push_str(&format!(
            "  <h3 class=\"pillar-title\">{}</h3>\n  <p class=\"muted\">{}</p>\n",
            escape_html(&pillar.name),
            escape_html(pillar.description.trim())
        ));
        for req in &pillar.requirements {
            let severity = severity_label(req.severity);
            out.push_str("  <div class=\"requirement\">\n");
            out.push_str(&format!(
                "    <div class=\"req-header\">\
                 <code class=\"req-id\">{}</code>\
                 <span class=\"badge severity-{}\">{}</span>\
                 <strong>{}</strong></div>\n",
                escape_html(&req.id),
                severity.to_lowercase(),
                severity,
                escape_html(&req.title)
            ));
            out.push_str(&format!(
                "    <p>{}</p>\n",
                escape_html(req.description.trim())
            ));
            out.push_str(&format!(
                "    <p class=\"muted\"><strong>Platforms:</strong> {}</p>\n",
                escape_html(&req.platforms.join(", "))
            ));
            if let Some(m) = &req.evaluation_method {
                out.push_str(&format!(
                    "    <div class=\"tech-box\"><strong>Evaluation method</strong><br>{}</div>\n",
                    escape_html(m.trim())
                ));
            }
            if let Some(c) = &req.technical_criteria {
                out.push_str(&format!(
                    "    <div class=\"tech-box\"><strong>Technical criteria</strong><br>{}</div>\n",
                    escape_html(c.trim())
                ));
            }
            out.push_str("  </div>\n");
        }
    }
    out.push_str("</section>\n");

    // Section 4: Vendor results
    out.push_str("<section class=\"card\">\n");
    out.push_str("  <h2>4. Vendor Assessment Results</h2>\n");
    for (rank, result) in ranked.iter().enumerate() {
        let score = result.overall_score.overall_score_percent;
        out.push_str(&format!(
            "  <article class=\"vendor-card\">\n\
               <div class=\"vendor-header\">\n\
                 <span class=\"rank\">#{}</span>\n\
                 <h3>{}</h3>\n\
                 <span class=\"score\" style=\"color:{}\">{:.1}%</span>\n\
               </div>\n\
               <p class=\"muted\">{}</p>\n",
            rank + 1,
            escape_html(&result.vendor.name),
            score_color(score),
            score,
            escape_html(&result.vendor.description)
        ));

        for pillar in &result.pillars {
            let ps = pillar.score.score_percent;
            out.push_str(&format!(
                "    <h4>{} — <span style=\"color:{}\">{:.1}%</span></h4>\n",
                escape_html(&pillar.pillar_name),
                score_color(ps),
                ps
            ));
            out.push_str("    <table class=\"data-table compact\">\n");
            out.push_str("      <thead><tr><th>ID</th><th>Requirement</th><th>Status</th><th>Notes</th></tr></thead>\n");
            out.push_str("      <tbody>\n");
            for req in &pillar.requirements {
                let status = status_str(req.status);
                out.push_str(&format!(
                    "        <tr><td><code>{}</code></td><td>{}</td>\
                     <td><span class=\"badge status-{}\">{}</span></td>\
                     <td>{}</td></tr>\n",
                    escape_html(&req.requirement_id),
                    escape_html(&req.title),
                    status,
                    status,
                    escape_html(req.notes.as_deref().unwrap_or("—"))
                ));
            }
            out.push_str("      </tbody>\n    </table>\n");
        }

        if !result.overall_score.critical_gaps.is_empty() {
            out.push_str("    <div class=\"gaps\"><strong>Critical gaps</strong><ul>\n");
            for gap in &result.overall_score.critical_gaps {
                out.push_str(&format!("      <li>{}</li>\n", escape_html(gap)));
            }
            out.push_str("    </ul></div>\n");
        }
        out.push_str("  </article>\n");
    }
    out.push_str("</section>\n");

    out.push_str("</main>\n");
    out.push_str("<footer class=\"footer\">Generated by Operation M.A.D. — Mobile MDM Vendor Evaluation. \
        Sample assessments for demonstration; production evaluations require lab validation.</footer>\n");
    out.push_str("</body>\n</html>\n");

    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn severity_label(severity: crate::pillar::RequirementSeverity) -> &'static str {
    match severity {
        crate::pillar::RequirementSeverity::Critical => "CRITICAL",
        crate::pillar::RequirementSeverity::High => "HIGH",
        crate::pillar::RequirementSeverity::Medium => "MEDIUM",
    }
}

fn status_str(status: ComplianceStatus) -> &'static str {
    match status {
        ComplianceStatus::Compliant => "compliant",
        ComplianceStatus::Partial => "partial",
        ComplianceStatus::NonCompliant => "non_compliant",
        ComplianceStatus::Untested => "untested",
    }
}

fn score_color(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "#28a745"
    } else if pct >= 70.0 {
        "#e6a800"
    } else {
        "#dc3545"
    }
}

const HTML_STYLES: &str = r#"
:root {
  --navy: #0a1628;
  --navy-light: #132238;
  --cyan: #00b4d8;
  --silver: #c0c8d4;
  --text: #1a2332;
  --muted: #5a6578;
  --bg: #e8eaed;
  --compliant: #28a745;
  --partial: #e6a800;
  --gap: #dc3545;
  --font: "Segoe UI", system-ui, -apple-system, sans-serif;
  --mono: "Cascadia Code", "Fira Code", ui-monospace, monospace;
}
* { box-sizing: border-box; margin: 0; }
body { font-family: var(--font); color: var(--text); background: var(--bg); line-height: 1.5; }
.header {
  background: linear-gradient(135deg, var(--navy) 0%, #1e3a5f 100%);
  color: white; padding: 1.5rem 2rem; border-bottom: 3px solid var(--cyan);
}
.header-inner { max-width: 960px; margin: 0 auto; display: flex; align-items: center; gap: 1.25rem; }
.logo { width: 64px; height: 64px; border-radius: 8px; border: 2px solid var(--cyan); }
.header h1 { font-size: 1.5rem; }
.subtitle { color: var(--silver); font-size: 0.9rem; margin-top: 0.2rem; }
.container { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
.meta-bar {
  display: flex; flex-wrap: wrap; gap: 1rem; background: var(--navy-light); color: white;
  padding: 1rem 1.25rem; border-radius: 8px; margin-bottom: 1.25rem; font-size: 0.85rem;
}
.meta-bar strong { color: var(--cyan); }
.scope-tag {
  background: var(--cyan); color: var(--navy); padding: 0.15rem 0.6rem;
  border-radius: 4px; font-weight: 600; font-size: 0.75rem;
}
.card {
  background: white; border-radius: 10px; padding: 1.5rem; margin-bottom: 1.25rem;
  box-shadow: 0 2px 8px rgba(10,22,40,0.08);
}
.card h2 { color: var(--navy); font-size: 1.15rem; border-bottom: 2px solid var(--cyan);
  padding-bottom: 0.5rem; margin-bottom: 1rem; }
.card h3, .pillar-title { color: var(--navy); font-size: 1rem; margin: 1.25rem 0 0.5rem; }
.card h4 { color: var(--navy); font-size: 0.9rem; margin: 1rem 0 0.5rem; }
.muted { color: var(--muted); font-size: 0.9rem; }
.scope-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1rem; }
@media (max-width: 600px) { .scope-grid { grid-template-columns: 1fr; } }
.scope-in, .scope-out { padding: 1rem; border-radius: 8px; font-size: 0.9rem; }
.scope-in { background: #e8f5e9; border-left: 4px solid var(--compliant); }
.scope-out { background: #fde8ea; border-left: 4px solid var(--gap); }
.scope-in h3, .scope-out h3 { font-size: 0.8rem; text-transform: uppercase; margin-bottom: 0.5rem; }
.scope-in ul, .scope-out ul { padding-left: 1.25rem; }
.data-table { width: 100%; border-collapse: collapse; font-size: 0.85rem; margin: 1rem 0; }
.data-table th, .data-table td { padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #e2e6ea; }
.data-table th { background: var(--navy-light); color: white; }
.data-table.compact { font-size: 0.8rem; }
.code {
  background: var(--navy); color: var(--cyan); padding: 1rem; border-radius: 8px;
  font-family: var(--mono); font-size: 0.8rem; overflow-x: auto; margin-top: 1rem;
}
.badge {
  display: inline-block; font-size: 0.7rem; font-weight: 700; text-transform: uppercase;
  padding: 0.1rem 0.45rem; border-radius: 3px;
}
.severity-critical { background: #fde8ea; color: var(--gap); }
.severity-high { background: #fff3e0; color: #fd7e14; }
.severity-medium { background: #fff8e1; color: #b8860b; }
.status-compliant { background: #e8f5e9; color: var(--compliant); }
.status-partial { background: #fff8e1; color: var(--partial); }
.status-non_compliant { background: #fde8ea; color: var(--gap); }
.status-untested { background: #eceff1; color: var(--muted); }
.requirement { padding: 1rem 0; border-bottom: 1px solid #e8eaed; }
.req-header { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.4rem; }
.req-id { font-family: var(--mono); font-size: 0.75rem; background: var(--navy-light);
  color: var(--cyan); padding: 0.1rem 0.4rem; border-radius: 3px; }
.tech-box {
  font-size: 0.8rem; color: var(--muted); background: #f4f6f8; padding: 0.5rem 0.75rem;
  border-radius: 4px; margin-top: 0.5rem; line-height: 1.5;
}
.vendor-card {
  border: 1px solid #e2e6ea; border-left: 4px solid var(--cyan); border-radius: 8px;
  padding: 1.25rem; margin-bottom: 1rem;
}
.vendor-header { display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap; }
.rank { font-size: 1.25rem; font-weight: 800; color: var(--navy); opacity: 0.3; }
.vendor-header h3 { flex: 1; font-size: 1.1rem; color: var(--navy); }
.score { font-size: 1.4rem; font-weight: 800; font-family: var(--mono); }
.gaps { background: #fde8ea; padding: 0.75rem 1rem; border-radius: 6px; margin-top: 1rem; font-size: 0.85rem; }
.gaps ul { padding-left: 1.25rem; margin-top: 0.4rem; }
.footer {
  text-align: center; padding: 1.5rem; font-size: 0.8rem; color: var(--muted);
  border-top: 1px solid #dde1e6;
}
@media print {
  body { background: white; }
  .card { box-shadow: none; border: 1px solid #ddd; break-inside: avoid; }
  .header { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
}
"#;
