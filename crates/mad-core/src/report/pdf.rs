use std::io::{BufWriter, Cursor};

use image::io::Reader as ImageReader;
use printpdf::image::Image;
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{
    BuiltinFont, Color, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerIndex, PdfLayerReference,
    PdfPageIndex, Point, Polygon, Rgb,
};

use std::collections::HashMap;

use crate::evaluation::{EvaluationReport, EvaluationResult};
use crate::pillar::RequirementSeverity;
use crate::policy::PolicyBundle;
use crate::report::{vendor_doc, vsm};
use crate::vendor_doc::VendorDocSection;
use crate::value_stream::{ValueStreamEntry, ValueStreamMap};
use crate::vendor::ComplianceStatus;

/// Options for PDF report generation.
#[derive(Debug, Clone, Default)]
pub struct PdfReportOptions {
    pub generated_at: Option<String>,
    pub logo_png: Option<Vec<u8>>,
}

const PAGE_W_MM: f32 = 210.0;
const PAGE_H_MM: f32 = 297.0;
const MARGIN_MM: f32 = 14.0;
const FOOTER_H_MM: f32 = 10.0;
const CONTENT_W_MM: f32 = PAGE_W_MM - 2.0 * MARGIN_MM;
const CONTENT_BOTTOM_MM: f32 = PAGE_H_MM - MARGIN_MM - FOOTER_H_MM;

mod colors {
    use super::Rgb;

    pub fn navy() -> Rgb {
        Rgb::new(0.039, 0.086, 0.157, None)
    }
    pub fn navy_light() -> Rgb {
        Rgb::new(0.075, 0.133, 0.220, None)
    }
    pub fn cyan() -> Rgb {
        Rgb::new(0.0, 0.706, 0.847, None)
    }
    pub fn white() -> Rgb {
        Rgb::new(1.0, 1.0, 1.0, None)
    }
    pub fn text() -> Rgb {
        Rgb::new(0.102, 0.137, 0.196, None)
    }
    pub fn muted() -> Rgb {
        Rgb::new(0.353, 0.396, 0.471, None)
    }
    pub fn silver() -> Rgb {
        Rgb::new(0.753, 0.784, 0.831, None)
    }
    pub fn compliant() -> Rgb {
        Rgb::new(0.157, 0.655, 0.271, None)
    }
    pub fn partial() -> Rgb {
        Rgb::new(0.902, 0.659, 0.0, None)
    }
    pub fn gap() -> Rgb {
        Rgb::new(0.863, 0.208, 0.271, None)
    }
    pub fn critical_bg() -> Rgb {
        Rgb::new(0.992, 0.910, 0.918, None)
    }
    pub fn high_bg() -> Rgb {
        Rgb::new(1.0, 0.953, 0.878, None)
    }
    pub fn medium_bg() -> Rgb {
        Rgb::new(1.0, 0.973, 0.882, None)
    }
    pub fn scope_in_bg() -> Rgb {
        Rgb::new(0.910, 0.961, 0.914, None)
    }
    pub fn scope_out_bg() -> Rgb {
        Rgb::new(0.992, 0.910, 0.918, None)
    }
    pub fn card_bg() -> Rgb {
        Rgb::new(0.973, 0.976, 0.980, None)
    }
    pub fn bar_bg() -> Rgb {
        Rgb::new(0.910, 0.918, 0.929, None)
    }
}

struct PdfLayout {
    doc: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    y_top: f32,
    page_num: u32,
    generated_label: String,
    font: printpdf::IndirectFontRef,
    font_bold: printpdf::IndirectFontRef,
}

