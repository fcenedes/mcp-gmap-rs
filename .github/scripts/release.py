"""Build-time release checks and packaging; not required by the executable."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]
NAME = PACKAGE["name"]
VERSION = PACKAGE["version"]
TARGETS = (
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
)


def verify_tag(tag):
    if tag != f"v{VERSION}":
        raise SystemExit(f"release tag must be v{VERSION}, received {tag!r}")
    print(f"Verified {tag} matches Cargo.toml")


def archive_name(target):
    extension = "zip" if "windows" in target else "tar.gz"
    return f"{NAME}-v{VERSION}-{target}.{extension}"


def package(target, output):
    binary_name = NAME + (".exe" if "windows" in target else "")
    binary = ROOT / "target" / target / "release" / binary_name
    env = {k: v for k, v in os.environ.items() if not k.startswith("GOOGLE_MAPS_")}
    env.pop("MCP_HTTP_TOKEN", None)
    env.pop("RUST_LOG", None)
    version = subprocess.check_output([binary, "--version"], env=env, text=True, timeout=30)
    if version.strip() != f"{NAME} {VERSION}":
        raise SystemExit("executable version does not match Cargo.toml")
    tools = json.loads(subprocess.check_output([binary, "tools"], env=env, timeout=30))
    if not tools or not any(tool.get("name") == "maps_geocode" for tool in tools):
        raise SystemExit("release executable failed tool-discovery smoke test")
    output.mkdir(parents=True, exist_ok=True)
    destination = output / archive_name(target)
    with tempfile.TemporaryDirectory() as directory:
        folder = Path(directory) / f"{NAME}-v{VERSION}-{target}"
        folder.mkdir()
        shutil.copy2(binary, folder / binary_name)
        for filename in ["README.md", "LICENSE", "CHANGELOG.md", ".env.example"]:
            shutil.copy2(ROOT / filename, folder / filename)
        for directory_name in ["licenses", "docs"]:
            shutil.copytree(ROOT / directory_name, folder / directory_name)
        if "windows" in target:
            with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as archive:
                for path in sorted(folder.rglob("*")):
                    if path.is_file():
                        archive.write(path, path.relative_to(folder.parent))
        else:
            with tarfile.open(destination, "w:gz") as archive:
                archive.add(folder, arcname=folder.name)
    digest = hashlib.sha256(destination.read_bytes()).hexdigest()
    destination.with_name(destination.name + ".sha256").write_text(
        f"{digest}  {destination.name}\n", encoding="utf-8"
    )
    print(f"Packaged {destination.name}; executable version and discovery passed")


def audit_package():
    path = ROOT / "target" / "package" / f"{NAME}-{VERSION}.crate"
    allowed_files = {
        "Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "README.md", "LICENSE",
        "CHANGELOG.md", ".env.example", ".cargo_vcs_info.json",
    }
    required = {"Cargo.lock", "LICENSE", "licenses/cablate-mcp-google-map-MIT.txt", ".env.example"}
    seen = set()
    with tarfile.open(path, "r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile():
                raise SystemExit(f"unexpected non-file package entry: {member.name}")
            name = member.name.removeprefix(f"{NAME}-{VERSION}/")
            if name not in allowed_files and not name.startswith(("src/", "tests/", "docs/", "licenses/")):
                raise SystemExit(f"unexpected package file: {name}")
            if any(part == ".env" or (part.startswith(".env.") and part != ".env.example") for part in Path(name).parts):
                raise SystemExit("credential file found in crate")
            if name == ".env.example":
                content = archive.extractfile(member).read()
                if b"GOOGLE_MAPS_API_KEY=your-google-api-key" not in content:
                    raise SystemExit(".env.example must retain its placeholder key")
            seen.add(name)
    if not required <= seen:
        raise SystemExit(f"missing required package files: {sorted(required - seen)}")
    print(f"Audited {len(seen)} crate files: licenses and example included; local credentials excluded")


def checksums(output):
    expected = {archive_name(target) for target in TARGETS}
    checks = sorted(output.glob("*.sha256"))
    if {path.name.removesuffix(".sha256") for path in checks} != expected:
        raise SystemExit("release must contain checksums for exactly the four expected archives")
    lines = []
    for path in checks:
        name = path.name.removesuffix(".sha256")
        digest = hashlib.sha256((output / name).read_bytes()).hexdigest()
        expected_line = f"{digest}  {name}"
        if path.read_text(encoding="utf-8").strip() != expected_line:
            raise SystemExit(f"checksum mismatch for {name}")
        lines.append(expected_line)
    (output / "SHA256SUMS").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("Verified all four archive checksums and generated SHA256SUMS")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    verify = commands.add_parser("verify-tag")
    verify.add_argument("tag")
    build = commands.add_parser("package")
    build.add_argument("--target", choices=TARGETS, required=True)
    build.add_argument("--output", type=Path, default=ROOT / "dist")
    commands.add_parser("audit-package")
    sums = commands.add_parser("checksums")
    sums.add_argument("--output", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    if args.command == "verify-tag":
        verify_tag(args.tag)
    elif args.command == "package":
        package(args.target, args.output)
    elif args.command == "audit-package":
        audit_package()
    else:
        checksums(args.output)


if __name__ == "__main__":
    main()
