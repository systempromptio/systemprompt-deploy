#!/bin/bash
# SVG RECORDING: Web Server & Publisher
# Same binary that governs AI agents serves your website.
set -e
source "$(dirname "$0")/_colors.sh"

header "WEB PUBLISHER" "Your governance server is your web server"
pause 1

# ── Content types ──
subheader "Content types" "the content model that drives the publishing pipeline"
pause 0.3

type_cmd "systemprompt web content-types list"
pause 0.3

"$CLI" web content-types list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pause 1.2

divider

# ── Sitemap ──
subheader "Sitemap" "every published route, discoverable"
pause 0.3

type_cmd "systemprompt web sitemap show"
pause 0.3

"$CLI" web sitemap show --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -16 \
  | while IFS= read -r line; do printf "    ${CYAN}%s${R}\n" "$line"; done
echo ""
pause 1.2

divider

# ── Validate ──
subheader "Validate" "type-check templates, assets, and routing before anything ships"
pause 0.3

type_cmd "systemprompt web validate"
pause 0.3

"$CLI" web validate --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pause 1.2

divider

# ── Web extension ──
subheader "Web extension" "jobs, schemas, capabilities — all compiled in"
pause 0.3

type_cmd "systemprompt plugins show web"
pause 0.3

"$CLI" plugins show web --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -14 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
check "same binary serves AI governance and your website"
pause 1.5
