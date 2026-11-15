const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '../../heavenlyBound/dist');
const game = fs.readFileSync(path.join(root, 'index.html'), 'utf8');
const lite = fs.readFileSync(path.resolve(__dirname, '../crates/hblite_desktop/src/earth.rs'), 'utf8');
const shader = fs.readFileSync(path.resolve(__dirname, '../crates/hblite_desktop/src/earth.wgsl'), 'utf8');
const spec = JSON.parse(fs.readFileSync(path.join(root, 'assets/globe-render-v1.json'), 'utf8'));
assert.equal(spec.version, 1);
assert.equal(spec.model, 'WGS84');
assert.equal(spec.equatorialRadiusMeters, 6378137);
assert.equal(spec.inverseFlattening, 298.257223563);
assert.ok(game.includes("fetch('assets/globe-render-v1.json')"));
assert.ok(game.includes('gl.uniform3fv(uEarthSun, globeRenderSpec.sunDirection)'));
assert.ok(game.includes('gl.uniform1f(uEarthCloudDrift, globeRenderSpec.cloudDriftTurnsPerSecond)'));
assert.ok(lite.includes('globe-render-v1.json'));
assert.ok(lite.includes('values[16..19].copy_from_slice(&spec.sun)'));
assert.ok(shader.includes('normalize(view.sun.xyz)'));
for (const asset of Object.values(spec.textures)) {
  assert.ok(fs.existsSync(path.join(root, asset)), `missing shared texture ${asset}`);
  assert.ok(game.includes(asset), `game does not load ${asset}`);
  assert.ok(lite.includes(path.basename(asset)), `Lite does not embed ${asset}`);
}
console.log('Nautilus globe geometry, lighting, cloud rate and four source maps share version 1 contract.');