impl PdfLayout {
    fn new(
        doc: PdfDocumentReference,
        page: PdfPageIndex,
        layer: PdfLayerIndex,
        generated_label: String,
    ) -> Self {
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).expect("font");
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .expect("bold font");
        Self {
            doc,
            page,
            layer,
            y_top: MARGIN_MM,
            page_num: 1,
            generated_label,
            font,
            font_bold,
        }
    }

    fn layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.page).get_layer(self.layer)
    }

    fn ensure(&mut self, height_mm: f32) {
        if self.y_top + height_mm > CONTENT_BOTTOM_MM {
            self.draw_page_footer();
            let (page, layer) = self.doc.add_page(Mm(PAGE_W_MM), Mm(PAGE_H_MM), "Page");
            self.page = page;
            self.layer = layer;
            self.page_num += 1;
            self.y_top = MARGIN_MM;
        }
    }

    fn draw_page_footer(&mut self) {
        let y = PAGE_H_MM - 7.0;
        let generated = self.generated_label.clone();
        let page = self.page_num;
        self.stroke_line(MARGIN_MM, y - 3.0, MARGIN_MM + CONTENT_W_MM, colors::bar_bg(), 0.3);
        self.text_at(
            MARGIN_MM,
            y,
            "MAD - Mobile Assessment & Defense",
            7.0,
            false,
            colors::muted(),
        );
        self.text_at(
            MARGIN_MM + 70.0,
            y,
            &generated,
            7.0,
            false,
            colors::muted(),
        );
        self.text_at(
            PAGE_W_MM - MARGIN_MM - 18.0,
            y,
            &format!("Page {page}"),
            7.0,
            false,
            colors::muted(),
        );
    }

    fn advance(&mut self, height_mm: f32) {
        self.y_top += height_mm;
    }

    fn fill_rect(&mut self, x: f32, y_top: f32, w: f32, h: f32, fill: Rgb) {
        let layer = self.layer();
        layer.set_fill_color(Color::Rgb(fill.clone()));
        layer.set_outline_color(Color::Rgb(fill));
        layer.add_polygon(Polygon {
            rings: vec![rect_points(x, y_top, w, h)],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        });
    }

    fn stroke_line(&mut self, x1: f32, y_top: f32, x2: f32, color: Rgb, thickness: f32) {
        self.stroke_line_xy(x1, y_top, x2, y_top, color, thickness);
    }

    fn stroke_line_xy(
        &mut self,
        x1: f32,
        y1_top: f32,
        x2: f32,
        y2_top: f32,
        color: Rgb,
        thickness: f32,
    ) {
        let layer = self.layer();
        layer.set_outline_color(Color::Rgb(color));
        layer.set_outline_thickness(thickness);
        let line = Line {
            points: vec![
                (Point::new(Mm(x1), Mm(PAGE_H_MM - y1_top)), false),
                (Point::new(Mm(x2), Mm(PAGE_H_MM - y2_top)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    fn stroke_rect(&mut self, x: f32, y_top: f32, w: f32, h: f32, color: Rgb, thickness: f32) {
        self.stroke_line_xy(x, y_top, x + w, y_top, color.clone(), thickness);
        self.stroke_line_xy(x + w, y_top, x + w, y_top + h, color.clone(), thickness);
        self.stroke_line_xy(x + w, y_top + h, x, y_top + h, color.clone(), thickness);
        self.stroke_line_xy(x, y_top + h, x, y_top, color, thickness);
    }

    fn text_at(
        &mut self,
        x: f32,
        y_top: f32,
        content: &str,
        size: f32,
        bold: bool,
        color: Rgb,
    ) {
        let layer = self.layer();
        layer.set_fill_color(Color::Rgb(color));
        let font = if bold { &self.font_bold } else { &self.font };
        layer.use_text(
            &sanitize_pdf_text(content),
            size,
            Mm(x),
            Mm(PAGE_H_MM - y_top),
            font,
        );
    }

    fn text_block(
        &mut self,
        content: &str,
        size: f32,
        bold: bool,
        color: Rgb,
        max_chars: usize,
        line_h: f32,
    ) {
        for line in wrap_text(content, max_chars) {
            self.ensure(line_h);
            self.text_at(MARGIN_MM, self.y_top, &line, size, bold, color.clone());
            self.advance(line_h);
        }
    }

    fn section_heading(&mut self, title: &str) {
        self.advance(4.0);
        self.ensure(10.0);
        self.text_at(MARGIN_MM, self.y_top, title, 13.0, true, colors::navy());
        self.advance(5.5);
        self.stroke_line(MARGIN_MM, self.y_top, MARGIN_MM + CONTENT_W_MM, colors::cyan(), 0.8);
        self.advance(3.0);
    }

    fn subheading(&mut self, title: &str) {
        self.ensure(6.0);
        self.text_at(MARGIN_MM, self.y_top, title, 10.5, true, colors::navy());
        self.advance(5.0);
    }

    fn paragraph(&mut self, content: &str) {
        self.text_block(content, 9.0, false, colors::text(), 98, 4.2);
        self.advance(1.5);
    }

    fn draw_logo(&mut self, png: &[u8], x: f32, y_top: f32, height_mm: f32) -> Option<f32> {
        let reader = ImageReader::new(Cursor::new(png)).with_guessed_format().ok()?;
        let img = reader.decode().ok()?;
        let w_px = img.width() as f32;
        let h_px = img.height() as f32;
        if h_px == 0.0 {
            return None;
        }
        let width_mm = height_mm * (w_px / h_px);
        let dpi = 300.0;
        let img_w_mm = w_px / dpi * 25.4;
        let scale = height_mm / (h_px / dpi * 25.4);
        let pdf_image = Image::from_dynamic_image(&img);
        pdf_image.add_to_layer(
            self.layer(),
            printpdf::image::ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(PAGE_H_MM - y_top - height_mm)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(dpi),
                rotate: None,
            },
        );
        let _ = img_w_mm;
        Some(width_mm)
    }

    fn severity_badge(&mut self, x: f32, y_top: f32, severity: RequirementSeverity) -> f32 {
        let (label, bg, fg) = match severity {
            RequirementSeverity::Critical => ("CRITICAL", colors::critical_bg(), colors::gap()),
            RequirementSeverity::High => ("HIGH", colors::high_bg(), colors::partial()),
            RequirementSeverity::Medium => ("MEDIUM", colors::medium_bg(), colors::muted()),
        };
        let w = 22.0;
        let h = 5.0;
        self.fill_rect(x, y_top, w, h, bg);
        self.text_at(x + 1.5, y_top + 3.8, label, 6.5, true, fg);
        w
    }

    fn status_badge(&mut self, x: f32, y_top: f32, status: ComplianceStatus) -> f32 {
        let (label, bg, fg) = match status {
            ComplianceStatus::Compliant => ("OK", colors::scope_in_bg(), colors::compliant()),
            ComplianceStatus::Partial => ("PARTIAL", colors::high_bg(), colors::partial()),
            ComplianceStatus::NonCompliant => ("GAP", colors::scope_out_bg(), colors::gap()),
            ComplianceStatus::Untested => ("N/A", colors::bar_bg(), colors::muted()),
        };
        let w = match status {
            ComplianceStatus::Partial => 18.0,
            ComplianceStatus::NonCompliant => 12.0,
            _ => 10.0,
        };
        let h = 4.5;
        self.fill_rect(x, y_top, w, h, bg);
        self.text_at(x + 1.2, y_top + 3.4, label, 6.0, true, fg);
        w
    }

    fn score_bar(&mut self, x: f32, y_top: f32, w: f32, pct: f64) {
        let h = 3.0;
        self.fill_rect(x, y_top, w, h, colors::bar_bg());
        let fill = w * (pct.clamp(0.0, 100.0) / 100.0) as f32;
        if fill > 0.1 {
            self.fill_rect(x, y_top, fill, h, score_rgb(pct));
        }
    }
}

/// Renders a styled multi-page PDF evaluation report.
pub fn render_pdf(
    bundle: &PolicyBundle,
    evaluation: &EvaluationReport,
    value_streams: &HashMap<String, Vec<ValueStreamEntry>>,
    vendor_docs: &HashMap<String, Vec<VendorDocSection>>,
    options: &PdfReportOptions,
) -> Result<Vec<u8>, String> {
    let generated = options.generated_at.as_deref().unwrap_or("-").to_string();
    let (doc, page1, layer1) =
        PdfDocument::new("MAD Report", Mm(PAGE_W_MM), Mm(PAGE_H_MM), "Layer 1");
    let mut layout = PdfLayout::new(doc, page1, layer1, generated.clone());

    // ── Header band ──────────────────────────────────────────────────────
    let header_h = 28.0;
    layout.fill_rect(0.0, 0.0, PAGE_W_MM, header_h, colors::navy());
    layout.fill_rect(0.0, header_h, PAGE_W_MM, 1.2, colors::cyan());

    let mut title_x = MARGIN_MM;
    if let Some(logo) = &options.logo_png {
        if let Some(w) = layout.draw_logo(logo, MARGIN_MM, 5.0, 18.0) {
            title_x = MARGIN_MM + w + 5.0;
        }
    }

    layout.text_at(title_x, 11.0, "MAD", 16.0, true, colors::white());
    layout.text_at(
        title_x,
        18.0,
        "Mobile Assessment & Defense",
        9.5,
        false,
        colors::silver(),
    );
    layout.text_at(
        PAGE_W_MM - MARGIN_MM - 42.0,
        10.0,
        "iOS & Android MDM",
        8.0,
        true,
        colors::cyan(),
    );

    layout.y_top = header_h + 6.0;

    // ── Meta bar ─────────────────────────────────────────────────────────
    let meta_h = 14.0;
    let meta_y = layout.y_top;
    layout.fill_rect(MARGIN_MM, meta_y, CONTENT_W_MM, meta_h, colors::navy_light());
    layout.text_at(
        MARGIN_MM + 3.0,
        meta_y + 5.0,
        &format!(
            "Policy v{}   Requirements: {} ({} critical)   Vendors: {}",
            evaluation.policy_version,
            evaluation.total_requirements,
            evaluation.critical_requirements,
            evaluation.vendors.len()
        ),
        8.0,
        false,
        colors::white(),
    );
    let mut meta_line2 = format!("Generated {generated}");
    if evaluation.procurement.device_count > 0 {
        meta_line2.push_str(&format!(
            "   Devices: {}",
            evaluation.procurement.device_count
        ));
        if evaluation.procurement.use_price_in_ranking {
            meta_line2.push_str(&format!(
                "   Price weight: {:.0}%",
                evaluation.procurement.price_weight_percent
            ));
        }
    }
    layout.text_at(
        MARGIN_MM + 3.0,
        meta_y + 10.0,
        &meta_line2,
        7.0,
        false,
        colors::silver(),
    );
    layout.advance(meta_h + 5.0);

    // ── Section 1 ────────────────────────────────────────────────────────
    layout.section_heading("1. Evaluation Methodology");
    layout.paragraph(
        "Requirements are defined in Policy-as-Code YAML. Each requirement maps to a compliance \
         status with scoring weights for pillar and overall scores.",
    );

    let table_y = layout.y_top;
    let row_h = 6.0;
    layout.fill_rect(MARGIN_MM, table_y, CONTENT_W_MM, row_h, colors::navy_light());
    layout.text_at(MARGIN_MM + 2.0, table_y + 4.5, "Status", 8.0, true, colors::white());
    layout.text_at(MARGIN_MM + 38.0, table_y + 4.5, "Weight", 8.0, true, colors::white());
    layout.text_at(MARGIN_MM + 55.0, table_y + 4.5, "Meaning", 8.0, true, colors::white());
    layout.advance(row_h);

    for (i, (status, weight, meaning)) in [
        ("compliant", "1.0", "Native capability, no workarounds"),
        ("partial", "0.5", "Limited, platform-specific, or manual"),
        ("non_compliant", "0.0", "Cannot be met"),
        ("untested", "0.0", "No assessment data"),
    ]
    .iter()
    .enumerate()
    {
        let row_y = layout.y_top;
        if i % 2 == 0 {
            layout.fill_rect(MARGIN_MM, row_y, CONTENT_W_MM, row_h, colors::card_bg());
        }
        let st = match *status {
            "compliant" => ComplianceStatus::Compliant,
            "partial" => ComplianceStatus::Partial,
            "non_compliant" => ComplianceStatus::NonCompliant,
            _ => ComplianceStatus::Untested,
        };
        layout.status_badge(MARGIN_MM + 2.0, row_y + 0.8, st);
        layout.text_at(MARGIN_MM + 38.0, row_y + 4.5, weight, 8.5, false, colors::text());
        layout.text_at(MARGIN_MM + 55.0, row_y + 4.5, meaning, 8.5, false, colors::text());
        layout.advance(row_h);
    }
    layout.advance(3.0);

    let code_y = layout.y_top;
    let code_h = 16.0;
    layout.fill_rect(MARGIN_MM, code_y, CONTENT_W_MM, code_h, colors::navy());
    let formula = "pillar_score = ((compliant x 1.0) + (partial x 0.5)) / total x 100";
    layout.text_at(MARGIN_MM + 3.0, code_y + 6.0, formula, 8.0, false, colors::cyan());
    let pillar_names: Vec<String> = bundle.pillars.iter().map(|p| p.name.clone()).collect();
    let overall_formula = if pillar_names.is_empty() {
        "overall_score = mean(pillar scores)".to_string()
    } else if pillar_names.len() <= 3 {
        format!("overall_score = mean({})", pillar_names.join(", "))
    } else {
        format!(
            "overall_score = mean({} + {} more)",
            pillar_names[..2].join(", "),
            pillar_names.len() - 2
        )
    };
    layout.text_at(
        MARGIN_MM + 3.0,
        code_y + 11.0,
        &overall_formula,
        8.0,
        false,
        colors::silver(),
    );
    layout.advance(code_h + 5.0);

    let sc = &evaluation.scoring;
    layout.subheading("Active scoring model");
    if sc.use_severity_weighting {
        layout.paragraph(&format!(
            "Severity weighting enabled — critical x{:.1}, high x{:.1}, medium x{:.1}. \
             Critical requirements weigh more in the overall score.",
            sc.critical_weight, sc.high_weight, sc.medium_weight
        ));
    } else {
        layout.paragraph("Severity weighting disabled — each requirement counts equally.");
    }
    layout.paragraph(&format!(
        "Status points: compliant {:.1}, partial {:.1}, non-compliant {:.1}, untested {:.1}.",
        sc.compliant_points,
        sc.partial_points,
        sc.non_compliant_points,
        sc.untested_points
    ));
    layout.advance(2.0);

    // ── Section 2 ────────────────────────────────────────────────────────
    layout.section_heading("2. Requirements and Technical Criteria");
    for pillar in &bundle.pillars {
        layout.subheading(&pillar.name);
        layout.paragraph(pillar.description.trim());
        for req in &pillar.requirements {
            let card_y = layout.y_top;
            layout.ensure(20.0);
            let card_h = estimate_req_height(req);
            layout.fill_rect(MARGIN_MM, card_y, CONTENT_W_MM, card_h, colors::card_bg());
            layout.fill_rect(MARGIN_MM, card_y, 1.2, card_h, colors::cyan());

            let badge_x = MARGIN_MM + 4.0;
            layout.severity_badge(badge_x, card_y + 2.0, req.severity);
            layout.text_at(
                badge_x + 24.0,
                card_y + 5.5,
                &format!("{} — {}", req.id, req.title),
                9.5,
                true,
                colors::navy(),
            );
            let mut inner_y = card_y + 9.0;
            for line in wrap_text(req.description.trim(), 95) {
                layout.text_at(MARGIN_MM + 4.0, inner_y, &line, 8.5, false, colors::text());
                inner_y += 4.0;
            }
            layout.text_at(
                MARGIN_MM + 4.0,
                inner_y,
                &format!("Platforms: {}", req.platforms.join(", ")),
                7.5,
                false,
                colors::muted(),
            );
            inner_y += 4.5;
            if let Some(m) = &req.evaluation_method {
                for line in wrap_text(&format!("Test: {}", m.trim()), 92) {
                    layout.text_at(MARGIN_MM + 4.0, inner_y, &line, 7.5, false, colors::muted());
                    inner_y += 3.8;
                }
            }
            if let Some(tc) = &req.technical_criteria {
                for line in wrap_text(&format!("Criteria: {}", tc.trim()), 92) {
                    layout.text_at(MARGIN_MM + 4.0, inner_y, &line, 7.5, false, colors::muted());
                    inner_y += 3.8;
                }
            }
            layout.advance(card_h + 2.5);
        }
    }

    // ── Section 4 ────────────────────────────────────────────────────────
    layout.section_heading("3. Vendor Assessment Results");

    let ranked = rank_vendors(evaluation);
    let composite_ranking = uses_composite_ranking(evaluation);

    if evaluation.procurement.device_count > 0 {
        let price_note = if composite_ranking {
            format!(
                "Ranking uses composite score: {:.0}% capability + {:.0}% price (lower annual cost per device scores higher).",
                100.0 - evaluation.procurement.price_weight_percent,
                evaluation.procurement.price_weight_percent
            )
        } else {
            "Annual costs are estimated from vendor pricing for the configured device count (reference only).".into()
        };
        layout.paragraph(&price_note);
    }

    layout.subheading("Comparison summary");
    render_comparison_table(&mut layout, evaluation, &ranked, composite_ranking);
    layout.advance(4.0);

    let pillar_defs: Vec<(String, String)> = ranked
        .first()
        .map(|r| {
            r.pillars
                .iter()
                .map(|p| (p.pillar_id.clone(), p.pillar_name.clone()))
                .collect()
        })
        .unwrap_or_else(|| {
            bundle
                .pillars
                .iter()
                .map(|p| (p.id.as_str().to_string(), p.name.clone()))
                .collect()
        });

    layout.subheading("Pillar leaders");
    for (pid, name) in &pillar_defs {
        let mut best: Vec<&EvaluationResult> = Vec::new();
        let mut best_score = -1.0_f64;
        for result in &ranked {
            let pct = pillar_score_percent(result, pid);
            if (pct - best_score).abs() < f64::EPSILON {
                best.push(result);
            } else if pct > best_score {
                best_score = pct;
                best.clear();
                best.push(result);
            }
        }
        let leaders = best
            .iter()
            .map(|r| r.vendor.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        layout.paragraph(&format!(
            "{} — {:.0}% ({})",
            name,
            best_score.max(0.0),
            if leaders.is_empty() { "—" } else { &leaders }
        ));
    }
    layout.advance(2.0);

    for (rank, result) in ranked.iter().enumerate() {
        let capability = result.overall_score.overall_score_percent;
        let display_score = if composite_ranking {
            result
                .overall_score
                .composite_score_percent
                .unwrap_or(capability)
        } else {
            capability
        };
        let card_h = estimate_vendor_card_height(result);
        layout.ensure(card_h);
        let card_y = layout.y_top;

        layout.fill_rect(MARGIN_MM, card_y, CONTENT_W_MM, card_h, colors::card_bg());
        layout.fill_rect(MARGIN_MM, card_y, 2.5, card_h, colors::cyan());

        layout.text_at(
            MARGIN_MM + 6.0,
            card_y + 6.0,
            &format!("#{}", rank + 1),
            14.0,
            true,
            colors::muted(),
        );
        layout.text_at(
            MARGIN_MM + 18.0,
            card_y + 6.0,
            &result.vendor.name,
            12.0,
            true,
            colors::navy(),
        );
        let score_label = if composite_ranking {
            format!("{display_score:.1}%*")
        } else {
            format!("{display_score:.1}%")
        };
        layout.text_at(
            MARGIN_MM + CONTENT_W_MM - 32.0,
            card_y + 6.0,
            &score_label,
            13.0,
            true,
            score_rgb(display_score),
        );

        let (ok, part, gap, untested) = vendor_status_totals(result);
        let mut summary = format!("OK {ok} - Partial {part} - Gap {gap} - Untested {untested}");
        if let Some(cost) = result.overall_score.annual_cost_per_device {
            let cur = result
                .overall_score
                .price_currency
                .as_deref()
                .unwrap_or("USD");
            summary.push_str(&format!("   {cur} {cost:.0}/device/yr"));
            if let Some(total) = result.overall_score.total_annual_cost {
                summary.push_str(&format!(" ({cur} {total:.0} total)"));
            }
        }
        if composite_ranking {
            summary.push_str(&format!("   Capability {capability:.1}%"));
            if let Some(ps) = result.overall_score.price_score_percent {
                summary.push_str(&format!("   Price score {ps:.0}%"));
            }
        }
        layout.text_at(MARGIN_MM + 6.0, card_y + 12.0, &summary, 7.0, false, colors::muted());

        let mut inner_y = card_y + 16.5;
        for line in wrap_text(&result.vendor.description, 90) {
            layout.text_at(MARGIN_MM + 6.0, inner_y, &line, 8.0, false, colors::muted());
            inner_y += 4.0;
        }
        if let Some(site) = &result.vendor.website {
            if !site.is_empty() {
                layout.text_at(
                    MARGIN_MM + 6.0,
                    inner_y,
                    &format!("Website: {site}"),
                    7.5,
                    false,
                    colors::cyan(),
                );
                inner_y += 4.5;
            }
        }
        inner_y += 2.0;

        for pillar in &result.pillars {
            let pct = pillar.score.score_percent;
            layout.text_at(
                MARGIN_MM + 6.0,
                inner_y,
                &format!("{} — {pct:.0}%", pillar.pillar_name),
                8.5,
                true,
                colors::navy(),
            );
            layout.score_bar(MARGIN_MM + 90.0, inner_y - 1.0, 80.0, pct);
            inner_y += 6.0;

            for req in &pillar.requirements {
                if req.applicable {
                    layout.status_badge(MARGIN_MM + 8.0, inner_y, req.status);
                    let sev_w = layout.severity_badge(MARGIN_MM + 20.0, inner_y, req.severity);
                    layout.text_at(
                        MARGIN_MM + 20.0 + sev_w + 2.0,
                        inner_y + 3.5,
                        &format!(
                            "{} {}",
                            req.requirement_id,
                            truncate_chars(&req.title, 52)
                        ),
                        7.5,
                        false,
                        colors::text(),
                    );
                } else {
                    layout.text_at(MARGIN_MM + 8.0, inner_y + 3.5, "N/A", 7.5, true, colors::muted());
                    layout.text_at(
                        MARGIN_MM + 20.0,
                        inner_y + 3.5,
                        &format!(
                            "{} {}",
                            req.requirement_id,
                            truncate_chars(&req.title, 52)
                        ),
                        7.5,
                        false,
                        colors::muted(),
                    );
                }
                inner_y += 5.0;
                if req.applicable {
                    if let Some(notes) = &req.notes {
                        if !notes.trim().is_empty() {
                            for line in wrap_text(&format!("Note: {}", notes.trim()), 88) {
                                layout.text_at(MARGIN_MM + 22.0, inner_y, &line, 7.0, false, colors::muted());
                                inner_y += 3.8;
                            }
                        }
                    }
                }
            }
            inner_y += 1.5;
        }

        if !result.overall_score.critical_gaps.is_empty() {
            let gap_y = inner_y;
            let gap_h = estimate_critical_gaps_height(&result.overall_score.critical_gaps);
            layout.fill_rect(MARGIN_MM + 4.0, gap_y, CONTENT_W_MM - 8.0, gap_h, colors::scope_out_bg());
            layout.text_at(
                MARGIN_MM + 7.0,
                gap_y + 5.0,
                "Critical gaps",
                8.5,
                true,
                colors::gap(),
            );
            let mut gap_y_text = gap_y + 9.0;
            for gap in &result.overall_score.critical_gaps {
                for line in wrap_text(&format!("- {gap}"), 88) {
                    layout.text_at(MARGIN_MM + 7.0, gap_y_text, &line, 7.5, false, colors::gap());
                    gap_y_text += 3.8;
                }
            }
        }

        layout.advance(card_h + 5.0);
    }

    if composite_ranking {
        layout.paragraph("* Composite score used for ranking (capability + price).");
    }

    let mut section = 4u8;
    if vsm::any_value_streams(value_streams) {
        layout.section_heading(&format!("{section}. Value Stream Maps"));
        layout.paragraph(
            "Process flows documented per vendor: steps, flow types, durations, and authors.",
        );
        layout.advance(2.0);

        for result in &evaluation.vendors {
            let Some(entries) = value_streams.get(&result.vendor.id.0) else {
                continue;
            };
            for entry in entries {
                if !vsm::map_has_content(&entry.map) {
                    continue;
                }
                let title = format!("{} — {}", result.vendor.name, entry.name);
                append_vsm_vendor_pdf(&mut layout, &title, &entry.map);
            }
        }
        section += 1;
    }

    if vendor_doc::any_vendor_docs(vendor_docs) {
        layout.section_heading(&format!("{section}. Vendor Documentation"));
        layout.paragraph(
            "User-defined per-vendor documentation (e.g. privacy, support). \
             Informational only — not included in capability scores.",
        );
        layout.advance(2.0);

        for result in &evaluation.vendors {
            let Some(sections) = vendor_docs.get(&result.vendor.id.0) else {
                continue;
            };
            for section in sections {
                if section.is_empty() {
                    continue;
                }
                layout.subheading(&format!("{} — {}", result.vendor.name, section.name));
                if let Some(overview) = section.overview.as_deref().filter(|s| !s.trim().is_empty())
                {
                    layout.paragraph(overview);
                }
                for group in vendor_doc::groups_for_pdf(&section.items) {
                    let items: Vec<_> = section
                        .items
                        .iter()
                        .filter(|i| {
                            let g = i
                                .group
                                .as_deref()
                                .map(str::trim)
                                .filter(|s| !s.is_empty());
                            g.map(|s| s.to_string()) == group
                        })
                        .collect();
                    if items.is_empty() {
                        continue;
                    }
                    if let Some(ref label) = group {
                        layout.text_block(label, 9.0, true, colors::navy(), 100, 4.5);
                        layout.advance(1.0);
                    }
                    for item in items {
                        let mut line = item.title.clone();
                        if let Some(desc) =
                            item.description.as_deref().filter(|s| !s.trim().is_empty())
                        {
                            line.push_str(&format!(" — {desc}"));
                        }
                        layout.paragraph(&line);
                        if let Some(notes) = item.notes.as_deref().filter(|s| !s.trim().is_empty())
                        {
                            layout.text_block(
                                &format!("Notes: {notes}"),
                                8.0,
                                false,
                                colors::muted(),
                                100,
                                4.0,
                            );
                        }
                    }
                    layout.advance(2.0);
                }
            }
        }
    }

    layout.advance(4.0);
    layout.text_block(
        "Scores reflect workspace assessments and the active scoring/procurement model at export time.",
        7.5,
        false,
        colors::muted(),
        100,
        3.8,
    );
    layout.draw_page_footer();

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    layout
        .doc
        .save(&mut buffer)
        .map_err(|e| format!("failed to write PDF: {e}"))?;
    Ok(buffer.into_inner().map_err(|e| e.to_string())?.into_inner())
}

fn uses_composite_ranking(evaluation: &EvaluationReport) -> bool {
    evaluation.procurement.use_price_in_ranking
        && evaluation
            .vendors
            .iter()
            .any(|v| v.overall_score.composite_score_percent.is_some())
}

fn effective_rank_score(result: &EvaluationResult, evaluation: &EvaluationReport) -> f64 {
    if evaluation.procurement.use_price_in_ranking {
        if let Some(c) = result.overall_score.composite_score_percent {
            return c;
        }
    }
    result.overall_score.overall_score_percent
}

fn rank_vendors<'a>(evaluation: &'a EvaluationReport) -> Vec<&'a EvaluationResult> {
    let mut ranked: Vec<_> = evaluation.vendors.iter().collect();
    ranked.sort_by(|a, b| {
        effective_rank_score(b, evaluation)
            .partial_cmp(&effective_rank_score(a, evaluation))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked
}

fn render_comparison_table(
    layout: &mut PdfLayout,
    evaluation: &EvaluationReport,
    ranked: &[&EvaluationResult],
    composite_ranking: bool,
) {
    let pillar_defs: Vec<(String, String)> = ranked
        .first()
        .map(|r| {
            r.pillars
                .iter()
                .map(|p| (p.pillar_id.clone(), p.pillar_name.clone()))
                .collect()
        })
        .unwrap_or_default();

    let show_price = evaluation.procurement.device_count > 0
        && ranked
            .iter()
            .any(|r| r.overall_score.annual_cost_per_device.is_some());

    let status_w = 34.0_f32;
    let price_w = if show_price { 36.0 } else { 0.0 };
    let fixed_w = 6.0 + 34.0 + 16.0 + price_w + status_w;
    let pillar_area = (CONTENT_W_MM - fixed_w).max(20.0);
    let pillar_count = pillar_defs.len().max(1) as f32;
    let pillar_col_w = pillar_area / pillar_count;

    let row_h = 7.5_f32;
    let hdr_y = layout.y_top;
    layout.fill_rect(MARGIN_MM, hdr_y, CONTENT_W_MM, row_h, colors::navy_light());
    let mut col = MARGIN_MM + 2.0;
    layout.text_at(col, hdr_y + 5.0, "#", 6.5, true, colors::white());
    col += 6.0;
    layout.text_at(col, hdr_y + 5.0, "Vendor", 6.5, true, colors::white());
    col += 34.0;
    let score_hdr = if composite_ranking { "Score*" } else { "Score" };
    layout.text_at(col, hdr_y + 5.0, score_hdr, 6.5, true, colors::white());
    col += 16.0;
    for (_, name) in &pillar_defs {
        layout.text_at(
            col,
            hdr_y + 5.0,
            &truncate_chars(name, 8),
            6.0,
            true,
            colors::white(),
        );
        col += pillar_col_w;
    }
    if show_price {
        layout.text_at(col, hdr_y + 5.0, "$/yr", 6.0, true, colors::white());
        col += price_w;
    }
    layout.text_at(col + 2.0, hdr_y + 5.0, "OK", 6.0, true, colors::white());
    layout.text_at(col + 10.0, hdr_y + 5.0, "Pt", 6.0, true, colors::white());
    layout.text_at(col + 18.0, hdr_y + 5.0, "Gp", 6.0, true, colors::white());
    layout.text_at(col + 26.0, hdr_y + 5.0, "Cr", 6.0, true, colors::white());
    layout.advance(row_h);

    for (i, result) in ranked.iter().enumerate() {
        layout.ensure(row_h);
        let row_y = layout.y_top;
        if i % 2 == 0 {
            layout.fill_rect(MARGIN_MM, row_y, CONTENT_W_MM, row_h, colors::card_bg());
        }
        let score = effective_rank_score(result, evaluation);
        let (ok, part, gap, _) = vendor_status_totals(result);
        let crit = result.overall_score.critical_gaps.len();

        let mut col = MARGIN_MM + 2.0;
        layout.text_at(col, row_y + 5.0, &format!("{}", i + 1), 6.5, true, colors::muted());
        col += 6.0;
        layout.text_at(
            col,
            row_y + 5.0,
            &truncate_chars(&result.vendor.name, 18),
            6.5,
            true,
            colors::navy(),
        );
        col += 34.0;
        layout.text_at(
            col,
            row_y + 5.0,
            &format!("{score:.0}%"),
            6.5,
            true,
            score_rgb(score),
        );
        col += 16.0;
        for (pid, _) in &pillar_defs {
            let pct = pillar_score_percent(result, pid);
            layout.text_at(
                col,
                row_y + 5.0,
                &format!("{pct:.0}%"),
                6.0,
                false,
                score_rgb(pct),
            );
            col += pillar_col_w;
        }
        if show_price {
            if let Some(cost) = result.overall_score.annual_cost_per_device {
                let cur = result
                    .overall_score
                    .price_currency
                    .as_deref()
                    .unwrap_or("USD");
                layout.text_at(
                    col,
                    row_y + 5.0,
                    &format!("{cur}{cost:.0}"),
                    6.0,
                    false,
                    colors::text(),
                );
            }
            col += price_w;
        }
        layout.text_at(col + 2.0, row_y + 5.0, &format!("{ok}"), 6.0, false, colors::compliant());
        layout.text_at(col + 10.0, row_y + 5.0, &format!("{part}"), 6.0, false, colors::partial());
        layout.text_at(col + 18.0, row_y + 5.0, &format!("{gap}"), 6.0, false, colors::gap());
        layout.text_at(
            col + 26.0,
            row_y + 5.0,
            &format!("{crit}"),
            6.0,
            true,
            if crit > 0 { colors::gap() } else { colors::muted() },
        );
        layout.advance(row_h);
    }
}

fn estimate_vendor_card_height(result: &EvaluationResult) -> f32 {
    let mut h = 24.0;
    h += wrap_text(&result.vendor.description, 90).len() as f32 * 4.0;
    if result.vendor.website.as_ref().is_some_and(|w| !w.is_empty()) {
        h += 4.5;
    }
    h += 2.0;
    for pillar in &result.pillars {
        h += 6.0;
        for req in &pillar.requirements {
            h += 5.0;
            if let Some(notes) = &req.notes {
                if !notes.trim().is_empty() {
                    h += wrap_text(&format!("Note: {}", notes.trim()), 88).len() as f32 * 3.8;
                }
            }
        }
        h += 1.5;
    }
    if !result.overall_score.critical_gaps.is_empty() {
        h += estimate_critical_gaps_height(&result.overall_score.critical_gaps);
    }
    h.max(28.0)
}

fn estimate_critical_gaps_height(gaps: &[String]) -> f32 {
    let mut h = 8.0;
    for gap in gaps {
        h += wrap_text(&format!("- {gap}"), 88).len() as f32 * 3.8;
    }
    h.max(10.0)
}

fn sanitize_pdf_text(s: &str) -> String {
    s.replace('—', "-")
        .replace('–', "-")
        .replace('→', "->")
        .replace('·', "-")
        .replace('•', "-")
        .replace('\u{2018}', "'")
        .replace('\u{2019}', "'")
        .replace('\u{201c}', "\"")
        .replace('\u{201d}', "\"")
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let trimmed = s.trim();
    let count = trimmed.chars().count();
    if count <= max_chars {
        return trimmed.to_string();
    }
    let cut: String = trimmed.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{cut}...")
}

fn rect_points(x: f32, y_top: f32, w: f32, h: f32) -> Vec<(Point, bool)> {
    let y_bottom = PAGE_H_MM - y_top - h;
    let y_pdf_top = PAGE_H_MM - y_top;
    vec![
        (Point::new(Mm(x), Mm(y_bottom)), false),
        (Point::new(Mm(x + w), Mm(y_bottom)), false),
        (Point::new(Mm(x + w), Mm(y_pdf_top)), false),
        (Point::new(Mm(x), Mm(y_pdf_top)), false),
    ]
}

fn append_vsm_vendor_pdf(layout: &mut PdfLayout, vendor_name: &str, map: &ValueStreamMap) {
    let flow_types = vsm::resolve_flow_types(map);
    let timeline = vsm::build_timeline(map);

    layout.subheading(vendor_name);
    layout.paragraph(&format!(
        "{} nodes, {} flows{}",
        map.nodes.len(),
        map.edges.len(),
        if timeline.stats.total_minutes > 0.0 {
            format!(
                ", {} total lead time, {}% of flows timed",
                vsm::format_duration(timeline.stats.total_minutes, false),
                timeline.stats.coverage_percent
            )
        } else {
            String::new()
        }
    ));

    if !map.nodes.is_empty() {
        draw_vsm_diagram_pdf(layout, map, &flow_types);
    }

    if !flow_types.is_empty() {
        let legend: String = flow_types
            .iter()
            .map(|ft| {
                let style = if ft.dashed { "dashed" } else { "solid" };
                format!("{} ({style})", ft.label)
            })
            .collect::<Vec<_>>()
            .join("  |  ");
        layout.paragraph(&format!("Flow types: {legend}"));
    }

    if !map.nodes.is_empty() {
        layout.text_at(MARGIN_MM, layout.y_top, "Process steps", 9.0, true, colors::navy());
        layout.advance(5.0);
        let row_h = 5.5;
        let hdr_y = layout.y_top;
        layout.fill_rect(MARGIN_MM, hdr_y, CONTENT_W_MM, row_h, colors::navy_light());
        layout.text_at(MARGIN_MM + 2.0, hdr_y + 4.0, "Step", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 52.0, hdr_y + 4.0, "Type", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 78.0, hdr_y + 4.0, "Author", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 118.0, hdr_y + 4.0, "Lead", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 142.0, hdr_y + 4.0, "Cycle", 7.0, true, colors::white());
        layout.advance(row_h);

        let mut nodes = map.nodes.clone();
        nodes.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        for (i, node) in nodes.iter().enumerate() {
            layout.ensure(row_h);
            let row_y = layout.y_top;
            if i % 2 == 0 {
                layout.fill_rect(MARGIN_MM, row_y, CONTENT_W_MM, row_h, colors::card_bg());
            }
            let lead = node
                .lead_time_minutes
                .filter(|v| *v > 0.0)
                .map(|v| vsm::format_duration(v, true))
                .unwrap_or_else(|| "-".into());
            let cycle = node
                .cycle_time_minutes
                .filter(|v| *v > 0.0)
                .map(|v| vsm::format_duration(v, true))
                .unwrap_or_else(|| "-".into());
            layout.text_at(
                MARGIN_MM + 2.0,
                row_y + 4.0,
                &truncate_chars(&node.label, 24),
                7.0,
                false,
                colors::text(),
            );
            layout.text_at(
                MARGIN_MM + 52.0,
                row_y + 4.0,
                vsm::node_type_label(&node.node_type),
                7.0,
                false,
                colors::muted(),
            );
            layout.text_at(
                MARGIN_MM + 78.0,
                row_y + 4.0,
                &truncate_chars(node.author.as_deref().unwrap_or("-"), 18),
                7.0,
                false,
                colors::text(),
            );
            layout.text_at(MARGIN_MM + 118.0, row_y + 4.0, &lead, 7.0, false, colors::text());
            layout.text_at(MARGIN_MM + 142.0, row_y + 4.0, &cycle, 7.0, false, colors::text());
            layout.advance(row_h);
        }
        layout.advance(2.0);
    }

    if !timeline.segments.is_empty() {
        layout.text_at(MARGIN_MM, layout.y_top, "Process timeline", 9.0, true, colors::navy());
        layout.advance(5.0);
        let row_h = 5.5;
        let hdr_y = layout.y_top;
        layout.fill_rect(MARGIN_MM, hdr_y, CONTENT_W_MM, row_h, colors::navy_light());
        layout.text_at(MARGIN_MM + 2.0, hdr_y + 4.0, "From", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 42.0, hdr_y + 4.0, "To", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 82.0, hdr_y + 4.0, "Flow", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 118.0, hdr_y + 4.0, "Duration", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 148.0, hdr_y + 4.0, "Author", 7.0, true, colors::white());
        layout.advance(row_h);

        for (i, segment) in timeline.segments.iter().enumerate() {
            layout.ensure(row_h);
            let row_y = layout.y_top;
            if i % 2 == 0 {
                layout.fill_rect(MARGIN_MM, row_y, CONTENT_W_MM, row_h, colors::card_bg());
            }
            let ft = vsm::flow_type_config(&segment.edge_type, &flow_types);
            let duration = if segment.duration_minutes > 0.0 {
                vsm::format_duration(segment.duration_minutes, true)
            } else {
                "-".into()
            };
            let author = segment
                .target_author
                .as_deref()
                .or(segment.source_author.as_deref())
                .unwrap_or("-");
            layout.text_at(
                MARGIN_MM + 2.0,
                row_y + 4.0,
                &truncate_chars(&segment.from_label, 16),
                7.0,
                false,
                colors::text(),
            );
            layout.text_at(
                MARGIN_MM + 42.0,
                row_y + 4.0,
                &truncate_chars(&segment.to_label, 16),
                7.0,
                false,
                colors::text(),
            );
            layout.text_at(
                MARGIN_MM + 82.0,
                row_y + 4.0,
                &truncate_chars(&ft.label, 12),
                7.0,
                false,
                colors::text(),
            );
            layout.text_at(MARGIN_MM + 118.0, row_y + 4.0, &duration, 7.0, false, colors::text());
            layout.text_at(
                MARGIN_MM + 148.0,
                row_y + 4.0,
                &truncate_chars(author, 14),
                7.0,
                false,
                colors::muted(),
            );
            layout.advance(row_h);
        }

        if timeline.stats.total_minutes > 0.0 {
            layout.advance(2.0);
            layout.text_at(MARGIN_MM, layout.y_top, "Timeline (proportional)", 8.5, true, colors::navy());
            layout.advance(5.0);
            let bar_h = 4.5;
            let track_w = CONTENT_W_MM - 48.0;
            for segment in &timeline.segments {
                if segment.duration_minutes <= 0.0 {
                    continue;
                }
                layout.ensure(bar_h + 1.5);
                let row_y = layout.y_top;
                let label = format!(
                    "{} -> {}",
                    truncate_chars(&segment.from_label, 10),
                    truncate_chars(&segment.to_label, 10)
                );
                layout.text_at(MARGIN_MM, row_y + 3.8, &label, 6.5, false, colors::muted());
                let track_x = MARGIN_MM + 46.0;
                layout.fill_rect(track_x, row_y, track_w, bar_h, colors::bar_bg());
                let pct = (segment.duration_minutes / timeline.stats.total_minutes) as f32;
                let fill_w = (track_w * pct).max(1.5);
                let ft = vsm::flow_type_config(&segment.edge_type, &flow_types);
                let bar_color = vsm::hex_to_rgb(&ft.color)
                    .map(|(r, g, b)| Rgb::new(r, g, b, None))
                    .unwrap_or_else(colors::cyan);
                layout.fill_rect(track_x, row_y, fill_w, bar_h, bar_color);
                layout.text_at(
                    track_x + fill_w + 1.5,
                    row_y + 3.8,
                    &vsm::format_duration(segment.duration_minutes, true),
                    6.5,
                    false,
                    colors::text(),
                );
                layout.advance(bar_h + 1.5);
            }
        }
    }

    if !map.messages.is_empty() {
        layout.advance(2.0);
        layout.text_at(MARGIN_MM, layout.y_top, "Messages", 9.0, true, colors::navy());
        layout.advance(5.0);
        for msg in &map.messages {
            for line in wrap_text(&format!("- {}", msg.text.trim()), 95) {
                layout.paragraph(&line);
            }
        }
    }

    layout.advance(4.0);
}

fn draw_vsm_diagram_pdf(
    layout: &mut PdfLayout,
    map: &ValueStreamMap,
    flow_types: &[vsm::ResolvedFlowType],
) {
    use crate::value_stream::VsmNode;

    let min_x = map.nodes.iter().map(|n| n.x).fold(f64::INFINITY, f64::min);
    let min_y = map.nodes.iter().map(|n| n.y).fold(f64::INFINITY, f64::min);
    let max_x = map
        .nodes
        .iter()
        .map(|n| n.x + n.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = map
        .nodes
        .iter()
        .map(|n| n.y + n.height)
        .fold(f64::NEG_INFINITY, f64::max);
    let content_w = (max_x - min_x).max(1.0);
    let content_h = (max_y - min_y).max(1.0);

    let diagram_h = 42.0_f32;
    layout.ensure(diagram_h + 4.0);
    let box_y = layout.y_top;
    layout.fill_rect(MARGIN_MM, box_y, CONTENT_W_MM, diagram_h, colors::card_bg());
    layout.stroke_rect(MARGIN_MM, box_y, CONTENT_W_MM, diagram_h, colors::bar_bg(), 0.4);

    let inner_w = CONTENT_W_MM as f64 - 6.0;
    let inner_h = diagram_h as f64 - 6.0;
    let scale = (inner_w / content_w).min(inner_h / content_h);
    let offset_x = MARGIN_MM as f64 + 3.0 + (inner_w - content_w * scale) / 2.0;
    let offset_y = box_y as f64 + 3.0 + (inner_h - content_h * scale) / 2.0;

    let node_by_id: HashMap<String, &VsmNode> =
        map.nodes.iter().map(|n| (n.id.clone(), n)).collect();

    let to_pdf = |nx: f64, ny: f64| -> (f32, f32) {
        (
            (offset_x + (nx - min_x) * scale) as f32,
            (offset_y + (ny - min_y) * scale) as f32,
        )
    };

    for edge in &map.edges {
        let Some(from) = node_by_id.get(&edge.from) else {
            continue;
        };
        let Some(to) = node_by_id.get(&edge.to) else {
            continue;
        };
        let (x1, y1) = to_pdf(from.x + from.width, from.y + from.height / 2.0);
        let (x2, y2) = to_pdf(to.x, to.y + to.height / 2.0);
        let ft = vsm::flow_type_config(&edge.edge_type, flow_types);
        let color = vsm::hex_to_rgb(&ft.color)
            .map(|(r, g, b)| Rgb::new(r, g, b, None))
            .unwrap_or_else(colors::muted);
        layout.stroke_line_xy(x1, y1, x2, y2, color, 0.6);
    }

    for node in &map.nodes {
        let (x, y) = to_pdf(node.x, node.y);
        let w = (node.width * scale) as f32;
        let h = (node.height * scale).max(6.0) as f32;
        let accent = vsm::hex_to_rgb(vsm::node_accent_color(&node.node_type))
            .map(|(r, g, b)| Rgb::new(r, g, b, None))
            .unwrap_or_else(colors::cyan);
        layout.fill_rect(x, y, w, h, colors::white());
        layout.stroke_rect(x, y, w, h, accent, 0.8);
        layout.text_at(
            x + 1.5,
            y + h / 2.0 + 1.0,
            &truncate_chars(&node.label, 16),
            6.0,
            true,
            colors::navy(),
        );
    }

    layout.advance(diagram_h + 4.0);
}

fn vendor_status_totals(result: &EvaluationResult) -> (usize, usize, usize, usize) {
    let mut compliant = 0usize;
    let mut partial = 0usize;
    let mut non_compliant = 0usize;
    let mut untested = 0usize;
    for pillar in &result.pillars {
        for req in &pillar.requirements {
            if !req.applicable {
                continue;
            }
            match req.status {
                ComplianceStatus::Compliant => compliant += 1,
                ComplianceStatus::Partial => partial += 1,
                ComplianceStatus::NonCompliant => non_compliant += 1,
                ComplianceStatus::Untested => untested += 1,
            }
        }
    }
    (compliant, partial, non_compliant, untested)
}

fn pillar_score_percent(result: &EvaluationResult, pillar_id: &str) -> f64 {
    result
        .pillars
        .iter()
        .find(|p| p.pillar_id == pillar_id)
        .map(|p| p.score.score_percent)
        .unwrap_or(0.0)
}

fn score_rgb(pct: f64) -> Rgb {
    if pct >= 90.0 {
        colors::compliant()
    } else if pct >= 70.0 {
        colors::partial()
    } else {
        colors::gap()
    }
}

fn estimate_req_height(req: &crate::pillar::Requirement) -> f32 {
    let desc_lines = wrap_text(req.description.trim(), 95).len() as f32;
    let mut h = 14.0 + desc_lines * 4.0;
    h += 4.5;
    if let Some(m) = &req.evaluation_method {
        h += wrap_text(&format!("Test: {}", m.trim()), 92).len() as f32 * 3.8;
    }
    if let Some(tc) = &req.technical_criteria {
        h += wrap_text(&format!("Criteria: {}", tc.trim()), 92).len() as f32 * 3.8;
    }
    h.max(18.0)
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::evaluation::{Evaluator, sample_vendors};
    use crate::policy::PolicyBundle;
    use crate::report::default_pdf_options;

    #[test]
    fn renders_pdf_bytes() {
        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let mut evaluator = Evaluator::new(bundle.clone());
        for (v, a) in sample_vendors() {
            evaluator.add_vendor(v, a);
        }
        let evaluation = evaluator.evaluate().expect("eval");
        let logo = std::path::Path::new("assets/logo.png");
        let options = default_pdf_options(logo.exists().then_some(logo));
        let pdf = render_pdf(&bundle, &evaluation, &HashMap::new(), &HashMap::new(), &options)
            .expect("pdf");
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 8_000);
    }
}
