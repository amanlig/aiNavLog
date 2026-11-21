# EC2 deployment — separate diagram AI Docker image

Deployed November 4, 2026 to the existing EC2 host in us-west-2 at
`https://ainavlog.kodekomplete.com`. [Health endpoint](https://ainavlog.kodekomplete.com/healthz).
TLS, gateway authentication, CORS and one real example scan passed (10 component
proposals in 5.74 seconds). The existing aiMoney site remains healthy.
No new EC2 instance or public backend port was created.

## Image and runtime

- Image: `ainavlog-diagram-ai:2026-11-04-3` (Linux amd64).
- Source image/config digest: `sha256:3b98ac53d13c21bf55478cec1b8d72c06eb27465c5ba1b3b5fc908d7a78e109c`.
- Official Node 24 Debian base pinned in `Dockerfile` by digest.
- Runtime user: `node` (UID/GID 1000); read-only filesystem; all capabilities
  dropped; no new privileges; 256 MiB RAM / 0.5 CPU / 64 PID limits.
- No host ports or database. The service joins the verified existing Caddy
  network with DNS alias `ainavlog-diagram-api` on port 8788.
- Runtime secrets are files mounted read-only under `/run/secrets`, outside
  image layers and source archives. The image contains only server, healthcheck
  and package metadata. Never copy an `.env` or `.secrets` folder into the image.
- `GET /healthz` is a local liveness test; it makes no provider request and does
  not verify OpenAI credentials. Scans require the gateway token.
- SIGTERM drains in-flight requests; Compose grants 100 seconds before stopping.

Build and save (from repository root):

```sh
docker build -t ainavlog-diagram-ai:2026-11-04-3 diagram-ai-server
docker save ainavlog-diagram-ai:2026-11-04-3 | gzip > /tmp/ainavlog-diagram-image.tar.gz
```

The deployment lives in `/home/ubuntu/aiNavLog-diagram`, Compose project
`ainavlog-diagram`. Secrets are its own protected files, copied/generated on the
host after owner authorization. The gateway token is also saved locally in the
ignored `.secrets/diagram_scan_token` for private entry in the app. The OpenAI key
was not transferred back or embedded in the image. Mobile build profiles now
include the public scan URL; new mobile builds remain required.

Deployment receipts and sample results are under ignored
`user-interface/builds/diagram-server-*-2026-11-04.json`. The image archive SHA-256
is `e4dce0f708dd8dda11c131cbde52669e3185101aca4ae7b9ec9620ba8a7cd252`.
Docker 27's source config digest and Docker 29's imported image/manifest ID differ;
archive checksum, exact rootfs layers and runtime configuration were verified
before use. Do not treat these two identifier formats as interchangeable.

## Inspect the existing EC2 host first

Use the existing operator AWS profile and verify account/region/instance ID before
writing anything. Current local profile is `aimoney`, region `us-west-2`; those
names alone do not prove which live host should be changed. Inspect running
containers, their Compose labels/networks, disk/memory, the Caddyfile mount and
existing site health. Record the before-state, Caddy configuration backup and
current proxy image/config. Keep existing application/database containers and
certificate volumes intact. Do not expose the diagram port in a security group.

If `aws login` is expired or browser callbacks fail:

```sh
aws login --profile aimoney --region us-west-2 --remote
```

Open the complete fresh URL and paste the browser's authorization code into that
terminal, never into chat, shell history or deployment scripts. When using SSM,
never place secret values in Run Command parameters/CloudTrail-visible scripts;
use encrypted secret provisioning or existing protected files.

## Prepare a separate deployment directory on EC2

Suggested layout under the SSH operator's home:

```text
aiNavLog-diagram/
  compose.ec2.yaml
  .env.ec2                    # public runtime settings; mode 600
  .secrets/                   # mode 700; readable by the operator
    openai_api_key            # mode 400, owner UID 1000
    diagram_scan_token        # mode 400, owner UID 1000; >=24 characters
  deploy/install-ec2.sh
```

Transfer the tested image over authenticated SSH or an approved private artifact
channel; verify its archive SHA-256 and image ID before `docker load`/startup.
Copy `compose.ec2.yaml`, `deploy/install-ec2.sh`, and the runtime settings template.
Create `.env.ec2` from `deploy/ec2.env.example`; set `PROXY_NETWORK` to the
network actually used by Caddy. Set exact allowed browser origins as needed;
native requests omit Origin. Never print expanded secret-bearing config.

Provision the OpenAI key securely on the server. Generate the scan token there
with a cryptographic generator and write it directly to its secret file. Do not
print either secret in deployment logs. Reusing a key from another application
must be a deliberate owner decision; the diagram container gets its own runtime
secret file, not a mount of the other application's secrets directory.

Run `sh deploy/install-ec2.sh`. It validates configuration, starts only the new
Compose project and waits for health. It does not modify Caddy or DNS.

## HTTPS integration and verification

Use the existing Caddy instance, with a new host block from `Caddyfile.fragment`.
Replace `SCAN_HOSTNAME` with the agreed DNS hostname. Verify that its Route 53
record resolves to this EC2 host's existing Elastic IP before TLS issuance.
Preserve the existing site's block, authentication headers and certificate
volumes. The new block must forward the gateway Authorization header unchanged.

Back up the Caddyfile, append only the new block, validate it with the running
Caddy binary, then reload Caddy. If validation/reload fails, restore the backup;
do not restart unrelated services or reset certificate volumes.

Verify both the original site and new scan hostname after reload:

1. HTTPS certificate/hostname validates; `GET /healthz` returns 200.
2. Unauthenticated `POST /scan-diagram` returns 401.
3. Authenticated invalid input returns 400 without calling OpenAI.
4. One explicit sample-diagram scan verifies the real provider response without
   sending any equipment database/service records.
5. Image ID, health and restart policy match the tested release; no host port 8788.

Record instance ID, image ID, endpoint, deployment time and verification outcomes
without secrets. Set `EXPO_PUBLIC_DIAGRAM_SCAN_URL` for subsequent mobile builds.
The gateway token must be entered privately in the app; it is not the OpenAI key.

## Rollback

Restore the prior Caddyfile and reload after validation. Stop only this project's
`diagram-api` service, or set its previous `DIAGRAM_IMAGE` and run the installer.
Keep the previous image tag available. Never run another stack's `down -v` or
remove Caddy certificate/database volumes. Remove only a DNS record created by
this deployment if it is no longer needed and its exact value is verified.
