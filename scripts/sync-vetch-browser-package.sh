#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
requested_vetch_dir="${1:-}"

if [[ -n "$requested_vetch_dir" ]]; then
  vetch_dir="$(cd "$requested_vetch_dir" && pwd)"
else
  common_dir="$(git -C "$repo_root" rev-parse --path-format=absolute --git-common-dir)"
  primary_checkout="$(dirname "$common_dir")"
  vetch_dir="$(dirname "$primary_checkout")/vetch-app"
fi

quiet_surface="$vetch_dir/apps/quiet-surface"
destination="$quiet_surface/vendor/vicia-browser"
verify_only="${VICIA_SYNC_VERIFY_ONLY:-0}"
allow_dirty_publish="${VICIA_SYNC_ALLOW_DIRTY_PUBLISH:-0}"
# Publish the staged package to npm instead of swapping it into Vetch's vendor
# directory. Vetch keeps consuming its local `link:` build either way — the npm
# release is for outside consumers, not for Vetch.
npm_publish="${VICIA_NPM_PUBLISH:-0}"
npm_publish_dry_run="${VICIA_NPM_PUBLISH_DRY_RUN:-0}"
# Run every gate, then leave the verified package on disk instead of publishing
# it. This exists because npm one-time passwords expire in about 30 seconds
# while the gate chain takes minutes — a code entered at the start of the run is
# always dead by the time `npm publish` is reached. Staging splits the two so
# the OTP is typed against a publish that starts immediately.
npm_keep_stage="${VICIA_NPM_KEEP_STAGE:-0}"

release_mode=0
release_flag=""
if [[ "$npm_publish" == 1 ]]; then
  release_mode=1
  release_flag="VICIA_NPM_PUBLISH"
fi
if [[ "$npm_keep_stage" == 1 ]]; then
  if [[ "$release_mode" == 1 ]]; then
    echo "error: VICIA_NPM_PUBLISH cannot be combined with VICIA_NPM_KEEP_STAGE" >&2
    exit 1
  fi
  release_mode=1
  release_flag="VICIA_NPM_KEEP_STAGE"
fi

if [[ "$release_mode" == 1 ]]; then
  # A published artifact has to be reproducible from a commit. There is no
  # allow-dirty escape hatch here, unlike the Vetch-local sync. Staging is held
  # to the same rule: a staged package exists only to be published.
  if [[ "$allow_dirty_publish" == 1 ]]; then
    echo "error: VICIA_SYNC_ALLOW_DIRTY_PUBLISH cannot be combined with $release_flag" >&2
    exit 1
  fi
  if [[ "$verify_only" == 1 ]]; then
    echo "error: VICIA_SYNC_VERIFY_ONLY cannot be combined with $release_flag" >&2
    exit 1
  fi
  command -v npm >/dev/null || {
    echo "error: npm is required to publish" >&2
    exit 1
  }
fi

