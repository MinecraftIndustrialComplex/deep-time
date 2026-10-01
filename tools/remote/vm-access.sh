#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Keeps the laptop able to reach the Azure test VM (sourced by mac.sh, mac-gc.sh, mic-climate.sh).
#
# The VM's firewall (network security group) allows SSH only from addresses this laptop has had.
# The laptop's public address changes with the network it is on; on 2026-10-01 it changed twice and
# every agent's job timed out until the rule was edited by hand. Ben's decision that day: "Auto-update
# the rule". So: if the VM does not answer on SSH, add the laptop's current public address to that
# one rule and try again. Login stays key-only; nothing else in the subscription is touched.
#
#   vm_access_ensure <user@host>     returns 0 when SSH answers (after a repair if needed)

VM_RG="${DEEPTIME_VM_RG:-deeptime-test}"
VM_NSG="${DEEPTIME_VM_NSG:-deeptime-testNSG}"
VM_RULE="${DEEPTIME_VM_RULE:-ssh-from-laptop}"
VM_AZURE_IP="${DEEPTIME_VM_IP:-20.120.160.3}"
VM_KEEP=8          # addresses kept in the rule (newest last)

vm_access_ensure() {
    local host="$1" ip prefixes lock
    ssh -n -o BatchMode=yes -o ConnectTimeout=10 "$host" true 2>/dev/null && return 0
    # Only the Azure VM has this rule; any other host is left alone.
    case "$host" in *"$VM_AZURE_IP") ;; *) return 1 ;; esac
    command -v az >/dev/null || { echo "vm-access: $host does not answer and az is not installed" >&2; return 1; }
    lock="${XDG_RUNTIME_DIR:-/tmp}/deeptime-vm-access.lock"
    (
        flock -w 120 9 || exit 1
        # Another job may have repaired it while this one waited for the lock.
        ssh -n -o BatchMode=yes -o ConnectTimeout=10 "$host" true 2>/dev/null && exit 0
        ip="$(curl -s -m 10 https://api.ipify.org | tr -cd '0-9.')"
        [[ "$ip" =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]] || { echo "vm-access: could not read the laptop's public address" >&2; exit 1; }
        prefixes="$(timeout 90 az network nsg rule show -g "$VM_RG" --nsg-name "$VM_NSG" -n "$VM_RULE" --query sourceAddressPrefixes -o tsv 2>/dev/null | tr '\n' ' ')"
        case " $prefixes " in
            *" $ip "*) echo "vm-access: $ip is already allowed but $host does not answer (is the VM running?)" >&2; exit 1 ;;
        esac
        # shellcheck disable=SC2086
        set -- $prefixes "$ip"
        while [ $# -gt "$VM_KEEP" ]; do shift; done
        echo "vm-access: the laptop's address is now $ip; adding it to the VM's SSH rule" >&2
        timeout 120 az network nsg rule update -g "$VM_RG" --nsg-name "$VM_NSG" -n "$VM_RULE" --source-address-prefixes "$@" -o none || exit 1
        for _ in 1 2 3 4 5 6; do
            sleep 5
            ssh -n -o BatchMode=yes -o ConnectTimeout=10 "$host" true 2>/dev/null && exit 0
        done
        echo "vm-access: the rule was updated but $host still does not answer" >&2
        exit 1
    ) 9>"$lock"
}
