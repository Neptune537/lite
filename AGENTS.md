# AGENTS.md — HeavenlyBound Lite / Edge Client (Lemuria V2 Architecture Guide)

This document tracks AI agents working on this repository, their roles, and the strict operational invariants required under the **Lemuria V2 Architecture**.

---

## ⚠️ CRITICAL ARCHITECTURAL DIRECTIVE: LEMURIA V2 STANDARDS

All agents contributing to **HeavenlyBound Lite** MUST understand and uphold the unified **Lemuria V2 Architecture**:

### 1. Pure Rust & Zero-GC Memory Standard
* **Zero OS Heap Allocations in Tick Loops:** All 3D rendering, physics simulation, and spatial queries run using Lemuria's **16MB Double-Buffered Frame Allocator** (`DoubleBufferedFrameAllocator`).
* **Cache-Aligned Memory:** Transforms, entity structs, and bounding volumes are strictly 64-byte aligned to maximize CPU cache utilization and SIMD throughput.
* **$O(1)$ V-Sync Frame Swapping:** Active frame buffers are swapped in $O(1)$ time at display presentation, eliminating all runtime garbage collection stutters.

### 2. Elimination of Chromium, Electron, and Legacy FFI Bridges
* **Desktop is Native Machine Code:** Desktop applications run purely in Rust (`eframe` / `egui` on `wgpu`) directly addressing Vulkan, Metal, or DirectX 12. **No Chromium, No Blink, No V8, and No Electron.**
* **Zero FFI Marshaling:** UI, ECS, physics, and networking share the identical Rust memory heap, eradicating legacy serialization bottlenecks (`flutter_rust_bridge`, `frb_generated.rs`).
* **Web Runtime is Pure WebAssembly + WebGPU:** In browser environments, the client compiles directly to WebAssembly (`wasm32-unknown-unknown`) driving WebGPU/WebGL2 canvas contexts.

### 3. Planetary Geodesics: Nautilus World Server Standard
* **Nautilus Replaces Fluonder:** All legacy mapping (`FluonderGlobe`, `FluonderVectorMap`, `fluoderpod`) is superseded by the **Nautilus World Server (`nautilus_world`)**.
* **Dual-Precision 64-bit ECEF Coordinates:** The planetary digital twin operates in 64-bit (`f64`) Earth-Centered, Earth-Fixed (ECEF) coordinates, downscaled to camera-relative 32-bit (`f32`) offsets in shaders to eliminate sub-millimeter vertex jitter at ground level.
* **No Disconnected Virtual Silos:** Every sector, storefront, garage, and drone exists at an authoritative geodesic locus on Earth.

### 4. Advanced 3D Rendering & Gaussian Splatting
* **Clustered Forward+ Grid:** $16 \times 9 \times 24 = 3,456$ spatial cells for dynamic lighting and GGX PBR evaluated via WGSL compute shaders.
* **GPU Compute Occlusion Culling:** Hierarchical Z-Buffer (HZB) occlusion culling generates `DrawIndexedIndirectCommand` buffers entirely on the GPU.
* **3D Gaussian Splats:** Physical storefront totems, shelf merchandise, and high-fidelity assets render as 3D Gaussian Splats depth-sorted via an 8-bit, 4-pass LSD radix sorter.

### 5. Deterministic 60Hz Physics & Siren Spatial Acoustics
* **Rapier3D Continuous Collision Detection (CCD):** Fixed 60Hz deterministic accumulator with Pacejka Magic Formula tire dynamics.
* **Siren Acoustic Engine:** Sabine $RT_{60} = \frac{0.161 \cdot V}{S \cdot \bar{\alpha}}$ physical reverberation decoding for 24/7 Siren Radio broadcasts ($432\text{ Hz}$).

### 6. Content Creation & Swarm Orchestration
* **Authoritative 3D Editor: Lemuria Arena:** Built upon Mixar Core (`tools/lemuria_arena`), Lemuria Arena is the official 3D scene & totem editor. It preserves native AI text/image-to-3D generation, automated retopology, and PBR painting while exporting `.glb` + `lemuria_manifest.json`.
* **Swarm Orchestrator: Ursula Node Manager:** Tier 2 Pelagic Node Manager (`projects/Ursula`) provides Model Context Protocol (MCP) tooling to sign asset envelopes (`lemuria_ingest_scene`) with Ed25519 DIDs (`did:atlantis:ursula-node-manager`), inspect world state (`lemuria_query_scene`), and route LAN AI inference (`route_inference`).

