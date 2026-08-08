use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use bbr_persistence::{
    CommandHistoryEntry, DownloadOutcome, DownloadRecord, GlyphName, Profile, ProfileHue,
    RecentlyClosedTab, Repository, SearchEngineId, SearchSettings, SidebarState, Snapshot, Space,
    SpaceGlyph, SpaceTint, Split, Tab, WindowGeometry, WindowState, WriteStats,
};

const WARMUPS: usize = 10;
const SAMPLES: usize = 100;
static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "bbr-persistence-benchmark-{label}-{}-{}.sqlite",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_file(&path);
        Self(path)
    }

    fn first_run(label: &str) -> Self {
        let db = Self::new(label);
        rusqlite::Connection::open(&db.0)
            .unwrap()
            .pragma_update(None, "user_version", 0)
            .unwrap();
        db
    }

    fn current(label: &str) -> Self {
        let db = Self::new(label);
        Repository::new(&db.0).initialize_and_restore().unwrap();
        db
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(format!("{}-wal", self.0.display()));
        let _ = fs::remove_file(format!("{}-shm", self.0.display()));
    }
}

#[derive(Clone, Debug)]
struct Operation {
    name: String,
    warmups: usize,
    samples: Vec<u128>,
    write_stats: Option<WriteStats>,
}

impl Operation {
    fn render(&self) -> String {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let p50 = percentile(&sorted, 50);
        let p95 = percentile(&sorted, 95);
        let max = *sorted.last().unwrap();
        let stats = self.write_stats.unwrap_or_default();
        format!(
            "operation={}\nwarmups={}\nsample_count={}\nraw_samples_ns={}\np50_ns={}\np95_ns={}\nmax_ns={}\nwrite_transactions={}\nwrite_sql_statements={}\nwrite_deleted_rows={}\nwrite_inserted_rows={}\nend_operation\n",
            self.name,
            self.warmups,
            self.samples.len(),
            self.samples.iter().map(u128::to_string).collect::<Vec<_>>().join(","),
            p50,
            p95,
            max,
            stats.committed_transactions,
            stats.sql_statements,
            stats.deleted_rows,
            stats.inserted_rows,
        )
    }
}

#[derive(Debug)]
struct ParsedArtifact {
    metadata: BTreeMap<String, String>,
    operations: BTreeMap<String, Operation>,
}

