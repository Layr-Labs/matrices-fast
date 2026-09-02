#!/usr/bin/env bash
# Sanity check that outbound network is blocked in a grader sandbox. By default
# this remains diagnostic for local/manual use. Set SSI_REQUIRE_NETWORK_BLOCK=1
# in CI to make an unexpected successful connection fail closed.
#
# We attempt a short TCP connection to a well-known host+port. If it SUCCEEDS,
# egress is NOT blocked — we print a loud warning. If it FAILS (the expected,
# desired outcome), we note that isolation looks effective.
set -uo pipefail

target_host="1.1.1.1"
target_port="443"

if timeout 5 bash -c "cat < /dev/null > /dev/tcp/${target_host}/${target_port}" 2>/dev/null; then
  echo "::warning::assert-no-network: outbound connection to ${target_host}:${target_port} SUCCEEDED — the process is NOT network-isolated." >&2
  if [[ "${SSI_REQUIRE_NETWORK_BLOCK:-0}" == "1" ]]; then
    exit 1
  fi
else
  echo "assert-no-network: outbound to ${target_host}:${target_port} blocked — network isolation looks effective."
fi
