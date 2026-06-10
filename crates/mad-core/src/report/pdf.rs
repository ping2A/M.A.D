use std::io::{BufWriter, Cursor};

use image::io::Reader as ImageReader;
use printpdf::image::Image;
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{
    BuiltinFont, Color, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerIndex, PdfLayerReference,
    PdfPageIndex, Point, Polygon, Rgb,
};

use crate::evaluation::{EvaluationReport, EvaluationResult};
use crate::pillar::RequirementSeverity;
use crate::policy::PolicyBundle;
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
const CONTENT_W_MM: f32 = PAGE_W_MM - 2.0 * MARGIN_MM;

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
    font: printpdf::IndirectFontRef,
    font_bold: printpdf::IndirectFontRef,
}

impl PdfLayout {
    fn new(doc: PdfDocumentReference, page: PdfPageIndex, layer: PdfLayerIndex) -> Self {
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).expect("font");
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .expect("bold font");
        Self {
            doc,
            page,
            layer,
            y_top: MARGIN_MM,
            font,
            font_bold,
        }
    }

    fn layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.page).get_layer(self.layer)
    }

    fn ensure(&mut self, height_mm: f32) {
        if self.y_top + height_mm > PAGE_H_MM - MARGIN_MM {
            let (page, layer) = self.doc.add_page(Mm(PAGE_W_MM), Mm(PAGE_H_MM), "Page");
            self.page = page;
            self.layer = layer;
            self.y_top = MARGIN_MM;
        }
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
        let layer = self.layer();
        layer.set_outline_color(Color::Rgb(color));
        layer.set_outline_thickness(thickness);
        let y = PAGE_H_MM - y_top;
        let line = Line {
            points: vec![
                (Point::new(Mm(x1), Mm(y)), false),
                (Point::new(Mm(x2), Mm(y)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
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
            content,
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
    options: &PdfReportOptions,
) -> Result<Vec<u8>, String> {
    let (doc, page1, layer1) =
        PdfDocument::new("MAD Report", Mm(PAGE_W_MM), Mm(PAGE_H_MM), "Layer 1");
    let mut layout = PdfLayout::new(doc, page1, layer1);

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
        "Mobile Assessment & Defense — MDM Evaluation Report",
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
    let generated = options.generated_at.as_deref().unwrap_or("—");
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
    layout.text_at(
        MARGIN_MM + 3.0,
        meta_y + 10.0,
        &format!("Generated {generated}"),
        7.5,
        false,
        colors::silver(),
    );
    layout.advance(meta_h + 5.0);

    // ── Section 1 ────────────────────────────────────────────────────────
    layout.section_heading("1. Purpose and Scope");
    layout.paragraph(
        "MAD (Mobile Assessment & Defense) is an evaluation-only platform. It assesses whether \
         candidate MDM vendors meet a corporate mobile security standard before procurement. It \
         does not enroll devices or enforce policies.",
    );

    let scope_y = layout.y_top;
    let scope_h = 28.0;
    let scope_w = (CONTENT_W_MM - 4.0) / 2.0;
    layout.fill_rect(MARGIN_MM, scope_y, scope_w, scope_h, colors::scope_in_bg());
    layout.fill_rect(
        MARGIN_MM + scope_w + 4.0,
        scope_y,
        scope_w,
        scope_h,
        colors::scope_out_bg(),
    );
    layout.fill_rect(MARGIN_MM, scope_y, 1.5, scope_h, colors::compliant());
    layout.fill_rect(MARGIN_MM + scope_w + 4.0, scope_y, 1.5, scope_h, colors::gap());
    layout.text_at(MARGIN_MM + 4.0, scope_y + 5.0, "IN SCOPE", 8.0, true, colors::compliant());
    let in_x = MARGIN_MM + 4.0;
    for (i, item) in [
        "iOS MDM (ABM, supervised mode)",
        "Android Enterprise (Work Profile, COBO, kiosk)",
        "Vendor capability assessment & scoring",
    ]
    .iter()
    .enumerate()
    {
        layout.text_at(
            in_x,
            scope_y + 10.0 + i as f32 * 4.5,
            &format!("• {item}"),
            8.5,
            false,
            colors::text(),
        );
    }
    layout.text_at(
        MARGIN_MM + scope_w + 8.0,
        scope_y + 5.0,
        "OUT OF SCOPE",
        8.0,
        true,
        colors::gap(),
    );
    let out_x = MARGIN_MM + scope_w + 8.0;
    for (i, item) in [
        "Desktop / laptop management",
        "Post-selection policy enforcement",
        "Device deployment",
    ]
    .iter()
    .enumerate()
    {
        layout.text_at(
            out_x,
            scope_y + 10.0 + i as f32 * 4.5,
            &format!("• {item}"),
            8.5,
            false,
            colors::text(),
        );
    }
    layout.advance(scope_h + 6.0);

    // ── Section 2 ────────────────────────────────────────────────────────
    layout.section_heading("2. Evaluation Methodology");
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
    layout.text_at(
        MARGIN_MM + 3.0,
        code_y + 11.0,
        "overall_score = mean(cybersecurity, dfir, platform_os)",
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

    // ── Section 3 ────────────────────────────────────────────────────────
    layout.section_heading("3. Requirements and Technical Criteria");
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
    layout.section_heading("4. Vendor Assessment Results");

    let mut ranked: Vec<_> = evaluation.vendors.iter().collect();
    ranked.sort_by(|a, b| {
        b.overall_score
            .overall_score_percent
            .partial_cmp(&a.overall_score.overall_score_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    layout.subheading("Comparison summary");
    let pillar_defs: Vec<(String, String)> = ranked
        .first()
        .map(|r| {
            r.pillars
                .iter()
                .map(|p| (p.pillar_id.clone(), p.pillar_name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let pillar_count = pillar_defs.len().max(1) as f32;
    let pillar_col_w = 58.0 / pillar_count;
    let status_col = MARGIN_MM + 68.0 + pillar_col_w * pillar_count as f32;

    let lb_y = layout.y_top;
    let lb_row = 7.5;
    layout.fill_rect(MARGIN_MM, lb_y, CONTENT_W_MM, lb_row, colors::navy_light());
    layout.text_at(MARGIN_MM + 2.0, lb_y + 5.2, "#", 7.0, true, colors::white());
    layout.text_at(MARGIN_MM + 10.0, lb_y + 5.2, "Vendor", 7.0, true, colors::white());
    layout.text_at(MARGIN_MM + 52.0, lb_y + 5.2, "Overall", 7.0, true, colors::white());
    for (i, (_, name)) in pillar_defs.iter().enumerate() {
        layout.text_at(
            MARGIN_MM + 68.0 + i as f32 * pillar_col_w,
            lb_y + 5.2,
            &truncate(name, 10),
            6.5,
            true,
            colors::white(),
        );
    }
    layout.text_at(status_col + 6.0, lb_y + 5.2, "OK", 7.0, true, colors::white());
    layout.text_at(status_col + 18.0, lb_y + 5.2, "Part", 7.0, true, colors::white());
    layout.text_at(status_col + 30.0, lb_y + 5.2, "Gap", 7.0, true, colors::white());
    layout.text_at(status_col + 42.0, lb_y + 5.2, "Crit", 7.0, true, colors::white());
    layout.advance(lb_row);

    let show_price = evaluation.procurement.device_count > 0
        && ranked.iter().any(|r| r.overall_score.annual_cost_per_device.is_some());
    if show_price {
        let pr_y = layout.y_top;
        layout.fill_rect(MARGIN_MM, pr_y, CONTENT_W_MM, lb_row, colors::navy_light());
        layout.text_at(MARGIN_MM + 10.0, pr_y + 5.2, "Vendor", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 70.0, pr_y + 5.2, "Annual/device", 7.0, true, colors::white());
        layout.text_at(MARGIN_MM + 110.0, pr_y + 5.2, "Total annual", 7.0, true, colors::white());
        if evaluation.procurement.use_price_in_ranking {
            layout.text_at(MARGIN_MM + 150.0, pr_y + 5.2, "Composite", 7.0, true, colors::white());
        }
        layout.advance(lb_row);
        for (i, result) in ranked.iter().enumerate() {
            let row_y = layout.y_top;
            if i % 2 == 0 {
                layout.fill_rect(MARGIN_MM, row_y, CONTENT_W_MM, lb_row, colors::card_bg());
            }
            let os = &result.overall_score;
            layout.text_at(
                MARGIN_MM + 10.0,
                row_y + 5.2,
                &truncate(&result.vendor.name, 22),
                7.5,
                false,
                colors::navy(),
            );
            if let Some(cost) = os.annual_cost_per_device {
                let cur = os.price_currency.as_deref().unwrap_or("USD");
                layout.text_at(
                    MARGIN_MM + 70.0,
                    row_y + 5.2,
                    &format!("{cur} {cost:.0}"),
                    7.0,
                    false,
                    colors::text(),
                );
            }
            if let Some(total) = os.total_annual_cost {
                let cur = os.price_currency.as_deref().unwrap_or("USD");
                layout.text_at(
                    MARGIN_MM + 110.0,
                    row_y + 5.2,
                    &format!("{cur} {total:.0}"),
                    7.0,
                    false,
                    colors::text(),
                );
            }
            if let Some(comp) = os.composite_score_percent {
                layout.text_at(
                    MARGIN_MM + 150.0,
                    row_y + 5.2,
                    &format!("{comp:.1}%"),
                    7.5,
                    true,
                    score_rgb(comp),
                );
            }
            layout.advance(lb_row);
        }
        layout.advance(3.0);
    }

    for (i, result) in ranked.iter().enumerate() {
        let row_y = layout.y_top;
        layout.ensure(lb_row);
        if i % 2 == 0 {
            layout.fill_rect(MARGIN_MM, row_y, CONTENT_W_MM, lb_row, colors::card_bg());
        }
        let score = result.overall_score.overall_score_percent;
        let (ok, part, gap, untested) = vendor_status_totals(result);
        let _ = untested;
        layout.text_at(
            MARGIN_MM + 2.0,
            row_y + 5.2,
            &format!("{}", i + 1),
            7.5,
            true,
            colors::muted(),
        );
        layout.text_at(
            MARGIN_MM + 10.0,
            row_y + 5.2,
            &truncate(&result.vendor.name, 22),
            7.5,
            true,
            colors::navy(),
        );
        layout.text_at(
            MARGIN_MM + 52.0,
            row_y + 5.2,
            &format!("{score:.0}%"),
            7.5,
            true,
            score_rgb(score),
        );
        for (i, (pid, _)) in pillar_defs.iter().enumerate() {
            let pct = pillar_score_percent(result, pid);
            layout.text_at(
                MARGIN_MM + 68.0 + i as f32 * pillar_col_w,
                row_y + 5.2,
                &format!("{pct:.0}%"),
                7.0,
                false,
                score_rgb(pct),
            );
        }
        layout.text_at(status_col + 6.0, row_y + 5.2, &format!("{ok}"), 7.0, false, colors::compliant());
        layout.text_at(status_col + 18.0, row_y + 5.2, &format!("{part}"), 7.0, false, colors::partial());
        layout.text_at(status_col + 30.0, row_y + 5.2, &format!("{gap}"), 7.0, false, colors::gap());
        let crit = result.overall_score.critical_gaps.len();
        layout.text_at(
            status_col + 42.0,
            row_y + 5.2,
            &format!("{crit}"),
            7.0,
            true,
            if crit > 0 { colors::gap() } else { colors::muted() },
        );
        layout.advance(lb_row);
    }
    layout.advance(4.0);

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
        let score = result.overall_score.overall_score_percent;
        let card_y = layout.y_top;
        layout.ensure(30.0);

        let notes_lines = result
            .pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .filter(|r| r.notes.as_ref().is_some_and(|n| !n.trim().is_empty()))
            .count() as f32;
        let website_lines = if result.vendor.website.as_ref().is_some_and(|w| !w.is_empty()) {
            1.0
        } else {
            0.0
        };
        let card_h = 24.0
            + website_lines * 4.5
            + result.pillars.len() as f32 * 8.0
            + result
                .pillars
                .iter()
                .map(|p| p.requirements.len() as f32 * 5.5)
                .sum::<f32>()
            + notes_lines * 4.0
            + if result.overall_score.critical_gaps.is_empty() {
                0.0
            } else {
                12.0
            };

        layout.fill_rect(MARGIN_MM, card_y, CONTENT_W_MM, card_h, colors::white());
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
        layout.text_at(
            MARGIN_MM + CONTENT_W_MM - 28.0,
            card_y + 6.0,
            &format!("{score:.1}%"),
            13.0,
            true,
            score_rgb(score),
        );

        let (ok, part, gap, untested) = vendor_status_totals(result);
        layout.text_at(
            MARGIN_MM + 6.0,
            card_y + 12.0,
            &format!("OK {ok} · Partial {part} · Gap {gap} · Untested {untested}"),
            7.5,
            false,
            colors::muted(),
        );

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
                layout.ensure(5.0);
                layout.status_badge(MARGIN_MM + 8.0, inner_y, req.status);
                let sev_w = layout.severity_badge(MARGIN_MM + 20.0, inner_y, req.severity);
                layout.text_at(
                    MARGIN_MM + 20.0 + sev_w + 2.0,
                    inner_y + 3.5,
                    &format!("{} {}", req.requirement_id, truncate(&req.title, 48)),
                    7.5,
                    false,
                    colors::text(),
                );
                inner_y += 5.0;
                if let Some(notes) = &req.notes {
                    if !notes.trim().is_empty() {
                        for line in wrap_text(&format!("Note: {}", notes.trim()), 88) {
                            layout.text_at(MARGIN_MM + 22.0, inner_y, &line, 7.0, false, colors::muted());
                            inner_y += 3.8;
                        }
                    }
                }
            }
            inner_y += 1.5;
        }

        if !result.overall_score.critical_gaps.is_empty() {
            let gap_y = inner_y;
            let gap_h = 8.0 + result.overall_score.critical_gaps.len() as f32 * 4.0;
            layout.fill_rect(MARGIN_MM + 4.0, gap_y, CONTENT_W_MM - 8.0, gap_h, colors::scope_out_bg());
            layout.text_at(
                MARGIN_MM + 7.0,
                gap_y + 5.0,
                "Critical gaps",
                8.5,
                true,
                colors::gap(),
            );
            for (gi, gap) in result.overall_score.critical_gaps.iter().enumerate() {
                layout.text_at(
                    MARGIN_MM + 7.0,
                    gap_y + 9.0 + gi as f32 * 4.0,
                    &format!("• {gap}"),
                    7.5,
                    false,
                    colors::gap(),
                );
            }
        }

        layout.y_top = card_y + card_h + 5.0;
    }

    // Footer
    layout.advance(4.0);
    layout.stroke_line(MARGIN_MM, layout.y_top, MARGIN_MM + CONTENT_W_MM, colors::bar_bg(), 0.4);
    layout.advance(3.0);
    layout.text_block(
        "Generated by MAD — Mobile Assessment & Defense. \
         Scores reflect the active workspace assessments and scoring model at export time.",
        7.5,
        false,
        colors::muted(),
        100,
        3.8,
    );

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    layout
        .doc
        .save(&mut buffer)
        .map_err(|e| format!("failed to write PDF: {e}"))?;
    Ok(buffer.into_inner().map_err(|e| e.to_string())?.into_inner())
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

fn vendor_status_totals(result: &EvaluationResult) -> (usize, usize, usize, usize) {
    let mut compliant = 0usize;
    let mut partial = 0usize;
    let mut non_compliant = 0usize;
    let mut untested = 0usize;
    for pillar in &result.pillars {
        for req in &pillar.requirements {
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
    if req.evaluation_method.is_some() {
        h += 5.0;
    }
    if req.technical_criteria.is_some() {
        h += 5.0;
    }
    h.max(18.0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
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
        let pdf = render_pdf(&bundle, &evaluation, &options).expect("pdf");
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 8_000);
    }
}
