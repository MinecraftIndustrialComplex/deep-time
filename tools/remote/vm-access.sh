#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Keeps the laptop able to reach the Azure test VM (sourced by mac.sh, mac-gc.sh, mic-climate.sh,
# tools/viewer/mac-watch.sh).
#
# Two reasons the VM stops answering on SSH, both repaired here:
#  - It is deallocated. Its idle watcher (tools/remote/vm/idle-watcher.py) deallocates it after 30
#    minutes with no work (Ben, 2026-10-03: "Auto-stop when idle"). Then this starts it with
#    `az vm start`, says so, and waits for SSH (a cold start takes 2 to 4 minutes). A VM carrying
#    the watcher's DEALLOCATING marker is on its way down: wait for that to finish, then start it.
#  - The laptop's public address changed. The VM's firewall (network security group) allows SSH only
#    from addresses this laptop has had; on 2026-10-01 it changed twice and every agent's job timed
#    out until the rule was edited by hand. Ben's decision that day: "Auto-update the rule". So the
#    laptop's current address is added to that one rule. Login stays key-only.
# Nothing else in the subscription is touched.
#
#   vm_access_ensure <user@host>     returns 0 when SSH answers (after a start or a repair if needed)

VM_RG="${DEEPTIME_VM_RG:-deeptime-test}"
VM_NAME="${DEEPTIME_VM_NAME:-deeptime-test}"
VM_NSG="${DEEPTIME_VM_NSG:-deeptime-testNSG}"
VM_RULE="${DEEPTIME_VM_RULE:-ssh-from-laptop}"
# The VM's DNS name (its public IP's label, kept across a rebuild); DEEPTIME_VM_IP may be a name or an address.
VM_AZURE_IP="${DEEPTIME_VM_IP:-deeptime-test-ban.eastus.cloudapp.azure.com}"
VM_KEEP=8          # addresses kept in the rule (newest last)
VM_GOING_DOWN=/data/ban/deeptime-cache/DEALLOCATING

# 0: SSH answers and the VM is not going down; 1: no answer; 2: answers but is being deallocated.
vm_probe() {
    ssh -n -o BatchMode=yes -o ConnectTimeout=10 "$1" "test ! -e $VM_GOING_DOWN" 2>/dev/null
    case $? in 0) return 0 ;; 1) return 2 ;; *) return 1 ;; esac
}

vm_power_state() {
    timeout 120 az vm get-instance-view -g "$VM_RG" -n "$VM_NAME" \
        --query "instanceView.statuses[?starts_with(code, 'PowerState/')].code | [0]" -o tsv 2>/dev/null
}

vm_wait_ssh() {   # <host> <seconds>
    local end=$(( $(date +%s) + $2 ))
    while [ "$(date +%s)" -lt "$end" ]; do
        vm_probe "$1" && return 0
        sleep 5
    done
    return 1
}

# Start the VM if Azure says it is stopped or on its way there; 0 when SSH then answers.
vm_start_if_stopped() {
    local host="$1" state t0 down=0 waited=0
    state="$(vm_power_state)"
    case "$state" in
        PowerState/deallocating|PowerState/stopping) down=1 ;;
        # running + the DEALLOCATING marker: the deallocate was just sent; Azure has not caught up.
        PowerState/running) vm_probe "$host"; [ $? = 2 ] && down=1 ;;
    esac
    if [ "$down" = 1 ]; then
        echo "vm-access: the test VM $VM_NAME is being deallocated (idle watcher); waiting for that to finish, then starting it" >&2
        while :; do
            case "$state" in PowerState/deallocated|PowerState/stopped) break ;; esac
            [ "$waited" -ge 600 ] && { echo "vm-access: $VM_NAME is still '$state' after 10 min" >&2; return 1; }
            sleep 10; waited=$((waited + 10))
            state="$(vm_power_state)"
        done
    fi
    case "$state" in
        PowerState/deallocated|PowerState/stopped)
            echo "vm-access: STARTING the test VM $VM_NAME (it was ${state#PowerState/}: the idle watcher stops it after 30 idle minutes); this takes a few minutes" >&2
            t0=$(date +%s)
            timeout 600 az vm start -g "$VM_RG" -n "$VM_NAME" -o none || { echo "vm-access: az vm start failed" >&2; return 1; }
            if vm_wait_ssh "$host" 300; then
                echo "vm-access: started the test VM $VM_NAME; SSH answers after $(( $(date +%s) - t0 )) s (the workbench and play server come back as services)" >&2
                return 0
            fi
            echo "vm-access: started $VM_NAME, but SSH does not answer yet; checking the firewall rule" >&2
            return 1 ;;
        PowerState/starting)
            echo "vm-access: the test VM $VM_NAME is starting (another job started it); waiting for SSH" >&2
            vm_wait_ssh "$host" 300 && return 0
            return 1 ;;
        "") echo "vm-access: could not read $VM_NAME's power state (az)" >&2; return 1 ;;
        *) return 1 ;;   # running: the address is the likelier cause
    esac
}

vm_access_ensure() {
    local host="$1" lock
    vm_probe "$host" && return 0
    # Only the Azure VM is started or has this rule; any other host is left alone.
    case "$host" in *"$VM_AZURE_IP") ;; *) ssh -n -o BatchMode=yes -o ConnectTimeout=10 "$host" true 2>/dev/null; return ;; esac
    # NixOS installs nothing globally: after a laptop restart az is not on PATH, so take it from nixpkgs.
    command -v az >/dev/null || PATH="$(nix build --no-link --print-out-paths nixpkgs#azure-cli 2>/dev/null)/bin:$PATH"
    command -v az >/dev/null || { echo "vm-access: $host does not answer and az is not installed (nix build nixpkgs#azure-cli failed)" >&2; return 1; }
    lock="${XDG_RUNTIME_DIR:-/tmp}/deeptime-vm-access.lock"
    (
        # Several agents may find the VM down at once: one starts it, the others wait here.
        flock -w 900 9 || exit 1
        vm_probe "$host" && exit 0
        vm_start_if_stopped "$host" && exit 0
        vm_probe "$host" && exit 0
        local ip prefixes
        ip="$(curl -s -m 10 https://api.ipify.org | tr -cd '0-9.')"
        [[ "$ip" =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]] || { echo "vm-access: could not read the laptop's public address" >&2; exit 1; }
        prefixes="$(timeout 90 az network nsg rule show -g "$VM_RG" --nsg-name "$VM_NSG" -n "$VM_RULE" --query sourceAddressPrefixes -o tsv 2>/dev/null | tr '\n' ' ')"
        case " $prefixes " in
            *" $ip "*) echo "vm-access: $ip is already allowed but $host does not answer (power state: $(vm_power_state))" >&2; exit 1 ;;
        esac
        # shellcheck disable=SC2086
        set -- $prefixes "$ip"
        while [ $# -gt "$VM_KEEP" ]; do shift; done
        echo "vm-access: the laptop's address is now $ip; adding it to the VM's SSH rule" >&2
        timeout 120 az network nsg rule update -g "$VM_RG" --nsg-name "$VM_NSG" -n "$VM_RULE" --source-address-prefixes "$@" -o none || exit 1
        vm_wait_ssh "$host" 30 && exit 0
        echo "vm-access: the rule was updated but $host still does not answer" >&2
        exit 1
    ) 9>"$lock"
}
