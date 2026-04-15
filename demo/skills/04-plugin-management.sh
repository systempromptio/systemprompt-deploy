#!/bin/bash
# DEMO: PLUGIN MANAGEMENT — PLUGINS, HOOKS, EXTENSIONS
# Read-only plugin and extension inspection.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: PLUGINS & HOOKS" "Plugin inspection, hook management"

subheader "STEP 1: List Database-Synced Plugins"
run_cli_head 30 core plugins list

subheader "STEP 2: Nested plugin directories on disk"
PLUGINS_DIR="$PROJECT_DIR/services/plugins"
echo "  \$ ls $PLUGINS_DIR/*/config.yaml"
echo ""
for cfg in "$PLUGINS_DIR"/*/config.yaml; do
  [[ -f "$cfg" ]] || continue
  echo "$cfg" | sed "s|$PROJECT_DIR/||" | sed 's/^/    /'
done
echo ""

subheader "STEP 3: Show enterprise-demo plugin"
run_cli_head 40 core plugins show enterprise-demo

subheader "STEP 4: List Hooks"
run_cli_head 20 core hooks list

subheader "STEP 5: Validate Hooks"
run_cli_indented core hooks validate

subheader "STEP 6: Extensions"
run_cli_head 20 plugins list

subheader "STEP 7: Extension Capabilities"
run_cli_indented plugins capabilities

header "PLUGIN MANAGEMENT DEMO COMPLETE"
