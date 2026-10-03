//! Command-specific CLI discovery; shared repository/output flags work on either side.
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Exact local visual review. Never auto-accepts or starts a server."
)]
pub struct Cli {
    #[arg(long, global = true, default_value = ".")]
    pub repo: PathBuf,
    #[arg(long, global = true, default_value = "visual.toml")]
    pub config: PathBuf,
    /// Emit one schema-1 JSON response. Producer stdout is redirected to stderr.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Operation,
}
#[derive(Args, Default)]
pub struct CompareArgs {
    /// Explicit sealed capture; never choose an implicit latest.
    #[arg(long)]
    pub run: PathBuf,
    #[arg(long)]
    pub baselines: Option<PathBuf>,
    #[arg(long)]
    pub report: Option<PathBuf>,
}
#[derive(Subcommand)]
pub enum Operation {
    /// Capture a fresh run and compare; writes runs/report, never references.
    Check {
        /// Preview resolved inputs and effects without invoking probes/producer or writing.
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        run: Option<PathBuf>,
        #[arg(long)]
        baselines: Option<PathBuf>,
        #[arg(long)]
        report: Option<PathBuf>,
        /// Exact story IDs; complete pairs/pages/timelines. Requires an explicit report.
        #[arg(long = "story", requires = "report")]
        stories: Vec<String>,
        /// Shared GUI/GPU lease; native producers still require consumer authorization.
        #[arg(long)]
        host_gate: bool,
        #[arg(long, requires = "host_gate")]
        compatibility_gate: Vec<PathBuf>,
    },
    /// Compare a sealed run; writes the original report, never captures or accepts.
    Compare(CompareArgs),
    /// Accept only explicitly reviewed IDs; changes references, never recaptures.
    Accept {
        #[command(flatten)]
        paths: CompareArgs,
        #[arg(long = "case")]
        cases: Vec<String>,
        /// Every change this report's review flags (changed/new stills, complete
        /// changed/new sequences), listed first; asks unless --yes.
        #[arg(long, conflicts_with = "cases")]
        all: bool,
        /// Explicit non-interactive confirmation for --all (rejected without --all).
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        adopt_environment: bool,
    },
    /// Losslessly compact historical provenance; never accepts images or changes environments.
    CompactBaselines {
        /// Explicit existing store (no config default).
        #[arg(long)]
        baselines: PathBuf,
        /// SHA-256 of the exact baseline.json to migrate; refuses concurrent changes.
        #[arg(long)]
        expected_hash: String,
        /// Validate and show byte savings without writing or locking.
        #[arg(long)]
        dry_run: bool,
    },
    /// Explicitly convert a store's reference format (e.g. to lossless WebP); every
    /// image is decoded and proven pixel-identical, never accepts or recaptures.
    ConvertBaselines {
        /// Explicit existing store (no config default).
        #[arg(long)]
        baselines: PathBuf,
        /// Target reference format: png or webp.
        #[arg(long)]
        format: String,
        /// SHA-256 of the exact baseline.json to migrate; refuses concurrent changes.
        #[arg(long)]
        expected_hash: String,
        /// Verify and report byte savings without writing or locking.
        #[arg(long)]
        dry_run: bool,
    },
    /// Explicitly partition approved environment metadata without changing old facts or images.
    PartitionEnvironment {
        #[arg(long)]
        baselines: PathBuf,
        #[arg(long)]
        expected_hash: String,
        /// Complete versioned environment JSON; must reconstruct the old observation exactly.
        #[arg(long)]
        environment: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Read-only config/source checks. Does not execute producers, probes or devices.
    Doctor {
        #[arg(long)]
        baselines: Option<PathBuf>,
        /// Legacy callers supplied --run to doctor; it has never validated that run.
        #[arg(long, hide = true)]
        run: Option<PathBuf>,
    },
    /// Print the bundled version-matched skill; no repository or installation needed.
    Skill {
        #[arg(long, value_parser = ["consumer"])]
        reference: Option<String>,
    },
    /// List the source-authored story catalog without building or capturing.
    List,
    /// Read stored report summaries or exact case/tick details. Never revalidates or writes.
    Inspect {
        #[arg(long)]
        report: PathBuf,
        #[arg(long = "case")]
        case: Option<String>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 12)]
        limit: usize,
        #[arg(long, requires = "case")]
        tick: Option<usize>,
    },
    /// Read-only source hashes for an explicit policy.
    Sources {
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        baselines: Option<PathBuf>,
    },
    /// Revalidate a sealed run's sources and images; never writes a report.
    Validate {
        #[arg(long)]
        run: PathBuf,
        #[arg(long)]
        baselines: Option<PathBuf>,
    },
    /// Review existing independent suites in a new explicit report; never captures.
    ReviewCollection {
        #[arg(long)]
        collection: PathBuf,
        #[arg(long)]
        report: PathBuf,
    },
    /// Accept exact suite:case selections from an explicitly reviewed collection.
    AcceptCollection {
        #[arg(long)]
        report: PathBuf,
        #[arg(long = "case")]
        cases: Vec<String>,
        #[arg(long = "adopt-suite")]
        adopt_suites: Vec<String>,
        /// Explicitly acknowledge each selected suite's withheld suite:base; coverage stays failing.
        #[arg(long = "acknowledge-withheld")]
        acknowledged_withheld: Vec<String>,
    },
}

