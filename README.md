# HeavenlyBond Lite / Game Maps IRL

HeavenlyBond Lite is a general-purpose community task, service, and gig booking marketplace built with Flutter, local SQLite parallel caching, and decentralized P2P event relay rails.

It restores the ability for ordinary people to turn useful work and services into an opportunity to earn, while allowing the community to decide what work deserves funding.

> [!IMPORTANT]
> **TARGET PLATFORMS: ANDROID AND WEB ONLY**
> All engineering, rendering, build pipelines, and integrations are strictly focused on **Android** (Vulkan NDK via `fluoderpod_render`) and **Web** (Wasm/WebGPU & CanvasKit).
>
> **MAPPING & PLANETARY ENGINE: EXCLUSIVELY FLUONDER**
> All planetary globes, geospatial navigation, radar tactical surfaces, and constellation maps exclusively use **Fluonder** (`FluonderGlobe`, `FluonderVectorMap`, `FluonderCoordinates`, and `package:fluoderpod`). External map engines (Google Maps, MapLibre, Cesium, iframes) are not used.

---

## The Three Architecture Pillars

1. **Pillar 1: Client Application (This Repository)**
   * Mobile (Android) & Web client, Riverpod state management, local SQLite parallel storage (`sqflite`), task & gig booking marketplace, and team/guild workflows.
2. **Pillar 2: Fluorescent & Fluoderpod 3D Graphics Engine ([`third_party/fluorescent`](third_party/fluorescent))**
   * GPU-driven compute frustum/occlusion culling, Nanite-style virtual geometry, unified indirect draw pipeline, Android NDK Vulkan zero-copy bridge (`android_vulkan.rs`), and WebGPU Wasm pipeline.
3. **Pillar 3: TitheX / Fluoridian Blockchain & Nexus Protocol ([`third_party/tithX`](third_party/tithX))**
   * Parallel storage sync, decentralized event envelopes (`NetworkEvent`), `mobile-vault-sdk` Ed25519 signing, WebSocket gossip relay, double-entry internal ledger, and on-chain escrow/staking settlement.

---

## Documentation & Architecture

*   **[Documentation](docs/DOCUMENTATION.md):** Application concept, architecture, data models, task state machine, gig booking, Fluonder vector engine, and security rules.
*   **[Engine Architecture](docs/ENGINE_ARCHITECTURE.md):** Architectural contract covering TithX P2P gossip, Event Envelopes, Fluorescent Vulkan/WebGPU integration, and Fluonder vector math.
*   **[Final Completion Plan](docs/COMPLETION_PLAN.md):** Multi-pillar roadmap and phased execution checklist for Android & Web.
*   **[Build Plan](docs/BUILD_PLAN.md):** Step-by-step roadmap for core marketplace and team sprints.
*   **[AI Agents Tracker](AGENTS.md):** Active agents, roles, and cross-pillar coordination protocols.

---

## Getting Started

### Mandatory Initiation Procedure
At the start of every session or agent handoff:
* **Windows (PowerShell):** `.\scripts\init.ps1`
* **Direct Command:** `git submodule update --init --recursive --remote && flutter pub get && dart run build_runner build --delete-conflicting-outputs`

### Running the App
* **Android:** `flutter run -d android`
* **Web:** `flutter run -d chrome --web-renderer canvaskit`
