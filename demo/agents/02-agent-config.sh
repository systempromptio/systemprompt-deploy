#!/bin/bash
# AGENTS: CONFIGURATION — Validation, tools, status
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/02-agent-config.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: CONFIGURATION" "Validation and MCP tool inventory for both demo agents"

subheader "STEP 1: Agent Process Status"
run_cli_indented admin agents status

subheader "STEP 2: Validate developer_agent"
run_cli_indented admin agents validate developer_agent

subheader "STEP 3: MCP Tools Available to developer_agent"
run_cli_head 30 admin agents tools developer_agent

subheader "STEP 4: Validate associate_agent"
run_cli_indented admin agents validate associate_agent

subheader "STEP 5: MCP Tools Available to associate_agent"
run_cli_head 30 admin agents tools associate_agent

header "AGENT CONFIG DEMO COMPLETE"
