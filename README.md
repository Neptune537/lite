# Kuroshio Dashboard / Lite

Dashboard is the companion client and intended shared menu for the platform. The active client uses Rust, egui and wgpu in `crates/hblite_desktop`.

Read [platform runtime status](../PLATFORM_RUNTIME_STATUS.md) for verified behavior, limitations and next work, and [AGENTS.md](AGENTS.md) before editing.

## Build and run

Keep the sibling `heavenlyBound` checkout present: Cargo path dependencies and embedded Earth textures require it. From this directory:

```powershell
trunk build --release
python -m http.server 8766 --bind 127.0.0.1 --directory dist
```

Open <http://127.0.0.1:8766>. Provision the Rust WebAssembly target and Trunk before building. CI still needs pinned sibling dependencies and reproducible release inputs.

## Current scope

The globe uses shared HeavenlyBound Earth image assets through the GPU renderer. Club chat, marketplace and storefront currently display unavailable states on their normal entry paths. Legacy code contains fixed state and authentication toggles that require a service audit before release.

Public Sonar availability, two-account messaging/presence and cross-app sessions need end-to-end verification. The `android` folder is the legacy Flutter wrapper; signed Android and Quest packages remain release work.

Run Sonar client checks from the projects directory:

```powershell
node --test dashboard/sonar-client.test.cjs
```

The documents under `docs` preserve earlier Flutter/TithX designs. Their status notices distinguish historical plans from the active runtime. Service deployment evidence belongs in the [operations runbook](../Kuroshio%20Gyre/OPERATIONS_RUNBOOK.md).
