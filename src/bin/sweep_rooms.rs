//! sweep_rooms: Mass verification of Host8 and 81-Mode Omega/Omnibit Rooms
//! against 3-Adic Navier-Stokes Regularity Invariants and Continuum Scaling.
//!
//! Enforces:
//! - Rust 1.81+
//! - Clean clippy with zero warnings
//! - Strict integer-only arithmetic (clippy::float_arithmetic denied, zero float types)
//! - Balanced ternary / trinary mapping via tribit Zero
//! - 0 JSON (HBP format output only: TAG|k=v|...|json=0)
//! - 0 Node, 0 V8 (pure native Rust binary)
//! - Dual Mode: Direct disk streaming from D:/ASOLARIA-HOST8-ROOMS-100G or canonical in-memory synthesis.
//! - n x 3 x 3 x 3 Architecture: 3 rungs of 27 trits = 81 modes (3^4 hypercube).

#![deny(
    clippy::float_arithmetic,
    clippy::cast_precision_loss,
    clippy::float_cmp
)]
#![forbid(unsafe_code)]

use continuum_clay_variation::{
    conserved_invariant81, prism81, shell_total81, OmegaOmnibitRoom, ShellModel81, K, P,
};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[allow(dead_code)]
const ROOM_BYTES: u64 = 8;
const BUFFER_CAPACITY: usize = 4 * 1024 * 1024; // 4 MiB buffer per worker thread

// Host8 Canonical Invertible Mix & Trit Constants
const M1: u64 = 0xbf58_476d_1ce4_e5b9;
const M2: u64 = 0x94d0_49bb_1331_11eb;
const H27: u64 = 3_812_798_742_493; // (3^27 - 1) / 2
const SEED_HOST8: u64 = 0x8467_a937_cba3_09f7;

#[inline]
fn host8_mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(M1);
    z = (z ^ (z >> 27)).wrapping_mul(M2);
    z ^ (z >> 31)
}

#[inline]
fn host8_room_stub(room_index: u64, total_rooms: u64) -> u64 {
    let centre = total_rooms / 2;
    let balanced = if room_index >= centre {
        (room_index - centre) as i64
    } else {
        -((centre - room_index) as i64)
    };
    let word = (balanced + H27 as i64) as u64;
    host8_mix(word ^ SEED_HOST8)
}

/// Unpack an 81-trit ensemble (from 3 Host8 stubs: n x 3 x 3 x 3 where n=3) into an 81-mode state.
#[inline(always)]
fn unpack_81_modes(stub0: u64, stub1: u64, stub2: u64) -> [u64; K] {
    let room = OmegaOmnibitRoom::new(stub0, stub1, stub2);
    room.unpack_state(333_360)
}

#[inline(always)]
fn verify_81_ensemble(stub0: u64, stub1: u64, stub2: u64, nu: u64, do_step: bool) -> bool {
    let u = unpack_81_modes(stub0, stub1, stub2);
    let mut model = ShellModel81 { u, nu };
    let nl = model.non_linear_transfer();

    // Invariant 1: Sum of non-linear transfer == 0 mod P
    let mut sum_nl = 0u64;
    for &val in &nl {
        sum_nl = (sum_nl + val) % P;
    }
    if sum_nl != 0 {
        return false;
    }

    // Invariant 2: NTT-81 Spherical Prism DC Center X[0] == 0 mod P
    let spectrum = prism81(&nl);
    if spectrum[0] != 0 {
        return false;
    }

    // Invariant 3: Step & Exact Conserved Invariant I = Q(u) + t*nu*D81
    if do_step {
        let q0 = shell_total81(&model.u);
        model.step();
        let i_t = conserved_invariant81(&model.u, nu, 1);
        if i_t != q0 {
            return false;
        }
        if model.total_enstrophy() >= P {
            return false;
        }
    }

    true
}

