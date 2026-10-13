# AI Agents Tracker

This document tracks the AI agents working on this repository, their roles, and their current assignments across the 3-pillar ecosystem.

> [!IMPORTANT]
> **TARGET PLATFORMS: ANDROID AND WEB ONLY**
> All engineering, rendering, build pipelines, and integrations in this repository are strictly focused on **Android** and **Web**.
> * **Android:** Uses Android NDK, Vulkan rendering (`fluoderpod_render` / `android_vulkan.rs`), and Android SDK APIs.
> * **Web:** Uses Flutter Web, WebAssembly (Wasm), and WebGPU / CanvasKit.
> * Other platforms (iOS, macOS, Windows/Linux desktop) are deprioritized and out of scope.
>
> **MAPPING & PLANETARY ENGINE: ONLY USE FLUONDER**
> All mapping, geospatial rendering, globes, radar surfaces, and planetary tactical views in this project must strictly and exclusively use **Fluonder** (`FluonderGlobe`, `FluonderVectorMap`, `FluonderCoordinates`, and `package:fluoderpod`).
> * Do NOT use external map tile engines, MapLibre, Google Maps, Cesium, or external iframes for globes/maps in this repository.
> * All spatial visualization is driven procedurally and mathematically by Fluonder's vector map pipeline, spherical unit vectors, and culling architecture.

---

## The Three Architecture Pillars & Cross-Pillar Coordination

This project is part of a 3-pillar ecosystem. Agents working on this repository **MUST** review and coordinate with the `AGENTS.md` files of the other two pillars:

1. **Pillar 1: HeavenlyBond Lite / Game Maps IRL (This Repository)**
   * `AGENTS.md` (Current file)
   * Domain: Mobile & Web client, Riverpod state management, local SQLite caching, task marketplace workflows, teams/factions, and user experience.
2. **Pillar 2: Fluorescent & Fluoderpod 3D Graphics Engine**
   * **Location:** [`third_party/fluorescent/AGENTS.md`](file:///c:/Users/blue-/projects/lite/third_party/fluorescent/AGENTS.md)
   * Domain: GPU-driven rendering (`fluoderpod_render`), compute frustum/occlusion culling, Nanite-style virtual geometry, unified indirect draw pipeline, Vulkan (Android) and WebGPU (Web).
3. **Pillar 3: TitheX / Fluoridian Blockchain & Nexus Protocol**
   * **Location:** [`third_party/fluorescent/third_party/tithX/AGENTS.md`](file:///c:/Users/blue-/projects/lite/third_party/fluorescent/third_party/tithX/AGENTS.md)
   * Domain: Parallel storage sync, decentralized event envelopes (`NetworkEvent`), `mobile-vault-sdk` Ed25519 signing, WebSocket gossip relay, double-entry internal ledger, and on-chain escrow/staking settlement.

---

## Active Agents (Pillar 1 - Lite App)

| Agent Name / ID | Role / Specialization | Current Task | Status | Target Scope | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Agent Alpha** | UI / Frontend | Task Creation Wizard (Sprint 2) | In Progress | Android & Web | Building `CreateTaskPage` 5-step wizard scaffold |
| **Agent Beta** | Backend / Firebase | Security Rules & Cloud Functions | Completed | Android & Web | Rules authored, awaiting verification hooks |
| **Jules** | UI / Local DB & Blockchain | Sprint 3 & 4: Bids, Offers & Verification | In Progress | Android & Web | SQLite persistence + Fluoridian P2P relay event envelopes |
| **Agent Gamma** | UI / Engine Integration | UPGRADE-02: Universal 3D Viewport Replacement | In Progress | Android & Web | Replacing placeholder tabs with `FluoderpodView` |
| **Agent Delta** | Native NDK / Vulkan | UPGRADE-03: Android Vulkan Zero-Copy Bridge | Ready | Android Only | HardwareBuffer swapchain & NDK 28 texture registration |
| **Agent Epsilon** | Systems & State | UPGRADE-01: Riverpod Binary Streamer | Ready | Android & Web | Packing 64-byte entity packets for GPU buffer ingestion |
| **Agent Zeta** | Web / Wasm / Jaspr | UPGRADE-04: WebGPU & Jaspr CanvasKit Interop | Ready | Web Only | Pure Dart Jaspr client + Wasm WebGPU pipeline |
| **Agent Theta** | Simulation & Power | UPGRADE-05: Real-Time Spatial VFX & Throttler | Ready | Android & Web | Volumetric beacon shaders & 15-60 FPS thermal scaling |

---

## Instructions for Agents
* **Mandatory Initiation Procedure:** At the start of EVERY new development session or agent handoff, execute the initiation script to recursively pull latest commits from all 3 pillars and re-generate code:
  * Windows (PowerShell): `.\scripts\init.ps1`
  * Linux/macOS/Bash: `./scripts/init.sh`
  * Direct Git command: `git submodule update --init --recursive --remote && flutter pub get && dart run build_runner build --delete-conflicting-outputs`
* **Platform Adherence:** Strictly test and optimize code for Android and Web.
* **Fluonder Exclusive Mapping:** Only use **Fluonder** (`FluonderGlobe`, `FluonderVectorMap`, `FluonderCoordinates`) for all planetary, map, globe, and geospatial navigation views. Do not introduce or use other map/globe frameworks.
* **Cross-Pillar Alignment:** When interacting with 3D views (Constellation, Mission Control, Neural Board) or blockchain/relay streams, refer directly to Pillar 2 (`third_party/fluorescent/AGENTS.md`) and Pillar 3 (`third_party/fluorescent/third_party/tithX/AGENTS.md`).
* When picking up or concluding a task, update this file to reflect your latest assignment and status.
* Follow the architectural blueprints in `docs/DOCUMENTATION.md`, `docs/BUILD_PLAN.md`, and `docs/ENGINE_ARCHITECTURE.md`.
