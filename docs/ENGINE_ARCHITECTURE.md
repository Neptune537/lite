# TithX & Mobile Client Architecture Integration

This document outlines the architectural contract between the Mobile Client (HeavenlyBond Lite / Game Maps IRL) and the TithX Core Engine (Nexus Protocol).

To achieve a true decentralized marketplace, the system relies on a **Local-First, Peer-to-Peer (P2P)** architecture.

## 1. Core Philosophy: Parallel Storage
The mobile client maintains its own sovereign state using a local SQLite database (`sqflite`). It operates in parallel to the Nexus chain.
*   **Offline-First:** Users can create tasks, form clubs (teams), and stage bids without a network connection.
*   **Settlement vs. State:** The actual Nexus Blockchain is reserved for financial settlement (escrow, releasing funds) and Proof of Health consensus. Heavy, mutable application state (task descriptions, marketplace discovery) operates entirely off-chain via P2P relay.

## 2. The Flexible Schema: Event Envelopes
To allow the mobile app to rapidly iterate and create new features without requiring database migrations on the TithX backend, TithX must act as a **data-agnostic relay**.

TithX should not implement rigid relational tables for application-level concepts (e.g., no `Task` or `Bid` tables). Instead, TithX should rely on an **Event Envelope**:

```prisma
// Example TithX Relay Schema Concept
model NetworkEvent {
  id        String   @id // Hash of the payload
  did       String   // Decentralized ID of the creator (did:nexus:...)
  eventType String   // E.g., "MARKETPLACE_BID", "TASK_CREATED"
  payload   String   // JSON blob (The actual data)
  signature String   // Ed25519 signature from MobileVaultSdk
  createdAt DateTime @default(now())
}
```
**Workflow:**
1. The Flutter app serializes a `Bid` to JSON.
2. The `mobile-vault-sdk` signs the JSON payload.
3. The app broadcasts the Envelope to TithX.
4. TithX validates the signature against the DID and relays it.

## 3. Real-Time Discovery (Gossip over WebSockets)
To allow users to see what other connected peers have to offer dynamically:
*   TithX nodes must implement a **Pub/Sub Gossip Protocol** (e.g., via WebSockets).
*   When a mobile user comes online, they subscribe to relevant marketplace topics (e.g., `topic:bids:plumbing`).
*   As other users submit Event Envelopes, the TithX node broadcasts them to all topic subscribers in real-time.
*   The Flutter app receives the WebSocket event, validates the signature, and persists the payload into its own local SQLite database.

## 4. Mobile Vault Integration
The `mobile-vault-sdk` will eventually be bridged to Dart (via `flutter_rust_bridge`).
*   It will manage the Secure Enclave key generation.
*   It will act as the signing authority for every Event Envelope pushed to the SQLite database and TithX WebSocket.

## 5. Graphics & Planetary Engine Integration: Fluorescent & Fluonder
All 3D visualization, constellation graphs, and geospatial map navigation in this project are strictly integrated across two high-performance graphics technologies:

### 5.1 Fluorescent & Fluoderpod Native Engine (Android & Web Only)
*   **Android (Vulkan NDK):** Uses `fluoderpod_render` (`android_vulkan.rs`) with NDK `ANativeWindow` handles bound to Flutter's `SurfaceProducer` texture registry via zero-copy GPU swapchain buffers.
*   **Web (WebGPU / Wasm):** Compiles through the Wasm pipeline to target the WebGPU canvas context.
*   **Compute Culling:** Hardware-accelerated frustum and occlusion culling with Nanite-style cluster LOD evaluation and unified indirect draw calls.

### 5.2 Fluonder Geospatial & Planetary Engine (Strict Mapping Exclusive)
*   All mapping, planetary tactical globes, radar surfaces, and location views strictly and exclusively use **Fluonder** (`FluonderGlobe`, `FluonderVectorMap`, `FluonderCoordinates`).
*   **Spherical Unit Vector Splines:** Precomputes cubic Catmull-Rom splines directly on 3D unit vectors normalized to the unit sphere $S^2$, eliminating dateline seam artifacts and low-poly jaggedness.
*   **Exact Limb Horizon Clipping:** Calculates precise edge intersections with the horizon plane ($z = 0$) at $t = \frac{z_1}{z_1 - z_2}$, terminating visible coastline strokes and tracing landmass fills along the spherical limb arc without ocean-crossing chords.
*   **Astronomical Solar Shading:** Real-time subsolar day/night terminator with dusk scattering aura ($3.0^\circ$ civil twilight band), specular ocean glint, and night-side metropolitan lights with golden phosphor bloom.

## Summary for Engineering Teams
1.  **TithX / Consensus:** Ed25519 signature verification, WebSocket gossip relay, DID resolution, and escrow settlement.
2.  **Fluorescent / Graphics:** Vulkan NDK zero-copy bridge (Android) and WebGPU Wasm pipeline (Web).
3.  **Fluonder / Mapping:** Exclusive mathematical spherical vector rendering across all planetary views.
