#!/usr/bin/env python3
"""Copy a source snapshot without nested Git metadata or local generated data."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, help="New directory outside this workspace")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    destination = args.destination.resolve()
    if destination.exists() or destination == root or root in destination.parents:
        parser.error("destination must not exist and must be outside the source directory")

    # A temporary repository lets Git interpret all .gitignore files without
    # touching the source worktrees or accidentally exporting them as gitlinks.
    with tempfile.TemporaryDirectory(prefix="ecdsa-export-") as tmp:
        stage = Path(tmp)
        subprocess.run(["git", "init", "--quiet", str(stage)], check=True)
        paths = []
        def collect(directory):
            for path in sorted(directory.iterdir()):
                if path.name == ".git" or path.is_symlink():
                    continue
                relative = path.relative_to(root)
                if path.is_dir():
                    if path.name not in {"target", "__pycache__", ".ipynb_checkpoints", ".venv", "github-export"}:
                        collect(path)
                else:
                    paths.append(relative)
        collect(root)
        # Copy ignore files first so nested rules are honored as well.
        for path in paths:
            if path.name == ".gitignore":
                (stage / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(root / path, stage / path)
        result = subprocess.run(
            ["git", "-C", str(stage), "check-ignore", "--no-index", "-z", "--stdin"],
            input=b"".join(str(p).encode() + b"\0" for p in paths), capture_output=True,
        )
        if result.returncode not in (0, 1):
            raise RuntimeError(result.stderr.decode())
        ignored = set(result.stdout.decode().split("\0"))
        selected = [p for p in paths if str(p) not in ignored]
        for path in selected:
            source = root / path
            if source.stat().st_size >= 50 * 1024 * 1024:
                raise RuntimeError(f"Unexpected large file; review exclusions: {path}")
            with source.open("rb") as stream:
                if stream.read(4) == b"\x7fELF":
                    raise RuntimeError(f"Unexpected executable; review exclusions: {path}")
        destination.mkdir(parents=True)
        for path in selected:
            (destination / path).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(root / path, destination / path)
        size = sum((destination / p).stat().st_size for p in selected)
        print(f"Exported {len(selected)} files ({size / 1024**2:.1f} MiB) to {destination}")


if __name__ == "__main__":
    main()