workspace_sha256() {
  node - "$repo_root" <<'NODE'
const { execFileSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const repo = process.argv[2];
const git = (...args) => execFileSync("git", ["-C", repo, ...args]);
const hash = createHash("sha256");
hash.update(git("rev-parse", "HEAD"));
hash.update(git("diff", "--binary", "--no-ext-diff", "--no-textconv", "HEAD", "--"));

const untracked = git("ls-files", "--others", "--exclude-standard", "-z")
  .toString("utf8")
  .split("\0")
  .filter(Boolean)
  .sort();
for (const relative of untracked) {
  const absolute = path.join(repo, relative);
  hash.update(relative);
  hash.update("\0");
  const stat = fs.lstatSync(absolute);
  hash.update(stat.isSymbolicLink() ? fs.readlinkSync(absolute) : fs.readFileSync(absolute));
  hash.update("\0");
}
process.stdout.write(hash.digest("hex"));
NODE
}

if [[ ! -f "$quiet_surface/package.json" ]]; then
  echo "error: Vetch quiet-surface package not found at $quiet_surface" >&2
  echo "usage: just sync [VETCH_APP_DIR]" >&2
  exit 1
fi

if ! grep -q '"@vicia-db/browser"' "$quiet_surface/package.json"; then
  echo "error: $quiet_surface/package.json does not declare @vicia-db/browser" >&2
  exit 1
fi

command -v wasm-pack >/dev/null || {
  echo "error: wasm-pack is required" >&2
  exit 1
}
command -v pnpm >/dev/null || {
  echo "error: pnpm is required" >&2
  exit 1
}

source_commit="$(git -C "$repo_root" rev-parse HEAD)"
source_dirty=false
if [[ -n "$(git -C "$repo_root" status --porcelain --untracked-files=normal)" ]]; then
  source_dirty=true
fi
if [[ "$source_dirty" == true && "$verify_only" != 1 && "$allow_dirty_publish" != 1 ]]; then
  echo "error: refusing to publish a browser package from a dirty Vicia checkout" >&2
  if [[ "$release_mode" == 1 ]]; then
    echo "hint: an npm release must be reproducible from a commit; commit or stash first" >&2
  else
    echo "hint: use 'just sync-local' to verify and publish the current local worktree" >&2
  fi
  exit 1
fi
source_workspace_sha256="$(workspace_sha256)"
source_kind="$([[ "$source_dirty" == true ]] && echo local-worktree || echo clean-commit)"

stage_root="$(mktemp -d "${TMPDIR:-/tmp}/vicia-browser-sync.XXXXXX")"
trap 'rm -rf "$stage_root"' EXIT
stage_package="$stage_root/package"

wasm-pack build \
  --target web \
  --scope vicia-db \
  --out-name vicia_db \
  --out-dir "$stage_package" \
  "$repo_root/bindings/browser" \
  --features browser

for required in package.json vicia_db.js vicia_db.d.ts vicia_db_bg.wasm; do
  if [[ ! -f "$stage_package/$required" ]]; then
    echo "error: wasm-pack did not produce $required" >&2
    exit 1
  fi
done

# wasm-pack ignores its own output by default. The Vetch-local package is an
# intentional, reviewable build artifact, so remove that generated ignore file.
rm -f "$stage_package/.gitignore"
cp "$repo_root/LICENSE-MIT" "$stage_package/LICENSE-MIT"
cp "$repo_root/LICENSE-APACHE" "$stage_package/LICENSE-APACHE"

wasm_sha256="$(sha256sum "$stage_package/vicia_db_bg.wasm" | awk '{print $1}')"
wasm_pack_version="$(wasm-pack --version | awk '{print $2}')"

SOURCE_COMMIT="$source_commit" \
SOURCE_DIRTY="$source_dirty" \
SOURCE_KIND="$source_kind" \
SOURCE_WORKSPACE_SHA256="$source_workspace_sha256" \
WASM_SHA256="$wasm_sha256" \
WASM_PACK_VERSION="$wasm_pack_version" \
node - "$stage_package" <<'NODE'
const fs = require("node:fs");
const path = require("node:path");

const packageDir = process.argv[2];
const packagePath = path.join(packageDir, "package.json");
const pkg = JSON.parse(fs.readFileSync(packagePath, "utf8"));
pkg.name = "@vicia-db/browser";
pkg.description = "Vicia DB browser WebAssembly package";
pkg.repository = {
  type: "git",
  // `git+https://`, not `https://`. npm normalizes the bare form on publish,
  // which rewrites package.json inside the tarball and makes the published
  // shasum differ from the staged one. Everything downstream of this script
  // verifies by hash, so the artifact has to leave here already normalized.
  url: "git+https://github.com/etc-sw/vicia-db.git",
};
pkg.files = [
  ...new Set([
    ...(pkg.files ?? []),
    "vicia-build.json",
    "vicia-integration.json",
    "LICENSE-MIT",
    "LICENSE-APACHE",
  ]),
];
fs.writeFileSync(packagePath, `${JSON.stringify(pkg, null, 2)}\n`);

const provenance = {
  package: pkg.name,
  version: pkg.version,
  sourceCommit: process.env.SOURCE_COMMIT,
  sourceDirty: process.env.SOURCE_DIRTY === "true",
  sourceKind: process.env.SOURCE_KIND,
  sourceWorkspaceSha256: process.env.SOURCE_WORKSPACE_SHA256,
  wasmSha256: process.env.WASM_SHA256,
  wasmPackVersion: process.env.WASM_PACK_VERSION,
};
fs.writeFileSync(
  path.join(packageDir, "vicia-build.json"),
  `${JSON.stringify(provenance, null, 2)}\n`,
);
NODE

if [[ "$release_mode" == 1 ]]; then
  # npm silently rewrites fields it considers malformed at publish time, which
  # changes package.json inside the tarball and breaks the equality between the
  # package we gated and the package that ships. Fail instead of shipping a
  # tarball we did not verify: if npm still has a correction to make, the
  # generator above is what needs updating.
  cp "$stage_package/package.json" "$stage_root/package.json.staged"
  npm pkg fix --prefix "$stage_package" >/dev/null 2>&1 || true
  if ! diff -u "$stage_root/package.json.staged" "$stage_package/package.json"; then
    echo "error: npm would rewrite package.json at publish time (diff above)" >&2
    echo "hint: apply the same normalization in this script's package.json generator" >&2
    exit 1
  fi
fi

latest_receipt="$repo_root/target/vetch-integration/latest.json"
mkdir -p "$(dirname "$latest_receipt")"
receipt="$latest_receipt"
SOURCE_COMMIT="$source_commit" \
SOURCE_DIRTY="$source_dirty" \
SOURCE_KIND="$source_kind" \
SOURCE_WORKSPACE_SHA256="$source_workspace_sha256" \
WASM_SHA256="$wasm_sha256" \
WASM_PACK_VERSION="$wasm_pack_version" \
  "$repo_root/scripts/verify-vetch-browser-package.sh" "$stage_package" "$vetch_dir" "$receipt"
cp "$receipt" "$stage_package/vicia-integration.json"

if [[ "$(workspace_sha256)" != "$source_workspace_sha256" ]]; then
  node - "$receipt" <<'NODE'
const fs = require("node:fs");
const receiptPath = process.argv[2];
const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
receipt.status = "failed";
receipt.checks.push({ name: "source-stability", status: "failed", durationMs: 0 });
fs.writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
NODE
  echo "error: Vicia source changed while integration gates were running; publish aborted" >&2
  exit 1
fi

if [[ "$npm_keep_stage" == 1 ]]; then
  staged="$repo_root/target/npm-package"
  rm -rf "$staged"
  mkdir -p "$(dirname "$staged")"
  mv "$stage_package" "$staged"
  echo "staged @vicia-db/browser from $source_commit — nothing published"
  echo "source kind: $source_kind"
  echo "workspace sha256: $source_workspace_sha256"
  echo "wasm sha256: $wasm_sha256"
  echo "receipt: $latest_receipt"
  echo
  echo "every gate passed. publish it with a FRESH one-time password:"
  echo
  echo "  cd $staged && npm publish --access public --otp=<6 digits>"
  echo
  echo "the staged package is bound to commit $source_commit; rebuild it if the"
  echo "working tree moves before you publish"
  exit 0
fi

if [[ "$npm_publish" == 1 ]]; then
  npm_args=(publish --access public)
  if [[ "$npm_publish_dry_run" == 1 ]]; then
    npm_args+=(--dry-run)
  fi
  if ! (cd "$stage_package" && npm "${npm_args[@]}"); then
    echo "error: npm publish failed; nothing was released and Vetch's vendored package is untouched" >&2
    exit 1
  fi
  if [[ "$npm_publish_dry_run" == 1 ]]; then
    echo "npm publish --dry-run passed for @vicia-db/browser from $source_commit"
  else
    echo "published @vicia-db/browser from $source_commit"
  fi
  echo "source kind: $source_kind"
  echo "workspace sha256: $source_workspace_sha256"
  echo "wasm sha256: $wasm_sha256"
  echo "receipt: $latest_receipt"
  echo "note: Vetch still consumes its vendored build; run 'just sync' to refresh it"
  exit 0
fi

if [[ "$verify_only" == 1 ]]; then
  echo "verified @vicia-db/browser candidate from $source_commit (publish skipped)"
  echo "receipt: $latest_receipt"
  exit 0
fi

mkdir -p "$(dirname "$destination")"
next_destination="${destination}.next.$$"
backup_destination="${destination}.previous.$$"
rm -rf "$next_destination" "$backup_destination"
mv "$stage_package" "$next_destination"

restore_previous() {
  rm -rf "$destination"
  if [[ -d "$backup_destination" ]]; then
    mv "$backup_destination" "$destination"
  fi
  pnpm --dir "$quiet_surface" install --prefer-offline >/dev/null 2>&1 || true
}

if [[ -d "$destination" ]]; then
  mv "$destination" "$backup_destination"
fi
if ! mv "$next_destination" "$destination"; then
  restore_previous
  echo "error: failed to publish candidate package; previous package restored" >&2
  exit 1
fi

if ! pnpm --dir "$quiet_surface" install --prefer-offline; then
  restore_previous
  echo "error: pnpm install failed after publish; previous package restored" >&2
  exit 1
fi

resolved_package="$quiet_surface/node_modules/@vicia-db/browser/package.json"
if [[ ! -f "$resolved_package" ]]; then
  restore_previous
  echo "error: pnpm did not link @vicia-db/browser; previous package restored" >&2
  exit 1
fi

resolved_build="$quiet_surface/node_modules/@vicia-db/browser/vicia-build.json"
resolved_commit="$(node -p "require(process.argv[1]).sourceCommit" "$resolved_build")"
resolved_workspace_sha="$(node -p "require(process.argv[1]).sourceWorkspaceSha256" "$resolved_build")"
resolved_sha="$(sha256sum "$quiet_surface/node_modules/@vicia-db/browser/vicia_db_bg.wasm" | awk '{print $1}')"
if [[ "$resolved_commit" != "$source_commit" || "$resolved_workspace_sha" != "$source_workspace_sha256" || "$resolved_sha" != "$wasm_sha256" ]]; then
  restore_previous
  echo "error: installed package provenance does not match candidate; previous package restored" >&2
  exit 1
fi

rm -rf "$backup_destination"

echo "synced @vicia-db/browser from $source_commit"
echo "source kind: $source_kind"
echo "workspace sha256: $source_workspace_sha256"
echo "wasm sha256: $wasm_sha256"
echo "destination: $destination"