// A normalized request keeps dispatch independent of clap without duplicating parsing.
#[derive(Default)]
pub struct Request {
    pub command: &'static str,
    pub repo: PathBuf,
    pub config: PathBuf,
    pub json: bool,
    pub dry_run: bool,
    pub expected_hash: Option<String>,
    pub environment: Option<PathBuf>,
    pub format: Option<String>,
    pub run: Option<PathBuf>,
    pub baselines: Option<PathBuf>,
    pub report: Option<PathBuf>,
    pub cases: Vec<String>,
    pub all: bool,
    pub yes: bool,
    pub stories: Vec<String>,
    pub adopt_environment: bool,
    pub host_gate: bool,
    pub compatibility_gate: Vec<PathBuf>,
    pub collection: Option<PathBuf>,
    pub adopt_suites: Vec<String>,
    pub acknowledged_withheld: Vec<String>,
    pub policy: Option<PathBuf>,
    pub inspect: cargo_gpui_visual::inspect::Options,
    pub reference: Option<String>,
}
impl From<Cli> for Request {
    fn from(cli: Cli) -> Self {
        let mut r = Self {
            repo: cli.repo,
            config: cli.config,
            json: cli.json,
            ..Default::default()
        };
        match cli.command {
            Operation::Check {
                dry_run,
                run,
                baselines,
                report,
                stories,
                host_gate,
                compatibility_gate,
            } => {
                r.command = "check";
                r.dry_run = dry_run;
                r.run = run;
                r.baselines = baselines;
                r.report = report;
                r.stories = stories;
                r.host_gate = host_gate;
                r.compatibility_gate = compatibility_gate;
            }
            Operation::Compare(p) => {
                r.command = "compare";
                r.paths(p);
            }
            Operation::Accept {
                paths,
                cases,
                all,
                yes,
                adopt_environment,
            } => {
                r.command = "accept";
                r.paths(paths);
                r.cases = cases;
                r.all = all;
                r.yes = yes;
                r.adopt_environment = adopt_environment;
            }
            Operation::CompactBaselines {
                baselines,
                expected_hash,
                dry_run,
            } => {
                r.command = "compact-baselines";
                r.baselines = Some(baselines);
                r.expected_hash = Some(expected_hash);
                r.dry_run = dry_run;
            }
            Operation::ConvertBaselines {
                baselines,
                format,
                expected_hash,
                dry_run,
            } => {
                r.command = "convert-baselines";
                r.baselines = Some(baselines);
                r.format = Some(format);
                r.expected_hash = Some(expected_hash);
                r.dry_run = dry_run;
            }
            Operation::PartitionEnvironment {
                baselines,
                expected_hash,
                environment,
                dry_run,
            } => {
                r.command = "partition-environment";
                r.baselines = Some(baselines);
                r.expected_hash = Some(expected_hash);
                r.environment = Some(environment);
                r.dry_run = dry_run;
            }
            Operation::Doctor { baselines, .. } => {
                r.command = "doctor";
                r.baselines = baselines;
            }
            Operation::List => r.command = "list",
            Operation::Skill { reference } => {
                r.command = "skill";
                r.reference = reference;
            }
            Operation::Inspect {
                report,
                case,
                offset,
                limit,
                tick,
            } => {
                r.command = "inspect";
                r.report = Some(report);
                r.inspect = cargo_gpui_visual::inspect::Options {
                    case,
                    offset,
                    limit,
                    tick,
                };
            }
            Operation::Sources { policy, baselines } => {
                r.command = "sources";
                r.policy = Some(policy);
                r.baselines = baselines;
            }
            Operation::Validate { run, baselines } => {
                r.command = "validate";
                r.run = Some(run);
                r.baselines = baselines;
            }
            Operation::ReviewCollection { collection, report } => {
                r.command = "review-collection";
                r.collection = Some(collection);
                r.report = Some(report);
            }
            Operation::AcceptCollection {
                report,
                cases,
                adopt_suites,
                acknowledged_withheld,
            } => {
                r.command = "accept-collection";
                r.report = Some(report);
                r.cases = cases;
                r.adopt_suites = adopt_suites;
                r.acknowledged_withheld = acknowledged_withheld;
            }
        }
        r
    }
}
impl Request {
    fn paths(&mut self, paths: CompareArgs) {
        self.run = Some(paths.run);
        self.baselines = paths.baselines;
        self.report = paths.report;
    }
}