fn print_usage() {
    println!("Usage: sweep_rooms [OPTIONS]");
    println!("Options:");
    println!("  --rooms <N>        Total 81-mode ensembles to verify (default: 1,000,000)");
    println!("  --threads <T>      Number of parallel worker threads (default: 8)");
    println!("  --sector <PATH>    Path to a specific .rooms sector file");
    println!("  --no-step          Skip time-stepping (validate triad & DC prism only)");
    println!(
        "  --full-sector      Verify full sector files (100M rooms / 33.3M 81-mode ensembles)"
    );
    println!("  --nu <NU>          Viscosity parameter (default: 1)");
    println!("  --help             Display this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut target_rooms: u64 = 1_000_000;
    let mut thread_count: usize = 8;
    let mut specific_sector: Option<String> = None;
    let mut do_step = true;
    let mut full_sector = false;
    let mut nu: u64 = 1;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--rooms" => {
                if i + 1 < args.len() {
                    target_rooms = args[i + 1].parse().unwrap_or(1_000_000);
                    i += 1;
                }
            }
            "--threads" => {
                if i + 1 < args.len() {
                    thread_count = args[i + 1].parse().unwrap_or(8);
                    i += 1;
                }
            }
            "--sector" => {
                if i + 1 < args.len() {
                    specific_sector = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--no-step" => do_step = false,
            "--full-sector" => full_sector = true,
            "--nu" => {
                if i + 1 < args.len() {
                    nu = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    if full_sector {
        target_rooms = 33_333_333; // 100M Host8 rooms / 3 rooms per 81-mode ensemble
    }

    println!("HEADER|suite=continuum_clay_variation|target=host8_81_modes|runtime=rust_1.81|arithmetic=integer_only|ternary=balanced_trit_k81|arch=nx3x3x3|json=0");
    println!(
        "CONFIG|target_ensembles={target_rooms}|threads={thread_count}|do_step={do_step}|nu={nu}|json=0"
    );

    let default_sector_path = "D:/ASOLARIA-HOST8-ROOMS-100G/sector-000.rooms";
    let sector_to_open = specific_sector.as_deref().unwrap_or(default_sector_path);

    let has_disk_file = Path::new(sector_to_open).exists();
    if has_disk_file {
        println!("MODE|source=disk|path={sector_to_open}|json=0");
    } else {
        println!("MODE|source=canonical_in_memory_generator|json=0");
    }

    let start_time = Instant::now();
    let verified_counter = Arc::new(AtomicU64::new(0));
    let violation_counter = Arc::new(AtomicU64::new(0));
    let rooms_per_thread = target_rooms / (thread_count as u64);
    let remainder_rooms = target_rooms % (thread_count as u64);

    let mut handles = Vec::new();

    for t in 0..thread_count {
        let count = rooms_per_thread + if t == 0 { remainder_rooms } else { 0 };
        let offset_ensembles =
            t as u64 * rooms_per_thread + if t == 0 { 0 } else { remainder_rooms };
        let sector_path_clone = sector_to_open.to_string();
        let verified = Arc::clone(&verified_counter);
        let violations = Arc::clone(&violation_counter);

        let handle = thread::spawn(move || {
            if has_disk_file {
                let file = match File::open(&sector_path_clone) {
                    Ok(f) => f,
                    Err(_) => return,
                };
                let mut reader = BufReader::with_capacity(BUFFER_CAPACITY, file);
                // 3 Host8 rooms (24 bytes) per 81-mode ensemble
                let byte_offset = offset_ensembles * 24;
                if reader.seek(std::io::SeekFrom::Start(byte_offset)).is_err() {
                    return;
                }

                let mut buf = [0u8; 24];
                for _ in 0..count {
                    if reader.read_exact(&mut buf).is_err() {
                        break;
                    }
                    let mut b0 = [0u8; 8];
                    let mut b1 = [0u8; 8];
                    let mut b2 = [0u8; 8];
                    b0.copy_from_slice(&buf[0..8]);
                    b1.copy_from_slice(&buf[8..16]);
                    b2.copy_from_slice(&buf[16..24]);

                    let s0 = u64::from_le_bytes(b0);
                    let s1 = u64::from_le_bytes(b1);
                    let s2 = u64::from_le_bytes(b2);

                    if verify_81_ensemble(s0, s1, s2, nu, do_step) {
                        verified.fetch_add(1, Ordering::Relaxed);
                    } else {
                        violations.fetch_add(1, Ordering::Relaxed);
                    }
                }
            } else {
                // Procedural Host8 Tri-Stub Generation (n x 3 x 3 x 3 architecture)
                for i in 0..count {
                    let ensemble_idx = offset_ensembles + i;
                    let r0 = ensemble_idx * 3;
                    let r1 = r0 + 1;
                    let r2 = r0 + 2;

                    let s0 = host8_room_stub(r0, 12_500_000_000);
                    let s1 = host8_room_stub(r1, 12_500_000_000);
                    let s2 = host8_room_stub(r2, 12_500_000_000);

                    if verify_81_ensemble(s0, s1, s2, nu, do_step) {
                        verified.fetch_add(1, Ordering::Relaxed);
                    } else {
                        violations.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    let elapsed = start_time.elapsed();
    let elapsed_ms = (elapsed.as_millis()) as u64;
    let total_v = verified_counter.load(Ordering::SeqCst);
    let total_err = violation_counter.load(Ordering::SeqCst);

    let rate_rps = if elapsed_ms > 0 {
        (total_v * 1000) / elapsed_ms
    } else {
        total_v
    };

    println!(
        "RECEIPT81|ensembles={total_v}|modes_per_ensemble=81|trits_per_ensemble=81|arch=nx3x3x3|elapsed_ms={elapsed_ms}|rate_eps={rate_rps}|violations={total_err}|status={}|json=0",
        if total_err == 0 { "VERIFIED_81_INVARIANTS_PASSED" } else { "VIOLATIONS_DETECTED" }
    );
}