#[test]
#[ignore = "release-only distribution evidence"]
fn repository_benchmark() {
    if cfg!(debug_assertions) {
        panic!("benchmark must run with --release");
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let root = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path.join("bbr-persistence-artifacts")
            } else {
                workspace.join(path).join("bbr-persistence-artifacts")
            }
        }
        None => workspace.join("target/bbr-persistence-artifacts"),
    };
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let artifact = root.join("repository-benchmark.txt");
    let run_id = format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::process::id()
    );

    let first_run_databases = (0..WARMUPS + SAMPLES)
        .map(|index| TempDb::first_run(&format!("first-run-{index}")))
        .collect::<Vec<_>>();
    let current = TempDb::current("current");
    let current_repository = Repository::new(&current.0);
    let write_databases = (0..WARMUPS + SAMPLES)
        .map(|index| TempDb::current(&format!("write-{index}")))
        .collect::<Vec<_>>();
    let dense = dense_snapshot();

    let first_run = measure("schema_v2_first_run_load", |index| {
        let outcome = Repository::new(&first_run_databases[index].0)
            .initialize_and_restore()
            .unwrap();
        assert_eq!(outcome.snapshot, Snapshot::default());
        None
    });
    let current_load = measure("schema_v2_current_load", |_| {
        let outcome = current_repository.restore_read_only().unwrap();
        assert_eq!(outcome.snapshot, Snapshot::default());
        None
    });
    let dense_write = measure("dense_structural_write", |index| {
        let repository = Repository::new(&write_databases[index].0);
        let stats = repository.replace_snapshot(&dense).unwrap();
        assert_eq!(stats.committed_transactions, 1);
        assert_eq!(stats.sql_statements, 517);
        assert_eq!(stats.inserted_rows, 508);
        assert_eq!(repository.restore_read_only().unwrap().snapshot, dense);
        Some(stats)
    });
    let operations = [first_run, current_load, dense_write];
    let contents = render_artifact(&artifact, &run_id, &operations);
    let temporary = root.join(format!(".repository-benchmark-{}.tmp", std::process::id()));
    fs::write(&temporary, &contents).unwrap();
    fs::rename(&temporary, &artifact).unwrap();

    let persisted = fs::read_to_string(&artifact).unwrap();
    let parsed = parse_artifact(&persisted, &run_id).unwrap();
    assert_eq!(parsed.operations.len(), 3);
    assert_eq!(
        parsed.metadata["artifact_path"],
        artifact.display().to_string()
    );
    assert!(parse_artifact(&persisted, "stale-run-id").is_err());
    assert!(parse_artifact(
        &persisted.replace("operation_count=3", "operation_count=2"),
        &run_id
    )
    .is_err());
    assert!(parse_artifact(
        &persisted.replace("sample_count=100", "sample_count=99"),
        &run_id
    )
    .is_err());
    let duplicate = format!("{persisted}{}", operations[0].render());
    assert!(parse_artifact(&duplicate, &run_id).is_err());

    println!("artifact_path={}", artifact.display());
    println!("run_id={run_id}");
    for operation in operations {
        let mut sorted = operation.samples.clone();
        sorted.sort_unstable();
        println!(
            "operation={} warmups={} samples={} p50_ns={} p95_ns={} max_ns={} write_transactions={} write_sql_statements={} write_deleted_rows={} write_inserted_rows={}",
            operation.name,
            operation.warmups,
            operation.samples.len(),
            percentile(&sorted, 50),
            percentile(&sorted, 95),
            sorted.last().unwrap(),
            operation.write_stats.unwrap_or_default().committed_transactions,
            operation.write_stats.unwrap_or_default().sql_statements,
            operation.write_stats.unwrap_or_default().deleted_rows,
            operation.write_stats.unwrap_or_default().inserted_rows,
        );
    }
}

fn measure(
    mut name: &'static str,
    mut operation: impl FnMut(usize) -> Option<WriteStats>,
) -> Operation {
    let mut samples = Vec::with_capacity(SAMPLES);
    let mut write_stats = None;
    for index in 0..WARMUPS + SAMPLES {
        let started = Instant::now();
        let stats = operation(index);
        let elapsed = started.elapsed().as_nanos();
        if let Some(stats) = stats {
            if let Some(expected) = write_stats {
                assert_eq!(expected, stats);
            }
            write_stats = Some(stats);
        }
        if index >= WARMUPS {
            samples.push(elapsed);
        }
    }
    Operation {
        name: std::mem::take(&mut name).into(),
        warmups: WARMUPS,
        samples,
        write_stats,
    }
}

fn render_artifact(path: &Path, run_id: &str, operations: &[Operation]) -> String {
    let mut result = format!(
        "format=bbr-persistence-benchmark-v1\nartifact_path={}\nrun_id={}\npackage={}\nversion={}\nprofile=release\ntarget_os={}\ntarget_arch={}\navailable_parallelism={}\noperation_count={}\n",
        path.display(), run_id, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),
        std::env::consts::OS, std::env::consts::ARCH,
        std::thread::available_parallelism().map(usize::from).unwrap_or(1), operations.len(),
    );
    for operation in operations {
        result.push_str(&operation.render());
    }
    result
}

