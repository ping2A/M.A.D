use std::collections::HashSet;

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
    section: &VendorDocSection,
    escape_html: fn(&str) -> String,
) -> String {
    if section.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("  <article class=\"vendor-doc-report-card\">\n");
    out.push_str(&format!(
        "    <h3>{} — {}</h3>\n",
        escape_html(vendor_name),
        escape_html(&section.name)
    ));
    if let Some(overview) = section.overview.as_deref().filter(|s| !s.trim().is_empty()) {
        out.push_str(&format!("    <p class=\"muted\">{}</p>\n", escape_html(overview)));
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
            let style = resolve_doc_color(item.color.as_deref())
                .map(|hex| format!(" style=\"border-left: 4px solid {hex}\""))
                .unwrap_or_default();
            out.push_str(&format!("      <li class=\"vendor-doc-item\"{style}>\n"));
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
    sections: &[VendorDocSection],
    escape_html: fn(&str) -> String,
) -> String {
    sections
        .iter()
        .filter(|s| !s.is_empty())
        .map(|section| render_vendor_doc_html(vendor_name, section, escape_html))
        .collect()
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
