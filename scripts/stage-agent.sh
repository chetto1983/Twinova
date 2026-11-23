#!/usr/bin/env bash
# Builds twinova-agent and puts it where app/src-tauri/tauri.agent.conf.json's externalBin looks for it: Tauri wants
# `<name>-<target triple>[.exe]` and installs it beside the app without the triple (tauri-utils 2.10.2,
# config.rs `external_bin`; spec 2026-11-23-app-mcp-design.md, decision 6). Prints the file it wrote.
#
#   bash scripts/stage-agent.sh [release|debug]
set -euo pipefail
profile=${1:-release}
cd "$(dirname "$0")/../app/src-tauri"
triple=$(rustc -vV | tr -d '\r' | sed -n 's/^host: //p')
ext=""
if [[ $triple == *windows* ]]; then ext=.exe; fi
if [[ $profile == release ]]; then cargo build --release -p twinova-agent; else cargo build -p twinova-agent; fi
mkdir -p binaries
cp "target/$profile/twinova-agent$ext" "binaries/twinova-agent-$triple$ext"
echo "app/src-tauri/binaries/twinova-agent-$triple$ext"
