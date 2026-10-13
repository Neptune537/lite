# HeavenlyBond Lite Documentation

## Overview

HeavenlyBond Lite is a general-purpose community task and service marketplace.

The core premise is:
**Anyone can create a legitimate task. The community can fund it. Providers can compete to fulfill it. The task creator chooses who performs it. Teams can form around complex tasks.**

It restores the ability for ordinary people to turn useful work into an opportunity to earn, while allowing the community to decide what work deserves funding.

## Core Workflow

1.  **Create a Task:** A person posts a task (e.g., "Clean up this vacant lot — approximately 2 hours of work. Bounty: $150.").
    *   **Task Details Include:** Title, Description, Photos/video, Location, Desired completion date, Budget/bounty, Required skills, Required equipment, Number of workers, Special requirements, Required licenses/certifications.
2.  **Initial Offer:** The creator establishes an initial bounty (e.g., $150).
3.  **Bidding Mechanism:** Providers respond with offers. They can accept the bounty, make a counteroffer (higher or lower), propose a different date, or propose a team.
    *   *Evaluation:* The task creator evaluates offers based on a combination of Price + Rating + Experience + Verified Credentials + Proposed Completion Time + Team (not necessarily lowest price wins).
4.  **Task Fulfillment:** A provider/team is selected, the task is performed, submitted for verification, approved, and payment is released.

## Architecture

The application is built using Flutter for the frontend, communicating with Firebase (Firestore, Authentication, Storage, Functions).
*   **State Management:** Riverpod (`flutter_riverpod`, `riverpod_annotation`)
*   **Navigation:** GoRouter (`go_router`)
*   **Data Models:** Freezed (`freezed`, `json_serializable`) for immutable data structures.
*   **Backend:** Firebase Firestore (with custom security rules), Firebase Authentication.

### Key Models

*   **Task (`Task`):** Represents a job request. Includes budget, location, required credentials, status, and creator/provider IDs.
*   **UserProfile (`UserProfile`):** Represents a user on the platform. Includes skills, ratings, job counts, and basic info.
*   **Offer (`Offer`):** Represents a bid on a task. Includes amount, proposed dates, status, and attached credentials.
*   **Team/Guild (`Team`):** Represents a group of workers, historically called "Teams" but conceptualized in the UI as "Guilds" or "Factions". Supports branching logic:
    *   `parentIds`: Links a child Cliq back to its mother Organizations.
    *   `sisterClubIds`: Links sibling clubs.
    *   `treatyIds`: Connects independent Guilds in partnerships.
    *   Permissions are resolved dynamically: a member of a child Cliq implicitly inherits access permissions to the mother Organization.
*   **Membership (`Membership`):** Junction between a user and a team/guild, including roles.
*   **Credential (`Credential`):** Represents an uploaded license/certification, its verification status, and type.

### Task State Machine

A task progresses through defined states (enforced by `TaskStatus` enum):

`DRAFT` → `PUBLISHED` → `FUNDING_OPEN` → `BIDDING` → `PROVIDER_SELECTED` → `TEAM_FORMING` → `SCHEDULED` → `IN_PROGRESS` → `SUBMITTED_FOR_VERIFICATION` → `APPROVED` → `PAYMENT_RELEASED` → `COMPLETED`

Other possible states: `CANCELLED`, `DISPUTED`, `EXPIRED`.

This robust state machine allows the foundation to support many different types of services.

### License & Credential System