### 7. AI Harness Standard: The Free Model Revolver Engine
* **Turn-Taking Free Cylinder:** AI agent harnesses (Goose, Ursula, Local AI Gate) maintain an `AiModelRevolver` loaded with free-tier model chambers (Groq Llama 3.3, OpenRouter DeepSeek R1 / Llama, Cloudflare Workers AI, Local Ollama).
* **Rate-Limit Tracking & Spinning:** The cylinder spins on each request turn, tracking RPM/RPD limits and taking tripped chambers offline upon HTTP 429.
* **Defaulting to Gemini Free Tier:** When all free revolver chambers meet their active limits, the harness seamlessly defaults to the **Google Gemini Free Tier** (`gemini-2.5-flash` / `gemini-1.5-flash`) as the persistent anchor.

### 8. Media Standard: The Seven Daughters 24/7 Radio Station
* **Canonical Broadcasters:** The 24/7 Siren Frequency Theater Radio (`siren:frequency:global_stage`, $432\text{ Hz}$ base) is canonically operated by **The Seven Daughters**:
  1. `Maia` (432 Hz • Dawn Oceanic Ambient / Horizon Chill • 04:00-08:00)
  2. `Alcyone` (528 Hz • High-Speed Cyberpunk D&B / Breakbeats • 08:00-12:00)
  3. `Asterope` (639 Hz • Tactical Deep House & Frontline Reports • 12:00-16:00)
  4. `Celaeno` (741 Hz • Glitch / Industrial Bass / Storm Chaser • 16:00-20:00)
  5. `Electra` (852 Hz • Vocal Trance / Eurobeat / Club Atlantis • 20:00-00:00)
  6. `Taygete` (963 Hz • Darkwave / Minimal Techno / Abyssal Drone • 00:00-02:00)
  7. `Merope` (396 Hz • Lo-Fi / Starlight Harbor Acoustic • 02:00-04:00)
* **Live Commentary & Tipping:** Live on-air chatter, sector alerts, and track IDs are generated autonomously by the active daughter via the Revolver Engine, supported by instant 0.1 CYAN tips via Coral State Channels.

### 9. The HeavenlyBound & Lite Dual Platform Standard
* **The Menu / Pause Menu Standard for HeavenlyBound:** HeavenlyBound Lite is the canonical standard for the in-game Menu, Pause Menu, and HUD matrix of HeavenlyBound and all future platform apps.
* **The 6-Tab Universal Architecture:**
  1. 📻 **Theater Radio:** Siren Frequency 24/7 hosted by The 7 Daughters.
  2. 🛒 **Task Marketplace:** Decentralized jobs & escrows.
  3. 🏪 **3D Storefront:** Physical-to-digital totems with $<2.0\text{m}$ walk-up triggers.
  4. 🚗 **Vehicle Dynamics:** Real-time Pacejka tire slip, RPM, and speed gauges.
  5. 🛡️ **Pearl Identity & PoH:** Hardware health scores and player stamina/shield.
  6. 🗺️ **Nautilus Tactical Map:** 64-bit ECEF terrain with Turn-by-Turn GPS guidance.
* **Universal Mandate for All Platform Apps:** Every application built on the Lemuria platform inherits this dual foundation: **HeavenlyBound** sets the flagship 3D simulation/rendering standard, and **Lite** sets the responsive pause menu, HUD, and edge marketplace standard across `CompactPortrait`, `StandardTablet`, and `WidescreenHud`.

