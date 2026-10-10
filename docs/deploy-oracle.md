# Coordinator on Oracle Cloud Free Tier

The coordinator runs on an Always Free ARM VM ($0/month, forever). One-time
console setup (~15 min), then `./scripts/deploy-oracle.sh` handles everything.

## 1. Account

Sign up at [oracle.com/cloud/free](https://www.oracle.com/cloud/free/).
A credit card is required for identity verification but is not charged —
Always Free resources never bill. Pick your home region carefully (it can't
be changed): less popular regions have better ARM capacity.

## 2. Create the VM

Console → Compute → Instances → **Create instance**:

- **Image:** Ubuntu 24.04 (aarch64)
- **Shape:** `VM.Standard.A1.Flex` — 1 OCPU / 6 GB is plenty
  (the free tier allows 4 OCPU / 24 GB total, so this leaves room)
- **SSH keys:** paste your `~/.ssh/id_ed25519.pub`
- Leave networking defaults (public IP assigned)

> **"Out of capacity" error:** A1 instances in popular regions fill up.
> Retry at off-peak hours, or fall back to `VM.Standard.E2.1.Micro` (x86,
> also Always Free, two allowed) — the deploy script works on either.

## 3. Open port 80 in the cloud firewall

Console → Networking → Virtual Cloud Networks → your VCN → Security Lists →
Default Security List → **Add Ingress Rule**:

- Source CIDR: `0.0.0.0/0`
- IP Protocol: TCP
- Destination Port Range: `80`

(The VM's own iptables firewall is handled by the deploy script.)

## 4. Deploy

Make sure `.env` in the project root has `RUNPOD_API_KEY=...`, then:

```bash
./scripts/deploy-oracle.sh <vm-public-ip>
```

The script is idempotent — rerun it any time to push coordinator/frontend
changes. It syncs the code, builds the venv, opens the host firewall, and
installs a `veiled-court` systemd service (auto-starts on boot, restarts on
crash, binds port 80).

## Operating

```bash
ssh ubuntu@<ip> journalctl -u veiled-court -f    # logs
ssh ubuntu@<ip> sudo systemctl restart veiled-court
curl http://<ip>/status                          # pod status
```

Note: the systemd `Restart=always` means a coordinator crash won't orphan a
GPU pod for long — the watchdog resumes on restart and stops any pod whose
`/activity` shows it idle past the timeout.

## Later: domain + HTTPS

Plain HTTP is fine to start (the game itself runs on the RunPod pod over
HTTPS). To add a domain later: point an A record at the VM, install Caddy
(`reverse_proxy localhost:8000`), switch the systemd unit to port 8000, and
open 443 in both firewalls. Caddy handles certificates automatically.