Tasks can specify required credentials (e.g., Driver's License, Electrical License, Insurance).
*   Providers upload documents to their profiles.
*   The system distinguishes between `Uploaded` and `Verified` credentials.
*   Verification requires platform admin approval.
*   Regulated services can require *verified* credentials before accepting tasks.

### "Official" Project Concept (Evolution of a Task)

For complex tasks (e.g., "Community Event — 300 People"), a task evolves into a project.
`TASK` → `PROJECT` → `ADMIN_SELECTED` → `REQUIREMENTS_MET` → `OFFICIAL_PROJECT`

An official project unlocks coordination tools: sub-teams (Security, Food, Cleanup), specific roles, budget tracking, and communications.

## Firebase Firestore Rules

Security is enforced via robust Firestore rules (`firestore.rules`):
*   **Users:** Read for signed-in users, update restricted to owner for specific fields.
*   **Tasks:** Public read for published tasks. Creation by owner. Updates restricted by fields and owner, or Admin.
*   **Bids:** Read for bidder and Admin (Creator read handled via Cloud Functions).
*   **Credentials:** Verification requires Admin role (`isAdmin()`).
*   **Teams/Memberships:** Public read, owner update. Admin capabilities.

## App Structure (Flutter)

The application features 5 major sections, managed by persistent bottom navigation:
1.  **Board (`/board`):** Feed of open tasks, nearby tasks, categories, and projects.
2.  **Create (`/create`):** Wizard to create tasks, gig bookings, and service requests.
3.  **Bids (`/bids`):** Manage offers received, own offers made, and active contracts.
4.  **Teams/Guilds (`/teams`):** Create/join guilds, manage credentials, offerings, and jobs. Features:
    *   **Offerings & Gig Booking:** Specialized listing system supporting direct **Booking / Gig Services** (e.g., DJs, audio engineers, event entertainers, technicians) with flexible billing periods (hourly, daily, weekly, monthly) and in-app communication channels (Direct Booking, Public Chat, or Team-only Chat).
    *   **Constellation Map (`/teams/constellation`):** An interactive 3D tactical planetary view rendered exclusively using **Fluonder** to visualize parent/child structures, sibling links, and treaty partnerships as glowing orbital nodes and great-circle data corridors.
5.  **Profile (`/profile`):** User skills, ratings, credentials, history, and earnings.

## Mapping & Planetary Visualization: Strictly Fluonder

> [!IMPORTANT]
> **TARGET PLATFORMS: ANDROID AND WEB ONLY**
> **MAPPING ENGINE: EXCLUSIVELY FLUONDER**
> All geospatial navigation, tactical radar surfaces, and 3D planetary views strictly use **Fluonder** (`FluonderGlobe`, `FluonderVectorMap`, `FluonderCoordinates`, and `package:fluoderpod`).
> External map engines (Google Maps, MapLibre, Cesium, or iframes) are strictly barred.

### Fluonder High-Fidelity Tactical Vector Globe Architecture
Fluonder delivers high-performance 60+ FPS tactical planetary rendering driven entirely by mathematical spherical geometry:
1.  **Spherical Catmull-Rom Splines:**
    *   Precomputes uniform cubic Catmull-Rom spline curves directly between 3D unit vectors in $\mathbb{R}^3$.
    *   Normalizes each interpolated point $\hat{V}(t) = \frac{V(t)}{\|V(t)\|}$ back onto the unit sphere $S^2$, producing organic $C^1$-smooth coastlines without dateline seams.
2.  **Limb Horizon Edge Clipping Math:**
    *   Calculates exact edge intersections with the horizon plane ($z = 0$ in camera space) at $t = \frac{z_1}{z_1 - z_2}$.
    *   Clips visible strokes cleanly at the planetary limb circle.
    *   Traces partially visible landmass fills along the circular limb arc between exit and entry intersections, completely eliminating ocean-crossing chords and front-face artifacts.
3.  **3D Planetary Shading & Lighting:**
    *   **Dual-Layer Atmosphere:** Rayleigh scattering halo ($1.0R$ to $1.32R$) and inner Mie forward-scattering neon-cyan limb ring.
    *   **Bathymetric Depth Gradient:** Multi-stop radial shader rendering deep ocean abyss, mid-ocean ridges, and continental shelves.
    *   **Real-Time Solar Terminator:** True astronomical subsolar positioning with specular solar glint and a warm copper/amber dusk aura ($3.0^\circ$ civil twilight band).
    *   **Night-Side Urban Lights:** Golden phosphor bloom clusters across 20 global metropolitan centers that automatically illuminate on the dark hemisphere.
    *   **Geodesic Grid Rings:** Parallels (Equator, Tropics, Arctic/Antarctic circles) and Meridians with Prime Meridian and foreshortened perspective curvature.
4.  **Orbital Drift & Multi-Level Zoom:**
    *   Continuous polar-axis orbital drift ($0.0012\text{ rad/tick}$) when idle.
    *   Smooth gesture zoom scaling from $0.6\times$ (orbital satellite view) up to $6.0\times$ (tactical street/sector recon) with live HUD telemetry.

## The Three Architecture Pillars

HeavenlyBond Lite / Game Maps IRL operates as part of a synchronized 3-pillar ecosystem:
1.  **Pillar 1: Client Application (This Repository)**
    *   Mobile (Android) and Web client, Riverpod state management, local SQLite parallel caching, task/gig marketplace, and team workflows.
2.  **Pillar 2: Fluorescent & Fluoderpod 3D Graphics Engine (`third_party/fluorescent`)**
    *   GPU-driven compute frustum/occlusion culling, Nanite-style virtual geometry, unified indirect draw pipeline, Android NDK Vulkan zero-copy bridge (`android_vulkan.rs`), and Web Wasm/WebGPU pipeline.
3.  **Pillar 3: TitheX / Fluoridian Blockchain & Nexus Protocol (`third_party/tithX`)**
    *   Parallel storage sync, decentralized event envelopes (`NetworkEvent`), `mobile-vault-sdk` Ed25519 signing, WebSocket gossip relay, double-entry internal ledger, and on-chain escrow settlement.

## Future: Nexus Protocol Ecosystem

HeavenlyBond Lite will integrate with the **Nexus Protocol** and the **Nexus Coin**. The Nexus Protocol is an edge-native compute substrate and decentralized marketplace for idle hardware, powered by a Proof of Health (PoH) consensus engine.

As the app ecosystem evolves into "Game Maps IRL", the native application will leverage the Nexus Protocol for:
*   **Mobile Sovereign Vaults & Identity Rails:** Utilizing hardware Secure Enclaves for managing local user context, credentials, and Decentralized Identifiers (DIDs).
*   **Distributed AI Compute:** Providing telemetry and participating in the decentralized marketplace for AI workload execution.
*   **Proof of Health Telemetry:** A future telemetry daemon will coordinate with the cluster to validate hardware state for consensus.