### 10. 1-to-1 Real Site Grounding & VR Globe Spectator Standard
* **Warehouse / Park to Virtual Arcade:** Local physical spaces are anchored to the Nautilus globe via `LocalTangentFrame` at authoritative 64-bit WGS84 geodesy (`Wgs84GeoCoord`). Local metric tracking (VR SLAM, SteamVR Lighthouse, RTK-GPS) translates seamlessly into planetary `f64` ECEF space without vertex or spatial drift.
* **Touch-to-Watch on the Nautilus Tactical Globe:**
  - Active VR broadcasts originating from HeavenlyBound appear as holographic beacons on the **🗺️ Nautilus Tactical Map / Globe Tab**.
  - Touching or clicking a broadcast pin switches the client view from orbital perspective to `SectorSlice` dynamic LOD (~500m Karman threshold) centered at the physical site.
  - Spectators receive synchronized 3D avatar pose streams (Kimodo dual-quaternion rigs) and room-tagged Siren spatial audio, with live Coral micro-tipping (0.1 CYAN) over Tide Ledger state channels.

---

## 🏛️ The Three Architecture Pillars & Cross-Pillar Coordination

This project is part of the unified Kuroshio Gyre & Lemuria V2 ecosystem:

1. **Pillar 1: HeavenlyBound Lite / Edge Client (This Repository - `lite`)**
   * Domain: Mobile edge client, local SQLite caching, task marketplace workflows, teams/factions, and user experience.
2. **Pillar 2: Lemuria Engine & Nautilus World Server**
   * **Location:** [`C:\Users\blue-\projects\Kuroshio Gyre\Lemuria`](file:///C:/Users/blue-/projects/Kuroshio%20Gyre/Lemuria)
   * Domain: Pure-Rust GPU rendering (`lemuria_engine`), 16MB double-buffered frame allocators, Clustered Forward+, 3D Gaussian Splats, 64-bit ECEF Nautilus terrain, and 60Hz deterministic physics.
3. **Pillar 3: Atlantis Protocol, Tide Ledger & Coral Escrow**
   * **Location:** [`C:\Users\blue-\projects\Kuroshio Gyre\Atlantis`](file:///C:/Users/blue-/projects/Kuroshio%20Gyre/Atlantis)
   * Domain: Pearl Enclave Ed25519 DIDs (`did:atlantis:`), double-entry Tide Ledger, Coral Escrow settlement, Proof-of-Health (PoH) hardware governance, and Solana Token-2022 reconciliation.

---

## Active Agents Roster (Pillar 1 - Lite App)

| Agent Name / ID | Role / Specialization | Scope | Mandate Under Lemuria V2 |
| :--- | :--- | :--- | :--- |
| **Agent Alpha** | UI & User Workflows | Android & Web | Task creation & inspection workflows aligned with Nautilus geodesic anchors |
| **Agent Beta** | Backend & Security | Firebase & Cloud | Security rules and Cloud Functions enforcing Pearl Enclave DID provenance |
| **Jules** | Edge Persistence & Escrow | SQLite & Ledger | Local persistence and Coral Escrow synchronization with Tide Ledger |
| **Agent Gamma** | Engine Integration | Native `wgpu` | Modular responsive HUD with live Theater 24/7 Siren Radio stream and zero-GC frame allocators |
| **Agent Delta** | Native Hardware / Vulkan | Android / Desktop | HardwareBuffer zero-copy swapchains and Vulkan 1.3 presentation |
| **Agent Epsilon** | Telemetry & State | Mesh Networking | Packing 64-byte entity packets for Lemuria GPU buffer ingestion and DTN queues |
| **Agent Zeta** | Web / Wasm Pipeline | WebAssembly | Pure Rust `wasm32-unknown-unknown` + WebGPU pipeline |
| **Agent Theta** | Power & Thermal Governor | Hardware Telemetry | Proof-of-Health (PoH) dynamic 15–60 FPS power scaling |

---

## Instructions for Agents

1. **Strict Platform Adherence:** Test and optimize for Android NDK / Vulkan and WebAssembly + WebGPU.
2. **Coordinate Geodesy:** Never use flat Cartesian coordinates without an explicit `f64` ECEF anchor to the Nautilus world globe.
3. **Privacy Boundary:** Never broadcast raw high-precision GPS to the public mesh; all external location disclosures must pass through the Club Node's `LocationDisclosurePolicy`.
4. **Tooling & Assets:** Author all 3D assets, storefront totems, and acoustic zones in **Lemuria Arena** (`tools/lemuria_arena`) and sign envelopes via **Ursula** MCP.
