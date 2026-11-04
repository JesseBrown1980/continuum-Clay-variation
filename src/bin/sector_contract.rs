//! Replay the existing integer-only 3/27/243 transition lattice unchanged.
#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../vendor/lattice/oscillator.rs"]
mod oscillator;
#[rustfmt::skip]
#[allow(dead_code, clippy::enum_variant_names)]
#[path = "../../vendor/lattice/trit.rs"]
mod trit;
use continuum_clay_variation::{host8_audit::check_state, ShellModel81};
use oscillator::{advance_completed_barrier, level_transition_keys};
use std::process::ExitCode;

fn verify() -> Result<(), String> {
    let mut total = 0;
    for (level, sectors) in [(0, 3), (1, 27), (2, 243)] {
        let keys = level_transition_keys(level, sectors)?;
        for (ordinal, key) in keys.iter().enumerate() {
            if key.inverse().inverse() != *key {
                return Err("local inverse mismatch".into());
            }
            // New Navier audit callback over the old lattice. This is not a model-service call.
            let u = ShellModel81::new(
                ordinal as u64 * 37 + u64::from(key.sector),
                u64::from(level),
            )
            .u;
            check_state(u, u64::from(level), 3)?;
        }
        if advance_completed_barrier(level, sectors, keys.len() - 1).is_ok() {
            return Err("partial barrier accepted".into());
        }
        let barrier = advance_completed_barrier(level, sectors, keys.len())?;
        println!("X3LATTICE|level={level}|sectors={sectors}|local_keys={}|next_sectors={}|shell_audit_callbacks={}|model_service_calls=0|status=PASS|json=0",keys.len(),barrier.next_sectors,keys.len());
        total += keys.len();
    }
    if total != 4095 {
        return Err("lattice census mismatch".into());
    }
    println!("X3LATTICERESULT|keys={total}|new_shell_transitions={}|existing_contract_reused=1|allocation_changes=0|float_used=0|node=0|v8=0|json=0",total*3);
    Ok(())
}
fn main() -> ExitCode {
    match verify() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("X3LATTICEFAIL|reason={e}|json=0");
            ExitCode::FAILURE
        }
    }
}
