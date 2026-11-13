# Source Export

## Preparing a GitHub repository

The original workspace contains nested Git repositories/worktree pointers and large local datasets. Export it before initializing the public repository so that every snapshot is included as ordinary source files:

```bash
# Run from ecdsa-research/. The destination must be new and outside this directory.
python scripts/export_github.py ../ecdsa-research-github
cd ../ecdsa-research-github
git init -b main
git add .
git status --short
# After reviewing the staged files:
git commit -m "Initial research source snapshot"
# Replace YOUR_ACCOUNT with the destination account; create the empty repo first.
git remote add origin https://github.com/YOUR_ACCOUNT/ecdsa-research.git
git push -u origin main
```

The exporter preserves source files, research notes, summary tables, figures, and attribution. It honors the root and nested `.gitignore` files, omits all nested `.git` metadata and symlinks, and rejects unexpected large files or ELF executables. It leaves the original workspace untouched and does not create a remote or push anything.

Build caches, logs, operation streams (`*.bin`), replay packs (`*.pack`), input corpora (`*.inputs`), failure masks, GPU state dumps, and local configuration are excluded. Historical notes may reference these omitted files or paths from the original machine. Local research data remains in the original workspace.

