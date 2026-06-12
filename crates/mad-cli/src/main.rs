use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use mad_core::{
    default_html_options, default_pdf_options, render_html, render_markdown, render_pdf,
    Evaluator, PolicyBundle,
};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "mad",
    about = "MAD — Mobile Assessment & Defense, mobile MDM vendor evaluation CLI (evaluation only)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List evaluation pillars and requirements from policy files
    Policy {
        #[arg(long, default_value = "policies")]
        policies_dir: PathBuf,
    },
    /// Evaluate MDM vendors against loaded policies
    Evaluate {
        #[arg(long, default_value = "policies")]
        policies_dir: PathBuf,
        /// Output format: table or json
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// Generate a shareable technical evaluation report
    Report {
        #[arg(long, default_value = "policies")]
        policies_dir: PathBuf,
        /// Output format: html (shareable), pdf, or md
        #[arg(long, default_value = "html", value_enum)]
        format: ReportFormat,
        /// Write report to file (recommended for HTML sharing)
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// Path to logo PNG embedded in HTML (default: assets/logo.png)
        #[arg(long, default_value = "assets/logo.png")]
        logo: PathBuf,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Clone, clap::ValueEnum, Default)]
enum ReportFormat {
    #[default]
    Html,
    Pdf,
    Md,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Policy { policies_dir } => cmd_policy(&policies_dir),
        Commands::Evaluate {
            policies_dir,
            format,
        } => cmd_evaluate(&policies_dir, format),
        Commands::Report {
            policies_dir,
            output,
            format,
            logo,
        } => cmd_report(&policies_dir, output.as_ref(), format, &logo),
    }
}

fn cmd_policy(policies_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let bundle = PolicyBundle::load_dir(policies_dir)?;

    println!("MAD Policy Bundle");
    println!("  Pillars:     {}", bundle.pillar_count());
    println!("  Requirements: {}", bundle.total_requirements());
    println!("  Critical:    {}", bundle.critical_requirements());
    println!();

    for pillar in &bundle.pillars {
        println!("▸ {} ({})", pillar.name, pillar.id.as_str());
        println!("  {}", pillar.description);
        for req in &pillar.requirements {
            let severity = match req.severity {
                mad_core::RequirementSeverity::Critical => "CRITICAL",
                mad_core::RequirementSeverity::High => "HIGH",
                mad_core::RequirementSeverity::Medium => "MEDIUM",
            };
            println!("    [{severity}] {} — {}", req.id, req.title);
        }
        println!();
    }

    Ok(())
}

fn cmd_evaluate(
    policies_dir: &PathBuf,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle = PolicyBundle::load_dir(policies_dir)?;
    let mut evaluator = Evaluator::new(bundle);

    for (vendor, assessment) in mad_core::sample_vendors() {
        evaluator.add_vendor(vendor, assessment);
    }

    let report = evaluator.evaluate()?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        OutputFormat::Table => {
            println!("MAD Vendor Evaluation Report");
            println!("  Policy version: {}", report.policy_version);
            println!("  Requirements:   {}", report.total_requirements);
            println!();

            let mut ranked: Vec<_> = report.vendors.iter().collect();
            ranked.sort_by(|a, b| {
                b.overall_score
                    .overall_score_percent
                    .partial_cmp(&a.overall_score.overall_score_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for (rank, result) in ranked.iter().enumerate() {
                println!(
                    "#{} {} — {:.1}%",
                    rank + 1,
                    result.vendor.name,
                    result.overall_score.overall_score_percent
                );
                for pillar in &result.pillars {
                    println!(
                        "    {} — {:.1}% ({} compliant, {} partial, {} gaps)",
                        pillar.pillar_name,
                        pillar.score.score_percent,
                        pillar.score.compliant,
                        pillar.score.partial,
                        pillar.score.non_compliant
                    );
                }
                if !result.overall_score.critical_gaps.is_empty() {
                    println!("    Critical gaps:");
                    for gap in &result.overall_score.critical_gaps {
                        println!("      • {gap}");
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}

fn cmd_report(
    policies_dir: &PathBuf,
    output: Option<&PathBuf>,
    format: ReportFormat,
    logo: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle = PolicyBundle::load_dir(policies_dir)?;
    let mut evaluator = Evaluator::new(bundle.clone());

    for (vendor, assessment) in mad_core::sample_vendors() {
        evaluator.add_vendor(vendor, assessment);
    }

    let evaluation = evaluator.evaluate()?;
    let value_streams: HashMap<String, Vec<mad_core::ValueStreamEntry>> = HashMap::new();
    let vendor_docs: HashMap<String, Vec<mad_core::VendorDocSection>> = HashMap::new();

    let default_output = match format {
        ReportFormat::Md => PathBuf::from("mad-evaluation-report.md"),
        ReportFormat::Html => PathBuf::from("mad-evaluation-report.html"),
        ReportFormat::Pdf => PathBuf::from("mad-evaluation-report.pdf"),
    };
    let path = output.cloned().unwrap_or(default_output);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    match format {
        ReportFormat::Md => {
            let content = render_markdown(&bundle, &evaluation, &value_streams, &vendor_docs);
            std::fs::write(&path, content)?;
        }
        ReportFormat::Html => {
            let logo_path = if logo.exists() { Some(logo.as_path()) } else { None };
            let options = default_html_options(logo_path);
            let content =
                render_html(&bundle, &evaluation, &value_streams, &vendor_docs, &options);
            std::fs::write(&path, content)?;
            eprintln!("Open in any browser or share as a single self-contained file.");
        }
        ReportFormat::Pdf => {
            let logo_path = if logo.exists() { Some(logo.as_path()) } else { None };
            let options = default_pdf_options(logo_path);
            let content =
                render_pdf(&bundle, &evaluation, &value_streams, &vendor_docs, &options)?;
            std::fs::write(&path, content)?;
        }
    }

    eprintln!("Report written to {}", path.display());

    Ok(())
}
