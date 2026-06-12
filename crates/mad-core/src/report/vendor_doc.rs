use std::collections::HashSet;

use crate::report::locale::{self, ReportLocale};
use crate::vendor_doc::{resolve_doc_color, VendorDocItem, VendorDocSection};

pub use crate::vendor_doc::any_vendor_docs;

pub fn render_vendor_doc_markdown(
    vendor_name: &str,
    section: &VendorDocSection,
) -> String {
    if section.is_empty() {
        return String::new();
    }
    let mut out = format!("### {vendor_name} — {}\n\n", section.name);
    if let Some(overview) = section.overview.as_deref().filter(|s| !s.trim().is_empty()) {
        out.push_str(&format!("{overview}\n\n"));
    }
    for group in groups_in_order(&section.items) {
        let items: Vec<_> = section
            .items
            .iter()
            .filter(|i| item_group(i) == group)
            .collect();
        if items.is_empty() {
            continue;
        }
        if let Some(label) = group {
            out.push_str(&format!("#### {label}\n\n"));
        }
        for item in items {
            if let Some(hex) = resolve_doc_color(item.color.as_deref()) {
                out.push_str(&format!("- **[{hex}] {}**", item.title));
            } else {
                out.push_str(&format!("- **{}**", item.title));
            }
            if let Some(desc) = item.description.as_deref().filter(|s| !s.trim().is_empty()) {
                out.push_str(&format!(" — {desc}"));
            }
            out.push('\n');
            if let Some(notes) = item.notes.as_deref().filter(|s| !s.trim().is_empty()) {
                out.push_str(&format!("  - Notes: {notes}\n"));
            }
        }
        out.push('\n');
    }
    out
}

pub fn render_vendor_doc_html(
    vendor_name: &str,
    vendor_id: &str,
    section: &VendorDocSection,
    escape_html: fn(&str) -> String,
    interactive: bool,
    report_locale: ReportLocale,
) -> String {
    if section.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(&format!(
        "  <article class=\"vendor-doc-report-card\" data-vendor-docs=\"{}\">\n",
        escape_html(vendor_id)
    ));
    out.push_str(&format!(
        "    <h3>{} — {}</h3>\n",
        escape_html(vendor_name),
        escape_html(&section.name)
    ));
    if let Some(overview) = section.overview.as_deref().filter(|s| !s.trim().is_empty()) {
        out.push_str(&format!("    <p class=\"muted\">{}</p>\n", escape_html(overview)));
    }
    if interactive {
        out.push_str(&render_doc_color_filter(&section.items, escape_html, report_locale));
    }
    for group in groups_in_order(&section.items) {
        let items: Vec<_> = section
            .items
            .iter()
            .filter(|i| item_group(i) == group)
            .collect();
        if items.is_empty() {
            continue;
        }
        if let Some(label) = group {
            out.push_str(&format!(
                "    <h4>{}</h4>\n    <ul class=\"vendor-doc-item-list\">\n",
                escape_html(label)
            ));
        } else {
            out.push_str("    <ul class=\"vendor-doc-item-list\">\n");
        }
        for item in items {
            let hex = resolve_doc_color(item.color.as_deref());
            let style = hex
                .as_ref()
                .map(|h| format!(" style=\"border-left: 4px solid {h}\""))
                .unwrap_or_default();
            let color_attr = hex
                .map(|h| format!(" data-doc-color=\"{h}\""))
                .unwrap_or_else(|| " data-doc-color=\"neutral\"".into());
            out.push_str(&format!(
                "      <li class=\"vendor-doc-item\"{color_attr}{style}>\n"
            ));
            out.push_str(&format!(
                "        <strong>{}</strong>",
                escape_html(&item.title)
            ));
            if let Some(desc) = item.description.as_deref().filter(|s| !s.trim().is_empty()) {
                out.push_str(&format!("<br><span class=\"muted\">{}</span>", escape_html(desc)));
            }
            if let Some(notes) = item.notes.as_deref().filter(|s| !s.trim().is_empty()) {
                out.push_str(&format!(
                    "<br><span class=\"vendor-doc-notes\">Notes: {}</span>",
                    escape_html(notes)
                ));
            }
            out.push_str("\n      </li>\n");
        }
        out.push_str("    </ul>\n");
    }
    out.push_str("  </article>\n");
    out
}

