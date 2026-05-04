import assert from 'node:assert/strict';
import http from 'node:http';
import test from 'node:test';
import { createRequestListener } from './server.js';

function get(options, path, headers = {}) {
  return new Promise((resolve, reject) => {
    const req = http.request({ ...options, path, method: 'GET', headers }, (res) => {
      const chunks = [];
      res.on('data', (c) => chunks.push(c));
      res.on('end', () => {
        resolve({
          statusCode: res.statusCode,
          headers: res.headers,
          body: Buffer.concat(chunks).toString('utf8'),
        });
      });
    });
    req.on('error', reject);
    req.end();
  });
}

test('GET /health returns JSON ok', async () => {
  const listener = createRequestListener('staging');
  const server = http.createServer(listener);

  await new Promise((r) => server.listen(0, r));
  const { port } = server.address();

  try {
    const res = await get({ hostname: '127.0.0.1', port }, '/health');
    assert.equal(res.statusCode, 200);
    assert.equal(res.headers['content-type'], 'application/json');
    assert.equal(res.body, '{"status":"ok"}');
  } finally {
    await new Promise((r) => server.close(r));
  }
});

test('docs routes 404 when NODE_ENV is production', async () => {
  const listener = createRequestListener('production');
  const server = http.createServer(listener);

  await new Promise((r) => server.listen(0, r));
  const { port } = server.address();

  try {
    const spec = await get({ hostname: '127.0.0.1', port }, '/api-spec.yaml');
    assert.equal(spec.statusCode, 404);

    const docs = await get({ hostname: '127.0.0.1', port }, '/api-docs/');
    assert.equal(docs.statusCode, 404);
  } finally {
    await new Promise((r) => server.close(r));
  }
});

test('GET /api-docs redirects to /api-docs/', async () => {
  const listener = createRequestListener('staging');
  const server = http.createServer(listener);

  await new Promise((r) => server.listen(0, r));
  const { port } = server.address();

  try {
    const res = await get({ hostname: '127.0.0.1', port }, '/api-docs');
    assert.equal(res.statusCode, 301);
    assert.equal(res.headers.location, '/api-docs/');
  } finally {
    await new Promise((r) => server.close(r));
  }
});
