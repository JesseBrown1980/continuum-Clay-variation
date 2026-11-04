//! Explicit failure-exit and short-read controls. Fixtures are not production rooms.
#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]
use continuum_clay_variation::{
    host8_audit::{read_window, ROOMS_PER_SECTOR},
    OmegaOmnibitRoom,
};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "clasy-x3-negative-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn file(&self, bytes: &[u8]) -> File {
        let p = self.0.join("sector-000.rooms");
        File::create(&p).unwrap().write_all(bytes).unwrap();
        File::open(p).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("sector-000.rooms"));
        let _ = fs::remove_dir(&self.0);
    }
}
fn rejects(root: &std::path::Path, extra: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_audit_rooms"))
        .arg("--store")
        .arg(root)
        .args(extra)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !stdout.contains("X3RESULT"),
        "failure must not emit success receipt"
    );
    assert!(stderr.contains("X3FAIL|status=FAIL"));
}
#[test]
fn missing_store_exits_nonzero_without_fallback() {
    let f = Fixture::new();
    rejects(&f.0.join("missing"), &[]);
}
#[test]
fn truncated_sector_exits_nonzero_without_success_receipt() {
    let f = Fixture::new();
    drop(f.file(&[0; 8]));
    rejects(&f.0, &[]);
}
#[test]
fn invalid_cli_exits_nonzero() {
    let f = Fixture::new();
    rejects(&f.0, &["--threads", "0"]);
    rejects(&f.0, &["--group", "242"]);
    rejects(&f.0, &["--unknown", "x"]);
}
#[test]
fn short_read_is_error_not_partial_success() {
    let f = Fixture::new();
    let mut file = f.file(&[0; 7]);
    assert!(read_window(&mut file, 0, 1).is_err());
}
#[test]
fn out_of_sector_and_empty_reads_rejected() {
    let f = Fixture::new();
    let mut file = f.file(&[0; 8]);
    assert!(read_window(&mut file, 0, 0).is_err());
    assert!(read_window(&mut file, ROOMS_PER_SECTOR, 1).is_err());
    assert!(read_window(&mut file, u64::MAX, 1).is_err());
}
#[test]
fn disk_word_little_endian_fixture() {
    let f = Fixture::new();
    let mut file = f.file(&[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(
        read_window(&mut file, 0, 1).unwrap(),
        vec![0x0807_0605_0403_0201]
    );
}
#[test]
fn room_encoding_has_fixed_little_endian_layout() {
    let room = OmegaOmnibitRoom::new(
        0x0807_0605_0403_0201,
        0x100f_0e0d_0c0b_0a09,
        0x1817_1615_1413_1211,
    );
    let bytes = std::array::from_fn(|i| i as u8 + 1);
    assert_eq!(room.to_bytes(), bytes);
    assert_eq!(OmegaOmnibitRoom::from_bytes(&bytes), room);
}
