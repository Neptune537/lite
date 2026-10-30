# Kuroshio Platform Dashboard & Companion In-App Menu Standard
*(Formerly HeavenlyBond Lite / Game Maps IRL)*

The **Dashboard** is the official Tier 2 client companion shell and standardized in-app menu interface across the entire Kuroshio Gyre ecosystem. It transitions the legacy Flutter/Firebase/tithX architecture into the modern pure-Rust, Lumeria WASM sandboxed, and Coral-settled platform standard.

---

## 🏛️ Standard Architecture: Dashboard & Companion Shell

The Dashboard serves two unified modes across the platform:
1. **Standalone Companion App:** The primary desktop (Windows/macOS/Linux) and mobile (Android/iOS) hub for community task bounties, IRL spatial navigation, guild rosters, 24/7 Siren Radio, and wallet management.
2. **Standardized In-App Menu Component:** The universal overlay HUD embedded inside all platform applications, games, and sub-apps (e.g., Lemuria AAA, Tactical Wings, and community WASM modules).

### Core Capabilities:
- **Decentralized Bounty & Gig Marketplace:** Off-chain 11-stage task lifecycle, ECEF geodetic task totems, and proximity check-ins (
autilus-world).
- **Universal Companion In-App Menu:** Standardized HUD overlay containing the Siren Radio player, squad/guild selector, and Coral state channel tipping.
- **Hardware-Backed Key Vault:** Zero-exposure DID key management using platform-native enclaves (Apple Keychain, AndroidKeyStore StrongBox, Windows DPAPI).
- **Offline & Low-Spec Resilient:** Runs smoothly on budget laptops and older mobile devices using Slint UI and bounded LRU caches (capped at 250 MB).
- **Zero Olympus Dependency:** Operates peer-to-peer and offline without creating runtime dependencies on the Olympus Apex Keystone.

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
