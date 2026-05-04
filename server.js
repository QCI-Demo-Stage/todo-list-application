import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function listenPort() {
  const raw = process.env.PORT;
  if (!raw) return 8000;
  const n = Number.parseInt(raw, 10);
  return Number.isFinite(n) && n > 0 ? n : 8000;
}

function nodeEnvValue() {
  return process.env.NODE_ENV || 'staging';
}

function docsEnabledForEnv(nodeEnv) {
  return nodeEnv === 'development' || nodeEnv === 'staging';
}

const openAPISpec = fs.readFileSync(path.join(__dirname, 'api-spec.yaml'));
const swaggerIndexHTML = fs.readFileSync(path.join(__dirname, 'swagger', 'index.html'));

export function createRequestListener(nodeEnv) {
  const docsEnabled = docsEnabledForEnv(nodeEnv);

  return function requestListener(req, res) {
    if (req.method !== 'GET') {
      res.writeHead(405);
      res.end();
      return;
    }

    const host = req.headers.host || 'localhost';
    const pathname = new URL(req.url || '/', `http://${host}`).pathname;

    if (pathname === '/health') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end('{"status":"ok"}');
      return;
    }

    if (!docsEnabled) {
      res.writeHead(404);
      res.end();
      return;
    }

    if (pathname === '/api-spec.yaml') {
      res.writeHead(200, { 'Content-Type': 'application/yaml; charset=utf-8' });
      res.end(openAPISpec);
      return;
    }

    if (pathname === '/api-docs') {
      res.writeHead(301, { Location: '/api-docs/' });
      res.end();
      return;
    }

    if (pathname === '/api-docs/' || pathname.startsWith('/api-docs/')) {
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
      res.end(swaggerIndexHTML);
      return;
    }

    res.writeHead(404);
    res.end();
  };
}

export function start() {
  const port = listenPort();
  const nodeEnv = nodeEnvValue();
  const server = http.createServer(createRequestListener(nodeEnv));

  server.listen(port, () => {
    console.log(`listening on http://localhost:${port} (NODE_ENV=${nodeEnv})`);
  });
}

const isMain =
  process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url));

if (isMain) {
  start();
}