fn parse_artifact(contents: &str, expected_run_id: &str) -> Result<ParsedArtifact, String> {
    let mut metadata = BTreeMap::new();
    let mut operations = BTreeMap::new();
    let mut current: Option<BTreeMap<String, String>> = None;
    for line in contents.lines() {
        let (key, value) = line.split_once('=').unwrap_or((line, ""));
        if key == "operation" {
            if current.is_some() {
                return Err("nested operation".into());
            }
            let mut block = BTreeMap::new();
            block.insert(key.into(), value.into());
            current = Some(block);
        } else if key == "end_operation" {
            let block = current.take().ok_or("orphan operation end")?;
            let operation = parse_operation(&block)?;
            if operations
                .insert(operation.name.clone(), operation)
                .is_some()
            {
                return Err("duplicate operation".into());
            }
        } else if let Some(block) = &mut current {
            if block.insert(key.into(), value.into()).is_some() {
                return Err("duplicate operation field".into());
            }
        } else if metadata.insert(key.into(), value.into()).is_some() {
            return Err("duplicate metadata".into());
        }
    }
    if current.is_some() {
        return Err("unterminated operation".into());
    }
    for field in [
        "format",
        "artifact_path",
        "run_id",
        "package",
        "version",
        "profile",
        "target_os",
        "target_arch",
        "available_parallelism",
        "operation_count",
    ] {
        if metadata.get(field).is_none_or(String::is_empty) {
            return Err(format!("missing metadata {field}"));
        }
    }
    if metadata["run_id"] != expected_run_id {
        return Err("stale run identifier".into());
    }
    let operation_count = metadata["operation_count"]
        .parse::<usize>()
        .map_err(|_| "operation count")?;
    if operation_count != operations.len() || operations.len() != 3 {
        return Err("wrong operation count".into());
    }
    for required in [
        "schema_v2_first_run_load",
        "schema_v2_current_load",
        "dense_structural_write",
    ] {
        if !operations.contains_key(required) {
            return Err(format!("missing operation {required}"));
        }
    }
    Ok(ParsedArtifact {
        metadata,
        operations,
    })
}

fn parse_operation(fields: &BTreeMap<String, String>) -> Result<Operation, String> {
    let field = |name: &str| {
        fields
            .get(name)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("missing {name}"))
    };
    let name = field("operation")?.clone();
    let warmups = field("warmups")?.parse::<usize>().map_err(|_| "warmups")?;
    let sample_count = field("sample_count")?
        .parse::<usize>()
        .map_err(|_| "sample count")?;
    let samples = field("raw_samples_ns")?
        .split(',')
        .map(|sample| sample.parse::<u128>().map_err(|_| "raw sample".into()))
        .collect::<Result<Vec<_>, String>>()?;
    for statistic in [
        "p50_ns",
        "p95_ns",
        "max_ns",
        "write_transactions",
        "write_sql_statements",
        "write_deleted_rows",
        "write_inserted_rows",
    ] {
        field(statistic)?;
    }
    if warmups != WARMUPS || sample_count != SAMPLES || samples.len() != SAMPLES {
        return Err("wrong sample count".into());
    }
    Ok(Operation {
        name,
        warmups,
        samples,
        write_stats: None,
    })
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    sorted[((sorted.len() * percentile).div_ceil(100)).saturating_sub(1)]
}

fn dense_snapshot() -> Snapshot {
    let tabs = (0..500)
        .map(|index| Tab {
            id: 100 + index,
            title: format!("Dense tab {index}"),
            url: format!("https://dense.test/{index}"),
            pinned: index < 2,
        })
        .collect::<Vec<_>>();
    Snapshot {
        profiles: vec![Profile {
            id: 1,
            title: "Dense".into(),
            hue: ProfileHue::Slate,
            spaces: vec![Space {
                id: 10,
                title: "500 tabs".into(),
                tint: SpaceTint::Slate,
                glyph: SpaceGlyph::Glyph(GlyphName::Star),
                active_tab: Some(100),
                split: Some(Split {
                    primary_tab: 100,
                    secondary_tab: 101,
                }),
                tabs,
                recently_closed: vec![RecentlyClosedTab {
                    tab_id: 900,
                    title: "Closed".into(),
                    url: "https://closed.test/".into(),
                    pinned: false,
                    former_position: 499,
                }],
            }],
        }],
        windows: vec![WindowState {
            id: 50,
            geometry: Some(WindowGeometry {
                x: -100.0,
                y: 20.0,
                width: 1080.0,
                height: 700.0,
            }),
            sidebar: SidebarState::default(),
        }],
        settings: SearchSettings {
            engine: SearchEngineId::DuckDuckGo,
            custom_template: None,
        },
        downloads: vec![DownloadRecord {
            id: 700,
            outcome: DownloadOutcome::Succeeded,
            size: 1024,
            destination: Some("/tmp/dense.bin".into()),
            terminal_at: 900,
        }],
        command_history: vec![CommandHistoryEntry {
            id: 800,
            title: "Dense".into(),
            url: "https://dense.test/".into(),
            visit_count: 1,
            last_visited: 800,
        }],
    }
}
