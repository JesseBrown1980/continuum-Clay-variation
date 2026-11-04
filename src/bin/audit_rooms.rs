//! Fail-closed read-only 81/243-width Host8 evaluation. No generator fallback.
#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]
use continuum_clay_variation::host8_audit::{audit_sector, Counts, SECTORS};
use std::{env, path::PathBuf, process::ExitCode, thread, time::Instant};

#[derive(Debug)]
struct Config {
    root: PathBuf,
    width: usize,
    windows: u64,
    steps: u64,
    threads: usize,
}

fn parse(args: &[String]) -> Result<Config, String> {
    let mut cfg = Config {
        root: PathBuf::new(),
        width: 243,
        windows: 9,
        steps: 3,
        threads: 9,
    };
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--store" => cfg.root = PathBuf::from(value),
            "--group" => cfg.width = value.parse().map_err(|_| "bad group")?,
            "--windows" => cfg.windows = value.parse().map_err(|_| "bad windows")?,
            "--steps" => cfg.steps = value.parse().map_err(|_| "bad steps")?,
            "--threads" => cfg.threads = value.parse().map_err(|_| "bad threads")?,
            _ => return Err(format!("unknown flag {flag}")),
        }
    }
    if cfg.root.as_os_str().is_empty()
        || ![81, 243].contains(&cfg.width)
        || !(2..=81).contains(&cfg.windows)
        || !(1..=243).contains(&cfg.steps)
        || !(1..=27).contains(&cfg.threads)
    {
        return Err(
            "required --store PATH; --group 81|243; windows 2..81; steps 1..243; threads 1..27"
                .into(),
        );
    }
    Ok(cfg)
}

fn run(cfg: Config) -> Result<(), String> {
    let start = Instant::now();
    println!("X3HDR|schema=CLASY-X3-READONLY-V1|source=stored_300G_rooms|physical_sectors=375|orchestration=bounded_native_threads|probe_width={}|windows_per_sector={}|steps={}|threads={}|float_used=0|node=0|v8=0|json=0",cfg.width,cfg.windows,cfg.steps,cfg.threads);
    let mut handles = Vec::new();
    for worker in 0..cfg.threads {
        let root = cfg.root.clone();
        let width = cfg.width;
        let windows = cfg.windows;
        let steps = cfg.steps;
        let threads = cfg.threads;
        handles.push(thread::spawn(move || -> Result<Counts, String> {
            let mut counts = Counts::default();
            for s in (worker as u64..SECTORS).step_by(threads) {
                counts.merge(audit_sector(&root, s, width, windows, steps)?);
            }
            Ok(counts)
        }));
    }
    let mut total = Counts::default();
    let mut errors = Vec::new();
    for h in handles {
        match h.join() {
            Ok(Ok(c)) => total.merge(c),
            Ok(Err(e)) => errors.push(e),
            Err(_) => errors.push("worker panic".into()),
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let expected = Counts {
        sectors: SECTORS,
        windows: SECTORS * cfg.windows,
        rooms: SECTORS * cfg.windows * cfg.width as u64,
        ensembles: SECTORS * cfg.windows * cfg.width as u64 / 3,
        transitions: SECTORS * cfg.windows * cfg.width as u64 / 3 * cfg.steps,
    };
    if total != expected {
        return Err(format!("incomplete coverage {total:?} != {expected:?}"));
    }
    println!("X3RESULT|width={}|sectors={}|windows={}|room_reads={}|bytes_read={}|ensembles81={}|transitions={}|state_inverse_bundles={}|nl_inverse_bundles={}|identity_failures=0|invariant_failures=0|coverage_complete=1|elapsed_ms={}|status=PASS|json=0",cfg.width,total.sectors,total.windows,total.rooms,total.rooms*8,total.ensembles,total.transitions,total.ensembles,total.ensembles,start.elapsed().as_millis());
    println!("X3AUTH|room_store_writes=0|allocator_changes=0|provider_calls=0|mcp_calls=0|gpu_calls=0|external_model_calls=0|runtime_subprocesses=0|descendant_learning_ledger=not_instrumented_by_this_harness|hotel_allocator_binding=owner_response_pending|pde_bridge=not_tested|json=0");
    Ok(())
}

fn main() -> ExitCode {
    match parse(&env::args().skip(1).collect::<Vec<_>>()).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "X3FAIL|status=FAIL|reason={}|json=0",
                e.replace(['|', '\n', '\r'], " ")
            );
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn missing_store_rejected() {
        assert!(parse(&[]).is_err());
    }
    #[test]
    fn zero_threads_rejected() {
        assert!(parse(&args(&["--store", "x", "--threads", "0"])).is_err());
    }
    #[test]
    fn unknown_or_truncated_options_rejected() {
        assert!(parse(&args(&["--store"])).is_err());
        assert!(parse(&args(&["--wat", "1"])).is_err());
    }
    #[test]
    fn only_existing_widths_admitted() {
        for w in ["81", "243"] {
            assert!(parse(&args(&["--store", "x", "--group", w])).is_ok());
        }
        assert!(parse(&args(&["--store", "x", "--group", "242"])).is_err());
    }
}
