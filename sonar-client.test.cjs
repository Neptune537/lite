const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const vm = require('node:vm');
const {test} = require('node:test');

test('signed Sonar frames reach the client, while tampered, stale and unknown frames do not', async () => {
  const {privateKey, publicKey} = crypto.generateKeyPairSync('ed25519');
  const key = publicKey.export({format: 'der', type: 'spki'}).subarray(-32).toString('hex');
  const now = Date.now();
  const sockets = [];
  const updates = [];
  const sent = [];
  class Socket {
    constructor() { sockets.push(this); }
    send(value) { sent.push(JSON.parse(value)); }
    close() {}
  }
  const context = vm.createContext({
    URL, Uint8Array, TextEncoder, JSON, Map, Date, crypto: crypto.webcrypto,
    location: {hostname: 'heavenlysent-lite-680ac.web.app'},
    WebSocket: Socket, setTimeout: () => 1, clearTimeout() {},
    setInterval: () => 1, clearInterval() {},
    window: {ingestHeavenlyTelemetry: update => { updates.push(JSON.parse(update)); return true; }},
  });
  vm.runInContext(fs.readFileSync(__dirname + '/sonar-client.js', 'utf8'), context);
  context.key = key;
  vm.runInContext('connectHeavenlySonar(["wss://radio.tr7n7t7.com/ws/sonar"], undefined, {"did:test": key}, () => {}, true)', context);
  const socket = sockets[0];
  socket.onopen();
  assert.deepEqual(sent.map(frame => frame.topic), ['gyre:sonar:tactical', 'gyre:telemetry:gods_eye']);
  function frame(observed = now, entityId = "radio") {
    const message = JSON.stringify({source_did: 'did:test', observed_at_unix_ms: observed, entity: {entity_id: entityId}});
    return {action: 'publish', topic: 'gyre:sonar:tactical', event: {
      did: 'did:test', eventType: 'MESH_SIGNAL', timestamp: Math.floor(observed / 1000),
      payload: {message}, signature: crypto.sign(null, Buffer.from(message), privateKey).toString('hex'),
    }};
  }
  await socket.onmessage({data: JSON.stringify(frame())});
  assert.equal(updates.length, 1);
  assert.equal(updates[0].entity.entity_id, 'did:test/radio');
  const tampered = frame();
  tampered.event.payload.message += ' ';
  await socket.onmessage({data: JSON.stringify(tampered)});
  await socket.onmessage({data: JSON.stringify(frame(now - 30001))});
  const unknown = frame();
  unknown.event.did = 'did:unknown';
  await socket.onmessage({data: JSON.stringify(unknown)});
  await socket.onmessage({data: '{invalid'});
  assert.equal(updates.length, 1);
  await Promise.all([
    socket.onmessage({data: JSON.stringify(frame(now, "first"))}),
    socket.onmessage({data: JSON.stringify(frame(now, "second"))}),
  ]);
  assert.equal(updates.length, 3);
  assert.deepEqual(updates.slice(1).map(update => update.entity.entity_id), ["did:test/first", "did:test/second"]);
});

test('production pages reject insecure loopback and LAN endpoints', () => {
  const context = vm.createContext({URL, location: {hostname: 'heavenlysent-lite-680ac.web.app'}});
  vm.runInContext(fs.readFileSync(__dirname + '/sonar-client.js', 'utf8'), context);
  assert.throws(() => vm.runInContext('validateSonarEndpoint("ws://127.0.0.1:8080/ws/sonar")', context));
  assert.throws(() => vm.runInContext('validateSonarEndpoint("ws://10.0.0.59:8080/ws/sonar")', context));
});

test('an open but silent relay rotates to the next endpoint', () => {
  const timers = new Map();
  const sockets = [];
  let id = 0;
  class Socket {
    constructor(url) { this.url = url; sockets.push(this); }
    send() {}
    close() { this.closed = true; }
  }
  const context = vm.createContext({
    URL, Map, Date, WebSocket: Socket,
    location: {hostname: 'heavenlysent-lite-680ac.web.app'},
    setTimeout: (fn, delay) => { timers.set(++id, {fn, delay}); return id; },
    clearTimeout: key => timers.delete(key),
    setInterval: () => 0, clearInterval() {},
    window: {},
  });
  vm.runInContext(fs.readFileSync(__dirname + '/sonar-client.js', 'utf8'), context);
  vm.runInContext('connectHeavenlySonar(["wss://one.example/ws/sonar","wss://two.example/ws/sonar"], undefined, {}, () => {}, true)', context);
  sockets[0].onopen();
  [...timers.values()].find(timer => timer.delay === 30000).fn();
  assert.equal(sockets[0].closed, true);
  [...timers.values()].find(timer => timer.delay === 500).fn();
  assert.equal(sockets[1].url, 'wss://two.example/ws/sonar');
});
