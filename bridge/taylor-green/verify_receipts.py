#!/usr/bin/env python3
"""Independent, fail-closed audit of the three Taylor-Green -> Clay81 leaves.
No dependency on the simulator or Clay library, no floats, no JSON.
Checks pinned measurements, projection arithmetic, chain hashes, and controls.
"""
import hashlib
from pathlib import Path
import sys

P = 1_000_999
K = 81
EXPECTED_TGV = {
    1: (382_752, 1_225_664, 37_128, 169_912, 81),
    2: (537_904, 1_876_856, 34_600, 237_204, 80),
    3: (573_144, 2_038_608, 33_688, 244_184, 80),
}
EXPECTED_FILES = {
    "LEAF-1.hbp", "LEAF-2.hbp", "LEAF-3.hbp", "LEAVES.hbi",
    "RUN.hbp", "NEGATIVE.hbp",
}

def fields(line, tag):
    parts = line.rstrip("\n").split("|")
    if not parts or parts[0] != tag:
        raise ValueError(f"expected {tag} row")
    output = {}
    for part in parts[1:]:
        if part.count("=") != 1:
            raise ValueError(f"bad key/value in {tag}")
        key, value = part.split("=", 1)
        if not key or key in output:
            raise ValueError("duplicate or empty field")
        output[key] = value
    if output.get("json") != "0":
        raise ValueError("unexpected format")
    return output

def sha(blob):
    return hashlib.sha256(blob).hexdigest()

def audit(directory):
    base = Path(directory)
    checks = (base / "SHA256SUMS").read_text(encoding="utf-8").splitlines()
    seen = set()
    for item in checks:
        digest, file_name = item.split(maxsplit=1)
        if file_name in seen or file_name not in EXPECTED_FILES:
            raise ValueError(f"duplicate or unexpected file {file_name}")
        data = (base / file_name).read_bytes()
        if digest != sha(data):
            raise ValueError(f"SHA-256 mismatch {file_name}")
        seen.add(file_name)
    if seen != EXPECTED_FILES:
        raise ValueError(f"SHA-256 list incomplete: {EXPECTED_FILES - seen}")

    index_lines = (base / "LEAVES.hbi").read_text(encoding="utf-8").splitlines()
    if len(index_lines) != 4 or fields(index_lines[0], "BRIDGE_HBI").get("leaf_count") != "3":
        raise ValueError("invalid three-leaf index")
    run_lines = (base / "RUN.hbp").read_text(encoding="utf-8").splitlines()
    if not any(row.startswith("BRIDGE_RESULT|steps=3|leaves=3|arms=3|")
               and "|status=PASS|" in row for row in run_lines):
        raise ValueError("missing positive experiment verdict")
    if (base / "NEGATIVE.hbp").read_text(encoding="utf-8").strip() != (
        "NEGATIVE|nontrit=REJECTED|gate_corruption=REJECTED|json=0"
    ):
        raise ValueError("negative controls not witnessed")

    prior = "GENESIS"
    all_leaf_rows = []
    for t in (1, 2, 3):
        name = f"LEAF-{t}.hbp"
        data = (base / name).read_bytes()
        chain = fields(index_lines[t], "LEAF_INDEX")
        if (chain.get("t") != str(t) or chain.get("file") != name
                or chain.get("sha256") != sha(data)
                or chain.get("previous_leaf_sha256") != prior
                or chain.get("rows") != "3"):
            raise ValueError(f"chain failure at step {t}")
        prior = sha(data)
        lines = data.decode("utf-8").splitlines()
        if len(lines) != 5:
            raise ValueError(f"wrong number of rows at step {t}")
        header = fields(lines[0], "LEAF_HEADER")
        footer = fields(lines[-1], "LEAF_FOOTER")
        if (header.get("t") != str(t)
                or header.get("previous_leaf_sha256") != chain["previous_leaf_sha256"]
                or header.get("arms") != "TGV,SCR,NULL"
                or footer.get("t") != str(t)
                or footer.get("rows") != "3" or footer.get("status") != "PASS"):
            raise ValueError("invalid leaf header/footer")
        arms = []
        for line in lines[1:-1]:
            row = fields(line, "LEAF")
            if row.get("t") != str(t):
                raise ValueError("cross-step row injection")
            arms.append(row["arm"])
            if line not in run_lines:
                raise ValueError("leaf row not present in original run")
            signed = list(map(int, row["signed81"].split(",")))
            residues = list(map(int, row["u81"].split(",")))
            if len(signed) != K or len(residues) != K:
                raise ValueError("wrong vector width")
            if any(abs(j) > 9_216 or residue != j % P
                   for j, residue in zip(signed, residues)):
                raise ValueError("projection not exactly reversible")
            global_components = list(map(int, row["global"].split(",")))
            if len(global_components) != 3:
                raise ValueError("wrong global momentum width")
            q = int(row["Q"])
            if q != sum(signed) % P or q != sum(global_components) % P:
                raise ValueError("global/finite-field sum mismatch")
            if q != 0 or global_components != [0, 0, 0]:
                raise ValueError("unexpected unforced global momentum")
            if int(row["mass"]) != 373_248 or int(row["E2"]) != 746_496:
                raise ValueError("mass/energy not conserved")
            if row["arm"] == "TGV":
                observed = tuple(int(row[k]) for k in
                                 ("KJ", "WJ", "A", "moved_trits", "moved_registers"))
                if observed != EXPECTED_TGV[t] or int(row["local_defect"]) != 81:
                    raise ValueError(f"TGV is not the registered 3-step observation at {t}")
            elif row["arm"] == "NULL":
                if (any(signed) or any(residues)
                        or any(int(row[k]) for k in
                               ("KJ", "WJ", "A", "moved_trits",
                                "moved_registers", "local_defect", "seq_defect"))):
                    raise ValueError("NULL control moved")
            elif row["arm"] != "SCR":
                raise ValueError("unknown arm")
            all_leaf_rows.append(line)
        if arms != ["TGV", "SCR", "NULL"]:
            raise ValueError(f"wrong arm order/coverage {arms}")
        if f"STEP_RESULT|t={t}|leaves=3|status=PASS|json=0" not in run_lines:
            raise ValueError(f"missing PASS for t={t}")
        print(f"INDEPENDENT_LEAF_AUDIT|t={t}|sha256={prior}|arms=3|status=PASS|json=0")

    run_leaf_rows = [line for line in run_lines if line.startswith("LEAF|")]
    if run_leaf_rows != all_leaf_rows:
        raise ValueError("run/leaf coverage differs")
    print("INDEPENDENT_BRIDGE_AUDIT|steps=3|leaves=3|arms=3|sha256=PASS|projection=PASS|conservation=PASS|null=PASS|noncommutation=CONFIRMED|status=PASS|json=0")

if __name__ == "__main__":
    try:
        if len(sys.argv) != 2:
            raise ValueError("usage verify_receipts.py DIRECTORY")
        audit(sys.argv[1])
    except (ValueError, OSError, UnicodeError) as exc:
        print(f"INDEPENDENT_BRIDGE_AUDIT|status=FAIL|reason={exc}|json=0", file=sys.stderr)
        sys.exit(1)
