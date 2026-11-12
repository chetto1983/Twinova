#!/usr/bin/env bash
# The pre-push cargo gate (lefthook.yml): tests the workspace crates the pushed files belong to and the crates that
# depend on them, not the whole workspace. Lefthook passes the pushed files ({push_files}), relative to the repo root.
#
# Crates depend on each other in a chain: project <- physics <- engine <- {agent, twinova (the app)}.
# On Windows the app is left out: its test binaries link Tauri and do not start there, so CI tests it on Linux and
# tests only the other crates on windows-2022 (CLAUDE.md, "Post-edit validation"; ci.yml, project-windows).
set -euo pipefail

all="project physics engine agent app"
downstream() {
  case "$1" in
    project) echo "$all" ;;
    physics) echo "physics engine agent app" ;;
    engine) echo "engine agent app" ;;
    agent) echo "agent" ;;
    app) echo "app" ;;
  esac
}

touched=""
for file in "$@"; do
  case "$file" in
    app/src-tauri/.cargo/*) ;;
    app/src-tauri/Cargo.lock | app/src-tauri/Cargo.toml | app/src-tauri/rust-toolchain.toml) touched="$touched $all" ;;
    app/src-tauri/project/*) touched="$touched $(downstream project)" ;;
    app/src-tauri/physics/*) touched="$touched $(downstream physics)" ;;
    app/src-tauri/engine/*) touched="$touched $(downstream engine)" ;;
    app/src-tauri/agent/*) touched="$touched $(downstream agent)" ;;
    app/src-tauri/*) touched="$touched $(downstream app)" ;;
  esac
done

windows=false
case "$(uname -s)" in MINGW* | MSYS* | CYGWIN*) windows=true ;; esac

packages=()
for crate in $all; do
  case " $touched " in *" $crate "*) ;; *) continue ;; esac
  if [ "$crate" = app ]; then
    if [ "$windows" = true ]; then
      echo "test-pushed-crates: the app's tests do not start on Windows; CI runs them on Linux"
      continue
    fi
    packages+=(-p twinova)
  else
    packages+=(-p "twinova-$crate")
  fi
done

if [ ${#packages[@]} -eq 0 ]; then
  echo "test-pushed-crates: no crate to test"
  exit 0
fi
echo "test-pushed-crates: cargo test ${packages[*]}"
cd app/src-tauri
cargo test "${packages[@]}"
