pub mod error;
pub mod evaluation;
pub mod pillar;
pub mod pricing;
pub mod policy;
pub mod vendor_doc;
pub mod report;
pub mod scoring;
pub mod value_stream;
pub mod vendor;
pub mod vendor_set;
pub mod workspace;
pub mod workspace_bundle;

pub use error::MadError;
pub use evaluation::{sample_vendors, EvaluationReport, EvaluationResult, Evaluator};
pub use pillar::{Pillar, PillarId, Requirement, RequirementSeverity};
pub use pricing::{BillingPeriod, ProcurementConfig, VendorPricing};
pub use policy::PolicyBundle;
pub use vendor_doc::{
    dedupe_vendor_doc_item_ids, mdm_privacy_template_section, migrate_legacy_vendor_privacy,
    new_vendor_doc_id, new_vendor_doc_item_id, normalize_vendor_doc_section, VendorDocItem,
    VendorDocSection,
};
pub use report::{
    default_html_options, default_pdf_options, load_logo_data_uri, render_html, render_markdown,
    render_pdf, HtmlReportOptions, PdfReportOptions,
};
pub use scoring::ScoringConfig;
pub use value_stream::{
    ValueStreamEntry, ValueStreamMap, VsmEdge, VsmMessage, VsmNode, VsmNodeType,
};
pub use vendor::{
    ComplianceStatus, RequirementAssessment, Vendor, VendorAssessment, VendorId, VendorScore,
};
pub use vendor_set::{
    sanitize_assessment, VendorImportMode, VendorImportResult, VendorSetFile,
};
pub use workspace::EvaluationWorkspace;
pub use workspace_bundle::{
    parse_workspace_import, ParsedWorkspaceImport, WorkspaceBundle, WorkspaceImportResult,
};
