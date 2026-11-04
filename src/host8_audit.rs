//! Read-only evaluation of the canonical 300 GB Host8 plane.
//! Codec adapted from D:/host8-rooms/src/main.rs; source hash in receipts/X3-SOURCES.hbp.
//! Probe width is not an assertion about a hotel allocator's ownership binding.

use crate::{
    conserved_invariant81, prism81, shell_total81, unprism81, OmegaOmnibitRoom, ShellModel81, K, P,
};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub const SECTORS: u64 = 375;
pub const ROOMS_PER_SECTOR: u64 = 100_000_000;
pub const TOTAL_ROOMS: u64 = SECTORS * ROOMS_PER_SECTOR;
pub const SEED: u64 = 0x518b_bb47_2a9d_b786;
pub const H27: u64 = 3_812_798_742_493;
const M1: u64 = 0xbf58_476d_1ce4_e5b9;
const M2: u64 = 0x94d0_49bb_1331_11eb;

const fn modinv(c: u64) -> u64 {
    let mut x = c;
    let mut i = 0;
    while i < 6 {
        x = x.wrapping_mul(2u64.wrapping_sub(c.wrapping_mul(x)));
        i += 1;
    }
    x
}

pub fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(M1);
    z = (z ^ (z >> 27)).wrapping_mul(M2);
    z ^ (z >> 31)
}

pub fn unmix(mut z: u64) -> u64 {
    z ^= (z >> 31) ^ (z >> 62);
    z = z.wrapping_mul(modinv(M2));
    z ^= (z >> 27) ^ (z >> 54);
    z = z.wrapping_mul(modinv(M1));
    z ^= (z >> 30) ^ (z >> 60);
    z
}

pub fn word(n: u64) -> Result<u64, String> {
    if n >= TOTAL_ROOMS {
        return Err("room outside canonical 300G plane".into());
    }
    Ok((i128::from(n) - i128::from(TOTAL_ROOMS / 2) + i128::from(H27)) as u64)
}

pub fn identity(n: u64, stub: u64) -> Result<(), String> {
    let expected = word(n)?;
    if unmix(stub) ^ SEED != expected || mix(expected ^ SEED) != stub {
        return Err(format!("room identity mismatch at n={n}"));
    }
    Ok(())
}

/// Independent signed-integer oracle: no call to the production transfer or step.
pub fn reference_step(u: &[u64; K], nu: u64) -> [u64; K] {
    let p = i128::from(P);
    let mut nl = [0i128; K];
    for n in 1..K - 1 {
        nl[n] = (i128::from(u[n]) * (i128::from(u[n + 1]) - i128::from(u[n - 1]))).rem_euclid(p);
    }
    let s = nl[1..K - 1].iter().sum::<i128>().rem_euclid(p);
    nl[0] = -(s / 2);
    nl[K - 1] = -s + s / 2;
    let mut out = [0u64; K];
    let mut weight = 1i128;
    for n in 0..K {
        out[n] = (i128::from(u[n]) + nl[n] - i128::from(nu % P) * weight).rem_euclid(p) as u64;
        weight = weight * 9 % p;
    }
    out
}

/// A complete check bundle; all coordinates and both inverse transforms are checked.
pub fn check_state(u: [u64; K], nu: u64, steps: u64) -> Result<(), String> {
    if u.iter().any(|&x| x >= P) || steps == 0 {
        return Err("invalid state or zero-step audit".into());
    }
    let mut model = ShellModel81 { u, nu };
    let q0 = shell_total81(&u);
    let spectrum = prism81(&u);
    if spectrum[0] != q0 || unprism81(&spectrum) != u {
        return Err("state NTT inverse mismatch".into());
    }
    let nl = model.non_linear_transfer();
    let spec = prism81(&nl);
    if shell_total81(&nl) != 0 || spec[0] != 0 || unprism81(&spec) != nl {
        return Err("transfer NTT mismatch".into());
    }
    for t in 1..=steps {
        let expected = reference_step(&model.u, nu);
        model.step();
        if model.u != expected
            || model.u.iter().any(|&x| x >= P)
            || conserved_invariant81(&model.u, nu, t) != q0
        {
            return Err(format!("transition or compensated-total mismatch t={t}"));
        }
    }
    Ok(())
}