fn item_group(item: &VendorDocItem) -> Option<&str> {
    item.group
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn groups_in_order(items: &[VendorDocItem]) -> Vec<Option<&str>> {
    let mut seen = HashSet::new();
    let mut groups = Vec::new();
    for item in items {
        let group = item_group(item);
        if seen.insert(group) {
            groups.push(group);
        }
    }
    groups
}

pub fn render_all_vendor_docs_markdown(
    vendor_name: &str,
    sections: &[VendorDocSection],
) -> String {
    sections
        .iter()
        .filter(|s| !s.is_empty())
        .map(|section| render_vendor_doc_markdown(vendor_name, section))
        .collect()
}

pub fn render_all_vendor_docs_html(
    vendor_name: &str,
    vendor_id: &str,
    sections: &[VendorDocSection],
    escape_html: fn(&str) -> String,
    interactive: bool,
    report_locale: ReportLocale,
) -> String {
    sections
        .iter()
        .filter(|s| !s.is_empty())
        .map(|section| {
            render_vendor_doc_html(
                vendor_name,
                vendor_id,
                section,
                escape_html,
                interactive,
                report_locale,
            )
        })
        .collect()
}

fn render_doc_color_filter(
    items: &[VendorDocItem],
    escape_html: fn(&str) -> String,
    report_locale: ReportLocale,
) -> String {
    let s = locale::strings(report_locale);
    let mut colors: Vec<String> = Vec::new();
    for item in items {
        let c = resolve_doc_color(item.color.as_deref()).unwrap_or_else(|| "neutral".into());
        if !colors.contains(&c) {
            colors.push(c);
        }
    }
    if colors.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "    <div class=\"mad-doc-filter\" role=\"group\" aria-label=\"Filter by highlight color\">\n",
    );
    out.push_str(&format!(
        "      <button type=\"button\" data-doc-color=\"all\" class=\"active\">{}</button>\n",
        escape_html(s.doc_filter_all)
    ));
    for color in colors {
        let swatch = if color == "neutral" {
            String::new()
        } else {
            format!(
                "<span class=\"mad-color-swatch\" style=\"background:{}\"></span>",
                escape_html(&color)
            )
        };
        out.push_str(&format!(
            "      <button type=\"button\" data-doc-color=\"{}\">{swatch}{}</button>\n",
            escape_html(&color),
            escape_html(&color_label(&color, report_locale)),
        ));
    }
    out.push_str("    </div>\n");
    out
}

fn color_label(hex: &str, report_locale: ReportLocale) -> String {
    match (report_locale, hex) {
        (ReportLocale::Fr, "#ef4444" | "#dc3545") => "Critique".into(),
        (ReportLocale::Fr, "#f59e0b" | "#e6a800") => "Attention".into(),
        (ReportLocale::Fr, "#3b82f6" | "#1e88e5" | "#00b4d8") => "Info".into(),
        (ReportLocale::Fr, "#22c55e" | "#28a745") => "OK".into(),
        (ReportLocale::Fr, "neutral") => "Défaut".into(),
        (_, "#ef4444" | "#dc3545") => "Critical".into(),
        (_, "#f59e0b" | "#e6a800") => "Warning".into(),
        (_, "#3b82f6" | "#1e88e5" | "#00b4d8") => "Info".into(),
        (_, "#22c55e" | "#28a745") => "Success".into(),
        (_, "neutral") => "Default".into(),
        (_, other) => other.to_string(),
    }
}

pub fn groups_for_pdf(items: &[VendorDocItem]) -> Vec<Option<String>> {
    let mut seen = HashSet::new();
    let mut groups = Vec::new();
    for item in items {
        let group = item
            .group
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        if seen.insert(group.clone()) {
            groups.push(group);
        }
    }
    groups
}
