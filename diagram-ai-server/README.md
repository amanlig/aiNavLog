# aiNavLog diagram AI gateway

Small Node 24 service between the app and OpenAI. Keeps the provider key on the
server; accepts an authenticated diagram image and returns component proposals.
Does not receive the maintenance database or save equipment.

See [server setup, workflow and verification](../docs/electrical-diagram-maintenance.md)
and [architecture diagram §5.6](../architecture.md#56-electrical-maintenance-diagram-and-ai-scan-gateway).

Run `npm test` here. For local setup, copy `.env.example` to `.env`, supply the
server-only OpenAI API key and a gateway token of at least 24 characters, then
`npm start`. Remote device access needs an HTTPS host and configured browser
origins. For the separate Docker image and existing-EC2 deployment, see
[deployment instructions](deploy/README.md). The gateway is deployed at
`https://ainavlog.kodekomplete.com` on the existing EC2 host, with a separate Docker
image and a verified real example scan. See the deployment receipt and operational
steps in that guide.