pub fn read_window(file: &mut File, local_start: u64, count: usize) -> Result<Vec<u64>, String> {
    let end = local_start
        .checked_add(count as u64)
        .ok_or("window overflow")?;
    if count == 0 || end > ROOMS_PER_SECTOR {
        return Err("invalid sector window".into());
    }
    file.seek(SeekFrom::Start(local_start * 8))
        .map_err(|e| e.to_string())?;
    let mut bytes = vec![0u8; count * 8];
    file.read_exact(&mut bytes)
        .map_err(|e| format!("short/unreadable room window: {e}"))?;
    Ok(bytes
        .chunks_exact(8)
        .map(|b| u64::from_le_bytes(b.try_into().expect("eight byte chunk")))
        .collect())
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    pub sectors: u64,
    pub windows: u64,
    pub rooms: u64,
    pub ensembles: u64,
    pub transitions: u64,
}

impl Counts {
    pub fn merge(&mut self, rhs: Self) {
        self.sectors += rhs.sectors;
        self.windows += rhs.windows;
        self.rooms += rhs.rooms;
        self.ensembles += rhs.ensembles;
        self.transitions += rhs.transitions;
    }
}

/// Deterministic, disjoint first-to-last probes, not a generated replacement for stored rooms.
pub fn audit_sector(
    root: &Path,
    sector: u64,
    width: usize,
    windows: u64,
    steps: u64,
) -> Result<Counts, String> {
    if sector >= SECTORS
        || ![81, 243].contains(&width)
        || !(2..=81).contains(&windows)
        || !(1..=243).contains(&steps)
    {
        return Err("invalid sector/width/windows/steps".into());
    }
    let path = root.join(format!("sector-{sector:03}.rooms"));
    let mut file = File::open(&path).map_err(|e| format!("open sector {sector}: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if metadata.len() != ROOMS_PER_SECTOR * 8 {
        return Err(format!(
            "sector {sector}: wrong byte length {}",
            metadata.len()
        ));
    }
    let mut counts = Counts {
        sectors: 1,
        ..Counts::default()
    };
    for w in 0..windows {
        let offset = (ROOMS_PER_SECTOR - width as u64) * w / (windows - 1);
        let stubs = read_window(&mut file, offset, width)?;
        for (i, &stub) in stubs.iter().enumerate() {
            identity(sector * ROOMS_PER_SECTOR + offset + i as u64, stub)?;
        }
        for (i, triple) in stubs.chunks_exact(3).enumerate() {
            let room = OmegaOmnibitRoom::new(triple[0], triple[1], triple[2]);
            if OmegaOmnibitRoom::from_bytes(&room.to_bytes()) != room {
                return Err("room packing mismatch".into());
            }
            // Preserve the pinned source's state projection, salt and amplitude unchanged.
            // Viscosity edges include positive multiples of P, not just nu=1.
            let nu = [0, 1, P - 1, P, P + 1, u64::MAX][(sector as usize + w as usize + i) % 6];
            check_state(room.unpack_state(333_360), nu, steps)?;
            counts.ensembles += 1;
            counts.transitions += steps;
        }
        counts.windows += 1;
        counts.rooms += width as u64;
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn published_300g_room_identities_match() {
        for (n, stub) in [
            (0, 0x56f4ee189ba889dd),
            (99_999_999, 0x58bc58f2c1a676c4),
            (100_000_000, 0x0da343c034637307),
            (18_750_000_000, 0x259d7863c53bae36),
            (37_499_999_999, 0x4ab3ca1cbd74c442),
        ] {
            identity(n, stub).unwrap();
        }
    }
    #[test]
    fn corrupted_identity_and_out_of_plane_rejected() {
        assert!(identity(0, 0).is_err());
        assert!(word(TOTAL_ROOMS).is_err());
    }
    #[test]
    fn mixer_is_exact_both_directions() {
        for x in [0, 1, SEED, H27, u64::MAX] {
            assert_eq!(unmix(mix(x)), x);
            assert_eq!(mix(unmix(x)), x);
        }
    }
    #[test]
    fn empty_audit_not_passed() {
        assert!(check_state([0; K], 1, 0).is_err());
    }
    #[test]
    fn noncanonical_state_rejected() {
        assert!(check_state([P; K], 1, 1).is_err());
    }
    #[test]
    fn independent_step_agrees_at_viscosity_edges() {
        for nu in [0, 1, P - 1, P, P + 1, u64::MAX] {
            check_state(ShellModel81::new(P - 1, nu).u, nu, 81).unwrap();
        }
    }
}
