"""Package a built Windows executable and generated examples; no dependencies."""
from pathlib import Path
import hashlib
import os
import tomllib
import zipfile

root = Path(__file__).resolve().parent.parent
version = tomllib.loads((root / "Cargo.toml").read_text("utf-8"))["package"]["version"]
target = Path(os.environ.get("CARGO_TARGET_DIR", root / "target"))
exe = target / "release/xor-112.exe"
if not exe.is_file():
    raise SystemExit("Build the Windows release before packaging")
for name in ["half-adder", "full-adder", "majority", "mux8", "alu32", "alu64"]:
    if not (root / f"examples/circuits/{name}.binary").is_file():
        raise SystemExit("Run cargo run --release --example export_examples first")
dist = root / "dist"
dist.mkdir(exist_ok=True)
archive = dist / f"XOR-112-v{version}-windows-x64.zip"
files = [root / name for name in ["README.md", "README.en.md", "LICENSE", "THIRD_PARTY_NOTICES.txt", "CHANGELOG.md"]]
files += sorted((root / "docs").rglob("*"))
files += sorted((root / "examples/circuits").glob("*"))
with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    z.write(exe, "XOR-112/xor-112.exe")
    for path in files:
        if path.is_file():
            z.write(path, "XOR-112/" + path.relative_to(root).as_posix())
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
(dist / "SHA256SUMS.txt").write_text(f"{digest}  {archive.name}\n", encoding="ascii")
print(f"Packaged {archive.name} ({archive.stat().st_size:,} bytes)")
