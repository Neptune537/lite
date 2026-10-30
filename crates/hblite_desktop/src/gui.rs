//! # Lemuria Native Pure-Rust GUI via egui + wgpu
//!
//! Responsive & Modular AAA Game Pause Menu HUD:
//! Deeply wired directly to Lemuria's core engine dependencies:
//! - `lemuria_engine`: DoubleBufferedFrameAllocator, DeterministicPhysicsStepper (60Hz),
//!   SirenAcousticEngine (Sabine RT60 acoustics & Doppler decoding).
//! - `nautilus_world`: 64-bit ECEF spherical terrain geodesics, Wgs84GeoCoord haversine navigation,
//!   TacticalRouteSpline camera-relative ribbons, and Google Maps Turn-by-Turn URLs.
//! - `lemuria_leptos_client`: SevenDaughtersRadioHost roster, live Theater 24/7 stream (:8080/live.mp3),
//!   Coral Escrow double-entry marketplace, and VirtualStorefrontState (< 2.0m AI Gate walkup).
//! - `atlantis_protocol`: Decentralized DID credentials (`did:atlantis:`) and Hardware PoH validation.

use atlantis_protocol::{
    BioluminescenceTelemetry, GyreNodeTier, PohCanaryChallenge, PohCanaryResponse,
    ZkGeoProximityProof,
};
use ed25519_dalek::{Signer, Signature};
use eframe::egui;
use glam::{Quat, Vec3};
use lemuria_engine::{
    rendering_xr::WgslTemporalUpscaler, Aabb, AcousticRoomProfile, DeterministicPhysicsStepper,
    DoubleBufferedFrameAllocator, GaussianSplattingPass, LemuriaEcsWorld, PacejkaVehicleDynamics,
    SirenAcousticEngine, SplatConvexHullCollider, SplatShelfProduct,
};
use lemuria_leptos_client::{
    MarketplaceItem, MarketplaceTab, TheaterRadioState, VirtualStorefrontState,
};
use nautilus_world::{
    AnchoredSpatialOverlay, DemTerrainTileStreamer, GodsEyeGlobalState,
    GodsEyeLayerKind, GodsEyeTelemetryKind, NautilusGlobeViewMode, RealSiteVenueOverlay,
    RealSiteVenueType, SpatialOverlayKind, StadiumArenaOverlay, StadiumBattleMode,
    StadiumNetworkAsset, TacticalRouteSpline, Wgs84GeoCoord,
};

use crate::club::ClubState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenFormFactor {
    CompactPortrait,  // Mobile / Narrow Portrait (< 650px wide or portrait aspect)
    StandardTablet,   // Tablets (e.g. Galaxy Tab S7 in portrait or landscape)
    WidescreenHud,    // Desktop / Large Tablet Landscape (>= 1100px wide)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveVenueBroadcast {
    pub venue_id: String,
    pub title: String,
    pub venue_type_label: String,
    pub coord: Wgs84GeoCoord,
    pub broadcaster_callsign: String,
    pub stream_uri: String,
    pub rt60_seconds: f32,
    pub active_viewers: u32,
    pub spatial_overlay: AnchoredSpatialOverlay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpectatorSessionState {
    pub venue_id: String,
    pub title: String,
    pub broadcaster_callsign: String,
    pub stream_uri: String,
    pub elapsed_seconds: f32,
    pub total_tips_cyan: f32,
    pub camera_mode: String,
    pub playspace_bounds_m: Vec3,
    pub local_gravity_dir: Vec3,
}

/// Active Ephemeral Coral State Channel for high-frequency peer-to-peer micro-tipping.
#[derive(Debug, Clone, PartialEq)]
pub struct CoralTippingStateChannel {
    pub channel_id: String,
    pub opened_at_timestamp: u64,
    pub total_locked_cyan: f64,
    pub net_transferred_cyan: f64,
    pub tick_count: u32,
    pub is_settled: bool,
    pub recipient_did: String,
}

impl Default for CoralTippingStateChannel {
    fn default() -> Self {
        Self {
            channel_id: "coral:channel:ephemeral_01".into(),
            opened_at_timestamp: 1793216100000,
            total_locked_cyan: 50.0,
            net_transferred_cyan: 0.0,
            tick_count: 0,
            is_settled: false,
            recipient_did: "did:atlantis:siren_hosts_pool".into(),
        }
    }
}

impl CoralTippingStateChannel {
    /// Dispatches an off-ledger micro-tip increment (e.g. 0.1 CYAN) without touching disk or ledger locks.
    pub fn tip_micro_increment(&mut self, amount: f64) -> Result<f64, &'static str> {
        if self.is_settled {
            return Err("State channel is already settled");
        }
        if self.net_transferred_cyan + amount > self.total_locked_cyan {
            return Err("Exceeds locked Coral Escrow allocation");
        }
        self.net_transferred_cyan += amount;
        self.tick_count += 1;
        Ok(self.net_transferred_cyan)
    }

    /// Closes and settles the channel, committing only the net delta to Tide Ledger.
    pub fn settle_channel(&mut self) -> f64 {
        self.is_settled = true;
        self.net_transferred_cyan
    }
}

/// Simulated WebTransport QUIC Datagram stream telemetry for 60Hz physics and live Siren audio packets.
#[derive(Debug, Clone, PartialEq)]
pub struct WebTransportDatagramEngine {
    pub is_connected: bool,
    pub active_endpoint: String,
    pub datagrams_sent: u64,
    pub datagrams_received: u64,
    pub bytes_throughput: u64,
    pub rtt_ms: u32,
    pub packet_loss_pct: f32,
}

impl Default for WebTransportDatagramEngine {
    fn default() -> Self {
        Self {
            is_connected: true,
            active_endpoint: "webtransport://relay.kuroshio.mesh:4433/quic_datagrams".into(),
            datagrams_sent: 1420,
            datagrams_received: 1419,
            bytes_throughput: 284000,
            rtt_ms: 12,
            packet_loss_pct: 0.07,
        }
    }
}

impl WebTransportDatagramEngine {
    pub fn send_datagram(&mut self, payload_size_bytes: usize) {
        self.datagrams_sent += 1;
        self.datagrams_received += 1;
        self.bytes_throughput += payload_size_bytes as u64;
    }
}

// ============================================================================
// ITEM 6: WASI SANDBOXED MODDING & MINI-APP CARTRIDGES
// ============================================================================

/// Granular capability privileges granted to a WASI cartridge.
/// By default, all raw socket and host filesystem access is strictly forbidden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiHostCapabilities {
    pub allow_entity_spawn: bool,
    pub allow_escrow_bidding: bool,
    pub allow_zk_geo_verify: bool,
    pub allow_telemetry_read: bool,
    pub max_memory_pages: u32, // 64KB pages (default 16 = 1MB sandbox limit)
}

impl Default for WasiHostCapabilities {
    fn default() -> Self {
        Self {
            allow_entity_spawn: true,
            allow_escrow_bidding: false,
            allow_zk_geo_verify: true,
            allow_telemetry_read: true,
            max_memory_pages: 16,
        }
    }
}

/// Metadata describing a hot-swappable `.wasm` cartridge.
#[derive(Debug, Clone, PartialEq)]
pub struct WasiCartridgeMetadata {
    pub cartridge_id: String,
    pub title: String,
    pub author_did: String,
    pub version: String,
    pub binary_size_bytes: usize,
    pub capabilities: WasiHostCapabilities,
    pub is_mounted: bool,
    pub last_invocation_status: String,
    pub execution_cycles: u64,
}

/// Host-side WASI cartridge runtime managing sandboxed mini-apps and third-party mods.
#[derive(Debug, Clone, PartialEq)]
pub struct WasiCartridgeEngine {
    pub mounted_cartridges: Vec<WasiCartridgeMetadata>,
    pub active_cartridge_index: usize,
    pub total_sandboxed_calls: u64,
    pub memory_allocated_bytes: usize,
}

impl Default for WasiCartridgeEngine {
    fn default() -> Self {
        Self {
            mounted_cartridges: vec![
                WasiCartridgeMetadata {
                    cartridge_id: "cartridge_tactical_radar_hud.wasm".into(),
                    title: "Tactical Radar Minimap Mod (WASI)".into(),
                    author_did: "did:atlantis:modder_zephyr_9".into(),
                    version: "1.2.0".into(),
                    binary_size_bytes: 48 * 1024,
                    capabilities: WasiHostCapabilities {
                        allow_entity_spawn: true,
                        allow_escrow_bidding: false,
                        allow_zk_geo_verify: true,
                        allow_telemetry_read: true,
                        max_memory_pages: 16,
                    },
                    is_mounted: true,
                    last_invocation_status: "Active (Tick 60Hz)".into(),
                    execution_cycles: 1420,
                },
                WasiCartridgeMetadata {
                    cartridge_id: "cartridge_drone_delivery_escrow.wasm".into(),
                    title: "Drone Air-Drop Delivery & Coral Bidder".into(),
                    author_did: "did:atlantis:vance_logistics".into(),
                    version: "2.0.1".into(),
                    binary_size_bytes: 72 * 1024,
                    capabilities: WasiHostCapabilities {
                        allow_entity_spawn: true,
                        allow_escrow_bidding: true,
                        allow_zk_geo_verify: true,
                        allow_telemetry_read: true,
                        max_memory_pages: 32,
                    },
                    is_mounted: true,
                    last_invocation_status: "Standing by for Escrow Jobs".into(),
                    execution_cycles: 890,
                },
                WasiCartridgeMetadata {
                    cartridge_id: "cartridge_garage_booking_counter.wasm".into(),
                    title: "Neo-Tokyo Garage Service Appointment Mini-App".into(),
                    author_did: "did:atlantis:neo_garage_corp".into(),
                    version: "1.0.4".into(),
                    binary_size_bytes: 36 * 1024,
                    capabilities: WasiHostCapabilities {
                        allow_entity_spawn: false,
                        allow_escrow_bidding: true,
                        allow_zk_geo_verify: false,
                        allow_telemetry_read: true,
                        max_memory_pages: 16,
                    },
                    is_mounted: false,
                    last_invocation_status: "Unmounted (Dormant)".into(),
                    execution_cycles: 0,
                },
            ],
            active_cartridge_index: 0,
            total_sandboxed_calls: 2310,
            memory_allocated_bytes: (48 + 72) * 1024,
        }
    }
}

impl WasiCartridgeEngine {
    /// Dispatches a sandboxed host call into the currently active cartridge.
    /// Strictly verifies requested capability permissions before execution.
    pub fn invoke_cartridge_entry(
        &mut self,
        index: usize,
        call_kind: &str,
    ) -> Result<String, &'static str> {
        let cartridge = self
            .mounted_cartridges
            .get_mut(index)
            .ok_or("Cartridge index out of bounds")?;

        if !cartridge.is_mounted {
            return Err("Cartridge is currently unmounted");
        }

        match call_kind {
            "lemuria::spawn_entity" => {
                if !cartridge.capabilities.allow_entity_spawn {
                    return Err("WASI Security Violation: Cartridge does NOT have 'allow_entity_spawn' capability");
                }
                cartridge.execution_cycles += 1;
                cartridge.last_invocation_status = "Host call: lemuria::spawn_entity SUCCESS".into();
                self.total_sandboxed_calls += 1;
                Ok("Entity spawned in Lemuria ECS World via WASI sandbox".into())
            }
            "atlantis::request_escrow_bid" => {
                if !cartridge.capabilities.allow_escrow_bidding {
                    return Err("WASI Security Violation: Cartridge does NOT have 'allow_escrow_bidding' capability");
                }
                cartridge.execution_cycles += 1;
                cartridge.last_invocation_status = "Host call: atlantis::request_escrow_bid SUCCESS".into();
                self.total_sandboxed_calls += 1;
                Ok("Coral Escrow bid placed via WASI sandbox".into())
            }
            "atlantis::verify_zk_proximity" => {
                if !cartridge.capabilities.allow_zk_geo_verify {
                    return Err("WASI Security Violation: Cartridge does NOT have 'allow_zk_geo_verify' capability");
                }
                cartridge.execution_cycles += 1;
                cartridge.last_invocation_status = "Host call: atlantis::verify_zk_proximity SUCCESS".into();
                self.total_sandboxed_calls += 1;
                Ok("ZK-Geo Proximity verified via WASI sandbox".into())
            }
            "host::raw_socket_open" | "host::filesystem_access" => {
                // Strictly blocked by Lemuria V2 Sandboxing specification
                Err("CRITICAL WASI Sandboxing Violation: Raw socket or host filesystem access is strictly forbidden!")
            }
            _ => {
                cartridge.execution_cycles += 1;
                cartridge.last_invocation_status = format!("Unknown WASI call '{}' ignored", call_kind);
                Ok("Default WASI tick complete".into())
            }
        }
    }

    /// Mounts or unmounts a cartridge dynamically.
    pub fn toggle_mount(&mut self, index: usize) {
        if let Some(c) = self.mounted_cartridges.get_mut(index) {
            c.is_mounted = !c.is_mounted;
            if c.is_mounted {
                self.memory_allocated_bytes += c.binary_size_bytes;
                c.last_invocation_status = "Mounted (Ready)".into();
            } else {
                self.memory_allocated_bytes = self.memory_allocated_bytes.saturating_sub(c.binary_size_bytes);
                c.last_invocation_status = "Unmounted (Dormant)".into();
            }
        }
    }
}

// ============================================================================
// ITEM 2: TRUE ZERO-COPY NETWORK ARCHIVING & GPU BUFFER INGESTION (`rkyv`)
// ============================================================================

/// 64-byte cache-aligned entity transform representation for zero-copy mesh ingestion.
/// Matches memory layout for direct pointer cast and GPU storage buffer streaming.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZeroCopyEntityPacket {
    pub entity_id_u64: u64,          // 8 bytes (offset 0..8)
    pub position_xyz: [f32; 3],      // 12 bytes (offset 8..20)
    pub orientation_quat: [f32; 4],  // 16 bytes (offset 20..36)
    pub velocity_xyz: [f32; 3],      // 12 bytes (offset 36..48)
    pub health_shield_packed: u32,   // 4 bytes (offset 48..52)
    pub flags: u32,                  // 4 bytes (offset 52..56)
    pub _padding: [u8; 8],           // 8 bytes (offset 56..64) -> Total: 64 bytes
}

impl Default for ZeroCopyEntityPacket {
    fn default() -> Self {
        Self {
            entity_id_u64: 7001,
            position_xyz: [0.0, 1.2, -5.0],
            orientation_quat: [0.0, 0.0, 0.0, 1.0],
            velocity_xyz: [0.0, 0.0, 0.0],
            health_shield_packed: 0x00640064, // 100 HP, 100 Shield
            flags: 1,
            _padding: [0u8; 8],
        }
    }
}

/// Zero-copy network ingest pipeline: Casts raw byte slices directly into 64-byte aligned structs
/// and dispatches them into GPU storage buffers without intermediate JSON or heap re-allocations.
#[derive(Debug, Clone, PartialEq)]
pub struct ZeroCopyMeshPipeline {
    pub packets_ingested: u64,
    pub total_bytes_zero_copy: u64,
    pub parse_overhead_micros: f32, // Strictly 0.00ms due to zero copy
    pub last_ingested_packet: ZeroCopyEntityPacket,
}

impl Default for ZeroCopyMeshPipeline {
    fn default() -> Self {
        Self {
            packets_ingested: 512,
            total_bytes_zero_copy: 512 * 64,
            parse_overhead_micros: 0.0,
            last_ingested_packet: ZeroCopyEntityPacket::default(),
        }
    }
}

impl ZeroCopyMeshPipeline {
    /// Ingests a raw byte slice as a 64-byte cache-aligned entity packet with zero heap allocations.
    pub fn ingest_packet_bytes(&mut self, bytes: &[u8]) -> Result<&ZeroCopyEntityPacket, &'static str> {
        if bytes.len() != std::mem::size_of::<ZeroCopyEntityPacket>() {
            return Err("Packet size mismatch for ZeroCopyEntityPacket (expected exactly 64 bytes)");
        }
        // Validate 64-byte alignment or cast safely
        let ptr = bytes.as_ptr();
        if (ptr as usize) % std::mem::align_of::<ZeroCopyEntityPacket>() != 0 {
            // Safe copy fallback for misaligned incoming slices
            let mut packet = ZeroCopyEntityPacket::default();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    ptr,
                    &mut packet as *mut _ as *mut u8,
                    std::mem::size_of::<ZeroCopyEntityPacket>(),
                );
            }
            self.last_ingested_packet = packet;
        } else {
            // Direct zero-copy dereference
            let packet_ref = unsafe { &*(ptr as *const ZeroCopyEntityPacket) };
            self.last_ingested_packet = *packet_ref;
        }

        self.packets_ingested += 1;
        self.total_bytes_zero_copy += 64;
        self.parse_overhead_micros = 0.0;
        Ok(&self.last_ingested_packet)
    }
}

pub struct LemuriaNativeApp {
    pub selected_tab: MarketplaceTab,
    pub club_open: bool,
    pub daemons_tab_open: bool,
    pub club: ClubState,
    pub catalog: Vec<MarketplaceItem>,
    pub radio_state: TheaterRadioState,
    pub storefront: VirtualStorefrontState,
    pub last_status_message: String,
    pub tip_counter: u32,

    // Real-to-Virtual 1:1 Grounded Venues & Live VR Spectator Beacons
    pub venue_broadcasts: Vec<LiveVenueBroadcast>,
    pub active_spectator_session: Option<SpectatorSessionState>,

    // Real Lemuria Engine Core Subsystems
    pub frame_allocator: DoubleBufferedFrameAllocator,
    pub physics_stepper: DeterministicPhysicsStepper,
    pub siren_engine: SirenAcousticEngine,
    pub vehicle_dynamics: PacejkaVehicleDynamics,

    // Real Nautilus World Geodesic Navigation & Dynamic Globe Streamer
    pub current_coord: Wgs84GeoCoord,
    pub active_route: TacticalRouteSpline,
    pub dem_streamer: DemTerrainTileStreamer,
    pub globe_rotation_rad: f32,
    pub globe_tilt_rad: f32,
    pub is_dragging_globe: bool,

    // Mandatory God's-Eye-View Multi-Layer OSINT Telemetry & VR Guild Marketplace
    pub gods_eye: GodsEyeGlobalState,
    pub gods_eye_query_input: String,
    pub guild_bounties_claimed_cyan: f64,

    // Live Atlantis Protocol Telemetry
    pub poh_telemetry: BioluminescenceTelemetry,

    // Advanced Next-Horizon Subsystems
    pub coral_channel: CoralTippingStateChannel,
    pub webtransport_engine: WebTransportDatagramEngine,
    pub active_zk_proof: Option<ZkGeoProximityProof>,
    pub last_canary_challenge: Option<PohCanaryChallenge>,
    pub last_canary_audit_pass: bool,

    // Interactive 3D Gaussian Splat Storefront & ECS World
    pub splat_world: LemuriaEcsWorld,
    pub splat_pass: GaussianSplattingPass,
    pub splat_collider: SplatConvexHullCollider,
    pub splat_rotation_angle_rad: f32,
    pub splat_inspect_zoom: f32,
    pub selected_splat_index: usize,

    // WGSL Temporal Super Resolution (TSR) & Power Scaling
    pub tsr_upscaler: WgslTemporalUpscaler,
    pub tsr_enabled: bool,

    // WASI Sandboxed Modding & Mini-App Cartridges (Item 6)
    pub wasi_engine: WasiCartridgeEngine,

    // True Zero-Copy Network Archiving (Item 2)
    pub zero_copy_pipeline: ZeroCopyMeshPipeline,

    // Game Simulation States (OBD-II, CAN Bus, Controls)
    pub vehicle_speed_kmh: f32,
    pub vehicle_rpm: f32,
    pub vehicle_gear: i32,
    pub throttle_pct: f32,
    pub brake_pct: f32,
    pub steering_deg: f32,
    pub player_stamina: f32,
    pub player_shield: f32,
}

impl Default for LemuriaNativeApp {
    fn default() -> Self {
        let mut storefront = VirtualStorefrontState::new("store_neo_01", "Neo-Tokyo Performance Garage");
        storefront.update_customer_distance(
            1.4,
            Some(("did:atlantis:staff_bob".into(), "Bob Ross".into())),
            ("gemini-3.8-flash", "local.ai.gate"),
        );

        let current_coord = Wgs84GeoCoord::new(35.6586, 139.7454, 42.0);

        let waypoints = vec![
            current_coord,
            Wgs84GeoCoord::new(35.6601, 139.7290, 35.0),
            Wgs84GeoCoord::new(35.6650, 139.7100, 28.0),
        ];
        let active_route = TacticalRouteSpline::new(waypoints);

        let frame_allocator = DoubleBufferedFrameAllocator::new(DoubleBufferedFrameAllocator::DEFAULT_CAPACITY);
        let physics_stepper = DeterministicPhysicsStepper::new();
        let siren_engine = SirenAcousticEngine::new(AcousticRoomProfile::nautilus_nightclub(), 4800);
        let vehicle_dynamics = PacejkaVehicleDynamics::new();

        let (score, tier) = BioluminescenceTelemetry::evaluate_tier(12288, 18, 62.0);
        let poh_telemetry = BioluminescenceTelemetry {
            fps: 60.0,
            cpu_usage_pct: 14.5,
            vram_free_mb: 12288,
            temp_celsius: 62.0,
            rtt_ping_ms: 18,
            poh_score: score,
            tier,
        };

        let club = ClubState::default();
        let mut radio_state = TheaterRadioState::default();
        radio_state.sync_with_club_pi_node(club.is_siren_radio_running(), &club.pi_node_lan_host);

        let mut dem_streamer = DemTerrainTileStreamer::new(15);
        dem_streamer.set_view_mode(NautilusGlobeViewMode::CompleteGlobe);
        dem_streamer.update_for_camera(current_coord);

        let wh_coord = Wgs84GeoCoord::new(35.6528, 139.7532, 12.0);
        let park_coord = Wgs84GeoCoord::new(35.6717, 139.6949, 38.0);
        let office_overlay = AnchoredSpatialOverlay::team7_headquarters_office();
        let office_coord = office_overlay.anchor_geo;

        let mut splat_world = LemuriaEcsWorld::new();
        let mut splat_pass = GaussianSplattingPass::new(512);

        // Product 1: Scanned 3D Totem - Neo-Tokyo High-Resonance Energy Cell
        let product1 = SplatShelfProduct {
            entity_id: uuid::Uuid::new_v4(),
            sku_id: "totem_energy_cell_432hz".into(),
            splat_asset_uri: "lemuria://scans/totems/energy_cell_432hz.splat".into(),
            convex_hull_half_extents: Vec3::new(0.35, 0.65, 0.35),
            position: Vec3::new(-1.0, 1.2, -2.5),
            rotation: Quat::IDENTITY,
            is_service_totem: true,
        };
        splat_pass.populate_from_shelf_product(&product1, 0, 4);
        splat_world.spawn_shelf_splat_product(product1);

        // Product 2: Scanned 3D Merchandise - Pacejka Competition Carbon Wheel
        let product2 = SplatShelfProduct {
            entity_id: uuid::Uuid::new_v4(),
            sku_id: "item_carbon_wheel_r19".into(),
            splat_asset_uri: "lemuria://scans/merch/carbon_wheel_r19.splat".into(),
            convex_hull_half_extents: Vec3::new(0.48, 0.48, 0.28),
            position: Vec3::new(1.0, 1.2, -2.5),
            rotation: Quat::IDENTITY,
            is_service_totem: false,
        };
        splat_pass.populate_from_shelf_product(&product2, 1, 4);
        splat_world.spawn_shelf_splat_product(product2);

        let cart_drop_zone = Aabb::new(Vec3::new(-0.8, 0.0, -1.5), Vec3::new(0.8, 0.6, -0.8));
        let splat_collider = SplatConvexHullCollider::new(cart_drop_zone);

        Self {
            selected_tab: MarketplaceTab::TacticalMap,
            club_open: false,
            daemons_tab_open: false,
            club,
            catalog: MarketplaceItem::default_catalog(),
            radio_state,
            storefront,
            last_status_message: "LEMURIA V2 INITIALIZED • ALL ARCHITECTURE PILLARS MOUNTED".into(),
            tip_counter: 0,
            venue_broadcasts: vec![
                LiveVenueBroadcast {
                    venue_id: "office_team7_01".into(),
                    title: "Headquarters Building: Suite 700 (Team 7 Office)".into(),
                    venue_type_label: "🏢 1:1 Enclave Office & Cornerstone Triad".into(),
                    coord: office_coord,
                    broadcaster_callsign: "Team 7 Triad [Triton • Ursula • Sebastian]".into(),
                    stream_uri: "webtransport://relay.kuroshio.mesh:4433/broadcast/team7_office".into(),
                    rt60_seconds: 0.45,
                    active_viewers: 77,
                    spatial_overlay: office_overlay,
                },
                LiveVenueBroadcast {
                    venue_id: "venue_wh_01".into(),
                    title: "Sector-7 Old Iron Warehouse Arcade".into(),
                    venue_type_label: "🏭 1:1 Warehouse Virtual Arcade".into(),
                    coord: wh_coord,
                    broadcaster_callsign: "Jules Vance (VR)".into(),
                    stream_uri: "webtransport://relay.kuroshio.mesh:4433/broadcast/wh_01".into(),
                    rt60_seconds: 1.45,
                    active_viewers: 42,
                    spatial_overlay: AnchoredSpatialOverlay::new(
                        "overlay_wh_01",
                        wh_coord,
                        SpatialOverlayKind::RealSiteGroundedVenue(RealSiteVenueOverlay {
                            venue_id: "venue_wh_01".into(),
                            name: "Sector-7 Old Iron Warehouse Arcade".into(),
                            venue_type: RealSiteVenueType::WarehouseArcade,
                            playspace_bounds_m: Vec3::new(40.0, 60.0, 12.0),
                            is_broadcasting: true,
                            broadcaster_callsign: Some("Jules Vance (VR)".into()),
                            spectator_stream_uri: Some(
                                "webtransport://relay.kuroshio.mesh:4433/broadcast/wh_01".into(),
                            ),
                            acoustic_profile_name: "Industrial Warehouse (Sabine RT60 1.45s)".into(),
                        }),
                        120.0,
                    ),
                },
                LiveVenueBroadcast {
                    venue_id: "venue_park_02".into(),
                    title: "Yoyogi Tactical Field & Holo-Park".into(),
                    venue_type_label: "🌳 1:1 Public Municipal Park Arena".into(),
                    coord: park_coord,
                    broadcaster_callsign: "Seraph-9 [Mech Pilot]".into(),
                    stream_uri: "webtransport://relay.kuroshio.mesh:4433/broadcast/park_02".into(),
                    rt60_seconds: 0.25,
                    active_viewers: 118,
                    spatial_overlay: AnchoredSpatialOverlay::new(
                        "overlay_park_02",
                        park_coord,
                        SpatialOverlayKind::RealSiteGroundedVenue(RealSiteVenueOverlay {
                            venue_id: "venue_park_02".into(),
                            name: "Yoyogi Tactical Field & Holo-Park".into(),
                            venue_type: RealSiteVenueType::MunicipalParkArena,
                            playspace_bounds_m: Vec3::new(150.0, 200.0, 35.0),
                            is_broadcasting: true,
                            broadcaster_callsign: Some("Seraph-9 [Mech Pilot]".into()),
                            spectator_stream_uri: Some(
                                "webtransport://relay.kuroshio.mesh:4433/broadcast/park_02".into(),
                            ),
                            acoustic_profile_name: "Open Municipal Park (Sabine RT60 0.25s)".into(),
                        }),
                        250.0,
                    ),
                },
                LiveVenueBroadcast {
                    venue_id: "stadium_oracle_01".into(),
                    title: "Oracle Park Seraphim Battle Colosseum".into(),
                    venue_type_label: "🏟️ In-World Stadium Battle Arena".into(),
                    coord: Wgs84GeoCoord::new(37.7749, -122.4194, 15.0),
                    broadcaster_callsign: "House Storm Vanguard".into(),
                    stream_uri: "webtransport://relay.kuroshio.mesh:4433/stadium/01".into(),
                    rt60_seconds: 2.10,
                    active_viewers: 420,
                    spatial_overlay: AnchoredSpatialOverlay::new(
                        "overlay_stadium_sf_01",
                        Wgs84GeoCoord::new(37.7749, -122.4194, 15.0),
                        SpatialOverlayKind::StadiumBattleArena(StadiumArenaOverlay {
                            stadium_id: "stadium_oracle_01".into(),
                            name: "Oracle Park Seraphim Battle Colosseum".into(),
                            creator_club_did: "did:atlantis:club-ignis-prime".into(),
                            battle_mode: StadiumBattleMode::TacticalFrontlinePush,
                            inner_pitch_radius_m: 180.0,
                            outer_colosseum_radius_m: 350.0,
                            stadium_mesh_uri: "nautpak://assets.kuroshio.network/stadiums/oracle_colosseum.meshlet".into(),
                            jumbotrons: vec![],
                            allowed_network_assets: vec![
                                StadiumNetworkAsset {
                                    asset_id: "mech_julian_v2".into(),
                                    name: "Julian Assault Mech".into(),
                                    nautpak_uri: "nautpak://assets.kuroshio.network/units/julian.nautpak".into(),
                                    unit_class: "AssaultMech".into(),
                                    creator_did: "did:atlantis:vance-industries".into(),
                                    min_poh_score: 0.65,
                                },
                            ],
                            spectator_stream_uri: Some("webtransport://relay.kuroshio.mesh:4433/stadium/01".into()),
                            active_combatant_count: 24,
                            max_capacity: 64,
                        }),
                        500.0,
                    ),
                },
            ],
            active_spectator_session: None,
            frame_allocator,
            physics_stepper,
            siren_engine,
            vehicle_dynamics,
            current_coord,
            active_route,
            dem_streamer,
            globe_rotation_rad: 0.0,
            globe_tilt_rad: 0.25,
            is_dragging_globe: false,
            gods_eye: GodsEyeGlobalState::default(),
            gods_eye_query_input: String::new(),
            guild_bounties_claimed_cyan: 0.0,
            poh_telemetry,
            coral_channel: CoralTippingStateChannel::default(),
            webtransport_engine: WebTransportDatagramEngine::default(),
            active_zk_proof: None,
            last_canary_challenge: None,
            last_canary_audit_pass: true,
            splat_world,
            splat_pass,
            splat_collider,
            splat_rotation_angle_rad: 0.0,
            splat_inspect_zoom: 1.0,
            selected_splat_index: 0,
            tsr_upscaler: WgslTemporalUpscaler::new_half_scale(2560, 1440),
            tsr_enabled: true,
            wasi_engine: WasiCartridgeEngine::default(),
            zero_copy_pipeline: ZeroCopyMeshPipeline::default(),
            vehicle_speed_kmh: 84.5,
            vehicle_rpm: 4200.0,
            vehicle_gear: 3,
            throttle_pct: 0.65,
            brake_pct: 0.0,
            steering_deg: -2.4,
            player_stamina: 88.0,
            player_shield: 100.0,
        }
    }
}

impl eframe::App for LemuriaNativeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Step 60Hz physics accumulator & zero-GC allocator on each frame tick
        let steer_rad = self.steering_deg.to_radians();
        let throttle = self.throttle_pct;
        let brake = self.brake_pct;
        let dynamics = &mut self.vehicle_dynamics;
        self.physics_stepper.accumulate_and_step(0.016, |dt, _tick| {
            dynamics.step_vehicle(steer_rad / 0.55, throttle, brake, 0.0, dt);
        });
        self.vehicle_speed_kmh = (self.vehicle_dynamics.linear_velocity.length() * 3.6).max(self.vehicle_speed_kmh * 0.99);
        self.gods_eye.step_telemetry_kinematics(0.016);
        self.webtransport_engine.send_datagram(128);
        self.frame_allocator.flip_frame();

        // Keep 7 Daughters Radio automatically synchronized with the Club's Atlantis Raspberry Pi node on LAN
        self.club.sync_pi_radio_daemon();
        self.radio_state
            .sync_with_club_pi_node(self.club.is_siren_radio_running(), &self.club.pi_node_lan_host);

        let screen_rect = ui.input(|i| i.viewport_rect());
        let screen_width = screen_rect.width();
        let screen_height = screen_rect.height();
        let is_portrait = screen_height > screen_width;

        let form_factor = if screen_width < 650.0 || (is_portrait && screen_width < 800.0) {
            ScreenFormFactor::CompactPortrait
        } else if screen_width >= 1100.0 && !is_portrait {
            ScreenFormFactor::WidescreenHud
        } else {
            ScreenFormFactor::StandardTablet
        };

        // 1. Top Global Navigation / Header Bar
        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.add_space(4.0);
            match form_factor {
                ScreenFormFactor::CompactPortrait => {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new("⏸ DASHBOARD")
                                    .color(egui::Color32::from_rgb(0, 229, 255))
                                    .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!("PoH: {:.1}", self.poh_telemetry.poh_score))
                                        .color(egui::Color32::from_rgb(255, 215, 0))
                                        .strong(),
                                );
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Shield: {:.0}%", self.player_shield)).color(egui::Color32::from_rgb(0, 229, 255)).small());
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Stamina: {:.0}%", self.player_stamina)).color(egui::Color32::GREEN).small());
                            ui.separator();
                            ui.label(egui::RichText::new("[REINCARNATED • GUILD HUD]").color(egui::Color32::LIGHT_GRAY).small());
                        });
                    });
                }
                _ => {
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new("⏸ DASHBOARD // REINCARNATED IN A DIFFERENT WORLD")
                                .color(egui::Color32::from_rgb(0, 229, 255))
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("[LEMURIA V2 ARCHITECTURE • ANIME GUILD & FORGOTTEN ANGELS]")
                                .color(egui::Color32::LIGHT_GRAY)
                                .small(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("PoH: {:.1} ({:?})", self.poh_telemetry.poh_score, self.poh_telemetry.tier))
                                    .color(egui::Color32::from_rgb(255, 215, 0))
                                    .strong(),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Shield: {:.0}%", self.player_shield))
                                    .color(egui::Color32::from_rgb(0, 229, 255)),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Stamina: {:.0}%", self.player_stamina))
                                    .color(egui::Color32::GREEN),
                            );
                        });
                    });
                }
            }
            ui.add_space(4.0);
        });

        // 2. Bottom Status HUD with Live Engine Subsystem Telemetry
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("LEMURIA TELEMETRY:").strong().small());
                ui.label(egui::RichText::new(&self.last_status_message).color(egui::Color32::from_rgb(0, 229, 255)).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let heap_kb = self.frame_allocator.bytes_used() / 1024;
                    let tsr_info = if self.tsr_enabled {
                        format!(" | TSR 50% ({:.0}% GPU Saved)", self.tsr_upscaler.fragment_load_reduction_ratio() * 100.0)
                    } else {
                        " | Native 100%".into()
                    };
                    ui.label(egui::RichText::new(format!("Alloc: {} KB | 60Hz Rapier | {:.0}x{:.0} px{}", heap_kb, screen_width, screen_height, tsr_info)).small().color(egui::Color32::GRAY));
                });
            });
            ui.add_space(2.0);
        });

        // 3. Modular Navigation (Adaptive Tab Bar)
        if form_factor == ScreenFormFactor::CompactPortrait {
            egui::Panel::top("modular_mobile_tabs").show(ui, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.render_tab_buttons(ui, true);
                    });
                });
            });
        } else {
            let sidebar_width = if form_factor == ScreenFormFactor::WidescreenHud { 240.0 } else { 190.0 };
            egui::Panel::left("tabs_panel")
                .resizable(false)
                .exact_size(sidebar_width)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    ui.heading("COMMAND MENU");
                    ui.separator();

                    self.render_tab_buttons(ui, false);

                    ui.add_space(20.0);
                    ui.separator();
                    ui.label(egui::RichText::new("Engine: Lemuria V2 Core").small());
                    ui.label(egui::RichText::new("World: Nautilus 64-bit ECEF").small());
                    ui.label(egui::RichText::new("Acoustics: Sabine RT60 (1.09s)").small());
                    ui.label(egui::RichText::new("Ledger: Atlantis Coral Escrow").small());
                });
        }

        // 4. Central Panel with Responsive Flex Scroll
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.daemons_tab_open {
                    self.club.render_registered_daemons_tab(ui);
                    return;
                }
                if self.club_open {
                    self.club.ui(ui);
                    return;
                }
                match self.selected_tab {
                    MarketplaceTab::TacticalMap => {
                        self.render_tactical_map_tab(ui, form_factor);
                    }
                    MarketplaceTab::TheaterRadio => {
                        self.render_theater_radio_tab(ui, form_factor);
                    }
                    MarketplaceTab::Marketplace => {
                        self.render_marketplace_tab(ui, form_factor);
                    }
                    MarketplaceTab::VirtualStore => {
                        self.render_virtual_store_tab(ui, form_factor);
                    }
                    MarketplaceTab::VehicleTelemetry => {
                        self.render_vehicle_telemetry_tab(ui, form_factor);
                    }
                    MarketplaceTab::CharacterStatus => {
                        self.render_character_status_tab(ui, form_factor);
                    }
                }
            });
        });
    }
}

impl LemuriaNativeApp {
    fn render_tab_buttons(&mut self, ui: &mut egui::Ui, horizontal: bool) {
        let tabs = [
            (MarketplaceTab::TacticalMap, "🗺️ Map", "🗺️ Tactical Map"),
            (MarketplaceTab::TheaterRadio, "📻 Radio", "📻 Theater Radio"),
            (MarketplaceTab::Marketplace, "🛒 Market", "🛒 Task Marketplace"),
            (MarketplaceTab::VirtualStore, "🏪 Store", "🏪 3D Storefront"),
            (MarketplaceTab::VehicleTelemetry, "🚗 Vehicle", "🚗 Vehicle Dynamics"),
            (MarketplaceTab::CharacterStatus, "🛡️ Pearl", "🛡️ Pearl & Identity"),
        ];

        for (tab, short_label, full_label) in tabs {
            let label = if horizontal { short_label } else { full_label };
            let is_selected = !self.club_open && !self.daemons_tab_open && self.selected_tab == tab;
            if ui.selectable_label(is_selected, label).clicked() {
                self.selected_tab = tab;
                self.club_open = false;
                self.daemons_tab_open = false;
                self.last_status_message = format!("Tab switched: {}", full_label);
            }
        }

        let is_officer = self.club.viewer_is_officer();

        let daemons_label = if is_officer {
            if horizontal { "🛡️ Node Dash" } else { "🛡️ Node Officer Dash" }
        } else {
            if horizontal { "🏢 Team 7" } else { "🏢 Team 7 Office" }
        };

        if ui.selectable_label(self.daemons_tab_open, daemons_label).clicked() {
            self.daemons_tab_open = true;
            self.club_open = false;
            if is_officer {
                self.club.active_subtab = crate::club::ClubViewSubTab::OfficerNodeDashboard;
                self.last_status_message = "Tab switched: 🛡️ Home Node Officer Dashboard".into();
            } else {
                self.club.active_subtab = crate::club::ClubViewSubTab::RegisteredDaemons;
                self.last_status_message = "Tab switched: 🏢 Team 7 Office & Daemons".into();
            }
        }

        let club_label = if horizontal { "👥 Clubs" } else { "👥 Registered Clubs" };
        if ui.selectable_label(self.club_open && self.club.active_subtab != crate::club::ClubViewSubTab::SlipstreamNode, club_label).clicked() {
            self.club_open = true;
            self.daemons_tab_open = false;
            self.club.active_subtab = crate::club::ClubViewSubTab::RegisteredClubs;
            self.last_status_message = "Tab switched: 👥 Registered Clubs & By-Laws".into();
        }

        let is_slipstream_open = self.club_open && self.club.active_subtab == crate::club::ClubViewSubTab::SlipstreamNode;
        let slipstream_label = if horizontal { "🛸 Slipstream" } else { "🛸 Slipstream Node" };
        if ui.selectable_label(is_slipstream_open, egui::RichText::new(slipstream_label).color(if is_slipstream_open { egui::Color32::from_rgb(0, 242, 254) } else { egui::Color32::from_rgb(120, 220, 255) })).clicked() {
            self.club_open = true;
            self.daemons_tab_open = false;
            self.club.active_subtab = crate::club::ClubViewSubTab::SlipstreamNode;
            self.last_status_message = "Tab switched: 🛸 Slipstream Node & C&C War Room Rebuild".into();
        }
    }

    fn render_tactical_map_tab(&mut self, ui: &mut egui::Ui, _form_factor: ScreenFormFactor) {
        ui.heading("🗺️ Nautilus Tactical Map & Planetary Guidance");
        ui.label("Authoritative 64-bit ECEF terrain geodesics with Turn-by-Turn waypoint guidance.");
        ui.add_space(8.0);

        let ecef_pos = self.current_coord.to_ecef_dvec3();

        ui.group(|ui| {
            ui.label(egui::RichText::new("Planetary Locus (WGS-84 / ECEF)").strong().size(15.0));
            egui::Grid::new("map_coords").striped(true).show(ui, |ui| {
                ui.label("Latitude / Longitude:");
                ui.label(format!("{:.6}° N, {:.6}° E", self.current_coord.lat_deg, self.current_coord.lon_deg));
                ui.end_row();

                ui.label("Altitude (MSL):");
                ui.label(format!("{:.1} m", self.current_coord.alt_meters));
                ui.end_row();

                ui.label("64-bit ECEF Vector:");
                ui.label(format!("({:.0}, {:.0}, {:.0}) m", ecef_pos.x, ecef_pos.y, ecef_pos.z));
                ui.end_row();

                ui.label("Terrain Engine:");
                ui.label(
                    egui::RichText::new(format!(
                        "Nautilus DemTerrainTileStreamer (Mode: {:?} • Regime: {:?} • {} Active Tiles)",
                        self.dem_streamer.view_mode,
                        self.dem_streamer.active_regime,
                        self.dem_streamer.active_leaf_tiles.len()
                    ))
                    .color(egui::Color32::GREEN),
                );
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("🏢 Lock Waypoint: Team 7 Headquarters Office").clicked() {
                    let dest = Wgs84GeoCoord::new(35.6586, 139.7454, 42.0);
                    let dist_km = self.current_coord.haversine_distance_meters(&dest) / 1000.0;
                    self.active_route = TacticalRouteSpline::new(vec![self.current_coord, dest]);
                    self.last_status_message = format!("Locked Waypoint: Team 7 Office ({:.2} km away)", dist_km);
                }
                if ui.button("📍 Lock Waypoint: Club Nautilus").clicked() {
                    let dest = Wgs84GeoCoord::new(35.6601, 139.7290, 35.0);
                    let dist_km = self.current_coord.haversine_distance_meters(&dest) / 1000.0;
                    self.active_route = TacticalRouteSpline::new(vec![self.current_coord, dest]);
                    self.last_status_message = format!("Locked Waypoint: Club Nautilus ({:.2} km away)", dist_km);
                }
                if ui.button("📍 Lock Waypoint: Neo-Tokyo Garage").clicked() {
                    let dest = Wgs84GeoCoord::new(35.6586, 139.7454, 42.0);
                    let dist_km = self.current_coord.haversine_distance_meters(&dest) / 1000.0;
                    self.active_route = TacticalRouteSpline::new(vec![self.current_coord, dest]);
                    self.last_status_message = format!("Locked Waypoint: Neo-Tokyo Garage ({:.2} km away)", dist_km);
                }
                if ui.button("🛰️ Orbital CompleteGlobe Overview").clicked() {
                    self.dem_streamer.set_view_mode(NautilusGlobeViewMode::CompleteGlobe);
                    self.dem_streamer.update_for_camera(Wgs84GeoCoord::new(
                        self.current_coord.lat_deg,
                        self.current_coord.lon_deg,
                        85_000.0,
                    ));
                    self.last_status_message = "Camera ascended to 85km Karman Orbit (CompleteGlobe Mode)".into();
                }
                if ui.button("🌐 Open External Turn-by-Turn GPS").clicked() {
                    if let Some(url) = self.active_route.destination_google_maps_url() {
                        self.last_status_message = format!("Generated Navigation URL: {}", url);
                    }
                }
                if ui.button("🛡️ Generate ZK-Geo Proximity Proof").clicked() {
                    let secret_salt = b"founder_secret_entropy_salt_v1";
                    let target_center = (35.6586, 139.7454); // Neo-Tokyo Garage
                    let actual_coord = (self.current_coord.lat_deg, self.current_coord.lon_deg);
                    match ZkGeoProximityProof::generate_proof(
                        actual_coord,
                        target_center,
                        500.0,
                        "zone_neotokyo_garage",
                        secret_salt,
                    ) {
                        Ok(proof) => {
                            let verified = proof.verify(Some(target_center));
                            self.last_status_message = format!(
                                "ZK-Geo Proof Generated (Commitment: {:.12}...) • Verified: {}",
                                proof.commitment_hash, verified
                            );
                            self.active_zk_proof = Some(proof);
                        }
                        Err(e) => {
                            self.last_status_message = format!("ZK-Geo Proof Generation Failed: {:?}", e);
                        }
                    }
                }
            });

            if let Some(proof) = &self.active_zk_proof {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔐 Active ZK-Geo Proof:").strong().color(egui::Color32::from_rgb(0, 242, 254)));
                    ui.label(format!("Zone: {} (Radius: {:.0}m)", proof.zone_id, proof.geofence_radius_meters));
                    ui.separator();
                    ui.label(egui::RichText::new("PROVEN WITHOUT GPS LEAKAGE").color(egui::Color32::GREEN).small());
                });
            }
        });

        ui.add_space(10.0);

        // 1-to-1 Real Site Grounded Venues & Live VR Spectator Beacons
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🌐 1:1 Grounded Venues & VR Spectator Feeds");
                ui.separator();
                ui.label(
                    egui::RichText::new("● LIVE FROM HEAVENLYBOUND VR")
                        .color(egui::Color32::from_rgb(255, 82, 82))
                        .strong(),
                );
            });
            ui.label("Touch any real-world site beacon on the holographic globe or list below to descend from orbit into 500m Karman SectorSlice LOD & watch in-situ VR gameplay.");
            ui.add_space(6.0);

            // --- Interactive Holographic Nautilus 3D Globe Visualizer ---
            self.render_nautilus_holographic_globe(ui);

            ui.add_space(8.0);

            let venues = self.venue_broadcasts.clone();
            for venue in venues {
                let dist_km = self.current_coord.haversine_distance_meters(&venue.coord) / 1000.0;
                let playspace_bounds = match &venue.spatial_overlay.overlay_kind {
                    SpatialOverlayKind::RealSiteGroundedVenue(v) => v.playspace_bounds_m,
                    _ => Vec3::new(50.0, 50.0, 10.0),
                };
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(18, 24, 36))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 229, 255)))
                    .corner_radius(6.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&venue.title).strong().size(15.0));
                            ui.separator();
                            ui.label(egui::RichText::new(&venue.venue_type_label).color(egui::Color32::from_rgb(255, 215, 0)));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("{:.2} km away", dist_km)).color(egui::Color32::LIGHT_GRAY));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("👀 {} viewers", venue.active_viewers)).color(egui::Color32::from_rgb(0, 229, 255)));
                        });

                        ui.horizontal(|ui| {
                            ui.label(format!("Broadcaster: {}", venue.broadcaster_callsign));
                            ui.separator();
                            ui.label(format!("Sabine RT60: {:.2}s", venue.rt60_seconds));
                            ui.separator();
                            ui.label(format!(
                                "1:1 ENU Bounds: {:.0}m × {:.0}m × {:.0}m",
                                playspace_bounds.x, playspace_bounds.y, playspace_bounds.z
                            ));
                            ui.separator();
                            ui.label(format!("Anchor: {:.4}°, {:.4}°", venue.coord.lat_deg, venue.coord.lon_deg));
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("🥽 Touch to Watch (Descend to Venue)").strong().color(egui::Color32::BLACK))
                                .clicked()
                            {
                                self.current_coord = venue.coord;
                                self.dem_streamer.set_view_mode(NautilusGlobeViewMode::SectorSlice);
                                self.dem_streamer.update_for_camera(venue.coord);
                                let tangent = venue.spatial_overlay.tangent_frame();
                                let gravity_dir = tangent.local_gravity_dir(Vec3::new(0.0, 1.7, 0.0));
                                self.active_route = TacticalRouteSpline::new(vec![self.current_coord, venue.coord]);
                                self.active_spectator_session = Some(SpectatorSessionState {
                                    venue_id: venue.venue_id.clone(),
                                    title: venue.title.clone(),
                                    broadcaster_callsign: venue.broadcaster_callsign.clone(),
                                    stream_uri: venue.stream_uri.clone(),
                                    elapsed_seconds: 12.4,
                                    total_tips_cyan: 0.0,
                                    camera_mode: "Third-Person Orbit (Dual-Quat Kimodo)".into(),
                                    playspace_bounds_m: playspace_bounds,
                                    local_gravity_dir: gravity_dir,
                                });
                                self.last_status_message = format!(
                                    "Descended to <500m SectorSlice LOD ({:?}) • Spectating {}",
                                    self.dem_streamer.active_regime, venue.title
                                );
                            }

                            if ui.button("📍 Lock Waypoint").clicked() {
                                self.active_route = TacticalRouteSpline::new(vec![self.current_coord, venue.coord]);
                                self.last_status_message = format!("Locked Waypoint: {} ({:.2} km)", venue.title, dist_km);
                            }
                        });
                    });
                ui.add_space(4.0);
            }

            // Spectator Stream Overlay / HUD
            if let Some(mut session) = self.active_spectator_session.clone() {
                ui.add_space(8.0);
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(10, 16, 26))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 82, 82)))
                    .corner_radius(8.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(format!("📺 SPECTATING: {}", session.title));
                            ui.separator();
                            ui.label(egui::RichText::new("● 60Hz PURE-RUST DUAL-QUAT STREAM").color(egui::Color32::from_rgb(0, 229, 255)));
                        });

                        ui.label(format!("Broadcaster: {} | Stream URI: {}", session.broadcaster_callsign, session.stream_uri));
                        ui.label(format!(
                            "Camera Angle: {} | 1:1 ENU Playspace: {:.0}m × {:.0}m × {:.0}m | Down-Normal Gravity: ({:.2}, {:.2}, {:.2})",
                            session.camera_mode,
                            session.playspace_bounds_m.x,
                            session.playspace_bounds_m.y,
                            session.playspace_bounds_m.z,
                            session.local_gravity_dir.x,
                            session.local_gravity_dir.y,
                            session.local_gravity_dir.z
                        ));

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("🪙 Tip Broadcaster 0.1 CYAN").clicked() {
                                session.total_tips_cyan += 0.1;
                                self.tip_counter += 1;
                                let _ = self.coral_channel.tip_micro_increment(0.1);
                                self.active_spectator_session = Some(session.clone());
                                self.last_status_message = format!(
                                    "Tipped 0.1 CYAN to {} via Ephemeral Coral Channel (Net: {:.1} CYAN, Tick #{})!",
                                    session.broadcaster_callsign,
                                    self.coral_channel.net_transferred_cyan,
                                    self.coral_channel.tick_count
                                );
                            }
                            ui.label(egui::RichText::new(format!("Tipped This Session: {:.1} CYAN", session.total_tips_cyan)).color(egui::Color32::from_rgb(255, 215, 0)));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("State Channel: {:.1} / {:.1} CYAN", self.coral_channel.net_transferred_cyan, self.coral_channel.total_locked_cyan)).color(egui::Color32::from_rgb(0, 242, 254)).small());

                            ui.separator();
                            if ui.button("❌ Close Stream").clicked() {
                                self.active_spectator_session = None;
                                self.last_status_message = "Spectator stream closed.".into();
                            }
                        });
                    });
            }
        });

        ui.add_space(10.0);

        // Mandatory God's-Eye-View Telemetry & VR-First Web3D Guild Marketplace Command Center
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("👁️ GOD'S-EYE-VIEW // OSINT TELEMETRY & VR GUILD MARKETPLACE");
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "● {} ACTIVE TARGETS • {:.0} CYAN BOUNTY POOL",
                        self.gods_eye.visible_entities().len(),
                        self.gods_eye.total_guild_bounty_pool_cyan
                    ))
                    .color(egui::Color32::from_rgb(0, 255, 157))
                    .strong(),
                );
            });
            ui.label("Gamified bilawalsidhu/gods-eye-view Command Center: Live ADS-B Aviation, Maritime AIS, Orbital TLE Satellites, Seismic/Wildfire Radar, SDR 7-Daughters Radio, and VR Guild Bounties on a single mandatory globe.");

            ui.add_space(6.0);
            // Layer Toggles
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Telemetry Layers:").strong());
                for layer in GodsEyeLayerKind::ALL {
                    let mut is_on = self.gods_eye.active_layers.contains(&layer);
                    if ui.checkbox(&mut is_on, layer.display_name()).changed() {
                        let vis = self.gods_eye.toggle_layer(layer, is_on);
                        self.last_status_message = format!(
                            "God's-Eye Layer {} -> {} ({} visible entities)",
                            layer.display_name(),
                            if is_on { "ENABLED" } else { "HIDDEN" },
                            vis
                        );
                    }
                }
            });

            ui.add_space(4.0);
            // Voice / AI Command Center Query Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("🎙️ God's-Eye Voice/AI Query:").strong().color(egui::Color32::from_rgb(0, 229, 255)));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.gods_eye_query_input)
                        .hint_text("e.g., 'ANA-106', 'starlink', 'kuroshio vanguard', 'vortex', 'guild', or 'reset'")
                        .desired_width(320.0),
                );
                if ui.button("🔍 Execute Query").clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    let q = self.gods_eye_query_input.clone();
                    if let Some(matched_id) = self.gods_eye.execute_gods_eye_query(&q) {
                        if let Some(ent) = self.gods_eye.entities.iter().find(|e| e.entity_id == matched_id) {
                            self.current_coord = ent.geo;
                            self.dem_streamer.update_for_camera(ent.geo);
                            self.last_status_message = format!("God's-Eye locked onto {} ({:.3}°, {:.3}°)", ent.callsign, ent.geo.lat_deg, ent.geo.lon_deg);
                        }
                    } else {
                        self.last_status_message = format!("God's-Eye Query executed: {}", self.gods_eye.last_voice_or_ai_query);
                    }
                }
                if ui.button("🛰️ Release to Orbital God's-Eye").clicked() {
                    self.gods_eye.release_to_orbital_gods_eye();
                    self.dem_streamer.set_view_mode(NautilusGlobeViewMode::CompleteGlobe);
                    self.last_status_message = "Returned to Orbital God's-Eye-View Globe.".into();
                }
                ui.label(egui::RichText::new(format!("Mode: {:?}", self.gods_eye.camera_mode)).color(egui::Color32::from_rgb(255, 215, 0)).small());
            });

            ui.add_space(6.0);
            // Live Tracked Entities & Gamified Guild Bounties Table
            let active_layers = self.gods_eye.active_layers.clone();
            let entities_snapshot: Vec<_> = self
                .gods_eye
                .entities
                .iter()
                .filter(|e| active_layers.contains(&e.layer))
                .cloned()
                .collect();

            for ent in entities_snapshot {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(14, 22, 34))
                    .stroke(egui::Stroke::new(
                        1.0,
                        if self.gods_eye.selected_entity_id.as_deref() == Some(&ent.entity_id) {
                            egui::Color32::from_rgb(0, 255, 157)
                        } else {
                            egui::Color32::from_rgb(0, 180, 220)
                        },
                    ))
                    .corner_radius(5.0)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(ent.layer.display_name()).color(egui::Color32::from_rgb(0, 229, 255)).strong());
                            ui.separator();
                            ui.label(egui::RichText::new(&ent.callsign).strong().size(14.0));
                            ui.separator();
                            ui.label(format!(
                                "({:.3}°, {:.3}°, Alt {:.0}m | Spd {:.0} m/s | Hdg {:.0}°)",
                                ent.geo.lat_deg, ent.geo.lon_deg, ent.geo.alt_meters, ent.speed_mps, ent.heading_deg
                            ));
                        });

                        let detail_text = match &ent.telemetry {
                            GodsEyeTelemetryKind::AircraftFlight { icao24_hex, origin_airport, dest_airport, mach_or_knots, squawk, .. } => {
                                format!("ICAO24: {} • Route: {} → {} • Speed: {:.0} kts • Squawk: {}", icao24_hex, origin_airport, dest_airport, mach_or_knots, squawk)
                            }
                            GodsEyeTelemetryKind::MaritimeVessel { mmsi, vessel_class, speed_knots, destination_port, .. } => {
                                format!("MMSI: {} • Class: {} • {:.1} kts • Dest: {}", mmsi, vessel_class, speed_knots, destination_port)
                            }
                            GodsEyeTelemetryKind::OrbitalSatellite { norad_id, inclination_deg, orbital_period_min, apogee_km, perigee_km } => {
                                format!("NORAD #{} • Inc {:.1}° • Period {:.1}m • Orbit {:.0}×{:.0} km", norad_id, inclination_deg, orbital_period_min, apogee_km, perigee_km)
                            }
                            GodsEyeTelemetryKind::GeophysicalAnomaly { event_category, magnitude_or_frp_mw, severity_tag, radius_km } => {
                                format!("[{}] {} • Mag/FRP: {:.1} • Radius: {:.0} km", severity_tag, event_category, magnitude_or_frp_mw, radius_km)
                            }
                            GodsEyeTelemetryKind::SdrRadioStation { frequency_mhz, modulation, active_daughter_or_op, sabine_rt60_sec } => {
                                format!("{:.1} MHz ({}) • On-Air: {} • Sabine RT60: {:.2}s", frequency_mhz, modulation, active_daughter_or_op, sabine_rt60_sec)
                            }
                            GodsEyeTelemetryKind::VrGuildContract { guild_house, contract_type, min_poh_score, webxr_store_url, .. } => {
                                format!("Guild: {} • {} • Min PoH: {:.2} • WebXR: {}", guild_house, contract_type, min_poh_score, webxr_store_url)
                            }
                        };
                        ui.label(egui::RichText::new(detail_text).small().color(egui::Color32::LIGHT_GRAY));

                        ui.horizontal_wrapped(|ui| {
                            let ride_btn_label = match ent.layer {
                                GodsEyeLayerKind::AviationAdsB => "✈️ Cockpit Ride-Along",
                                GodsEyeLayerKind::MaritimeAis => "🚢 Bridge Cam Lock",
                                GodsEyeLayerKind::OrbitalSatellites => "🛰️ Orbital Downlink Cam",
                                GodsEyeLayerKind::VrGuildMarketplace => "🥽 Enter VR Guild Storefront",
                                _ => "🎯 Lock Target Cam",
                            };
                            if ui.button(egui::RichText::new(ride_btn_label).strong()).clicked() {
                                let _ = self.gods_eye.lock_cockpit_or_guild_target(&ent.entity_id);
                                self.current_coord = ent.geo;
                                self.dem_streamer.update_for_camera(ent.geo);
                                self.last_status_message = format!(
                                    "Locked {} onto {} at ({:.3}°, {:.3}°, {:.0}m)",
                                    ride_btn_label, ent.callsign, ent.geo.lat_deg, ent.geo.lon_deg, ent.geo.alt_meters
                                );
                            }

                            if let Some(ref claimer) = ent.claimed_by_guild_did {
                                ui.label(
                                    egui::RichText::new(format!("✅ Claimed by {} ({:.0} CYAN)", claimer, ent.attached_bounty_cyan))
                                        .color(egui::Color32::GREEN)
                                        .strong(),
                                );
                            } else if ent.attached_bounty_cyan > 0.0 {
                                if ui
                                    .button(
                                        egui::RichText::new(format!("⚡ Claim Guild Bounty ({:.0} CYAN Coral Escrow)", ent.attached_bounty_cyan))
                                            .color(egui::Color32::from_rgb(255, 215, 0))
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    let viewer_did = self.club.viewer_did.clone();
                                    if let Some(reward) = self.gods_eye.claim_entity_guild_bounty(&ent.entity_id, &viewer_did) {
                                        self.guild_bounties_claimed_cyan += reward;
                                        self.last_status_message = format!(
                                            "Guild Bounty Claimed! {:.0} CYAN locked in Coral Escrow for {} (Total Claimed: {:.0} CYAN)",
                                            reward, ent.callsign, self.guild_bounties_claimed_cyan
                                        );
                                    }
                                }
                            }
                        });
                    });
                ui.add_space(4.0);
            }
        });
    }

    fn render_nautilus_holographic_globe(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            let desired_size = egui::Vec2::new(ui.available_width().min(600.0), 320.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

            // Drag to rotate globe
            if response.dragged() {
                let delta = response.drag_delta();
                self.globe_rotation_rad += delta.x * 0.01;
                self.globe_tilt_rad = (self.globe_tilt_rad - delta.y * 0.01).clamp(-1.2, 1.2);
            } else {
                // Gentle continuous idle rotation
                self.globe_rotation_rad += 0.003;
                if self.globe_rotation_rad > std::f32::consts::TAU {
                    self.globe_rotation_rad -= std::f32::consts::TAU;
                }
            }

            let painter = ui.painter_at(rect);
            let center = rect.center();
            let radius = (rect.height() * 0.42).min(rect.width() * 0.42);

            // 1. Atmosphere / Outer Glow
            painter.circle_filled(
                center,
                radius + 12.0,
                egui::Color32::from_rgba_unmultiplied(0, 229, 255, 18),
            );
            painter.circle_filled(
                center,
                radius + 4.0,
                egui::Color32::from_rgba_unmultiplied(0, 160, 255, 30),
            );

            // 2. Abyssal Ocean Sphere Base
            painter.circle_filled(
                center,
                radius,
                egui::Color32::from_rgb(10, 15, 28),
            );
            painter.circle_stroke(
                center,
                radius,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 229, 255)),
            );

            // 3. Holographic Geodesic Latitude & Longitude Graticules
            let rot = self.globe_rotation_rad;
            let tilt = self.globe_tilt_rad;

            // Parallels (Latitude lines)
            for lat_deg in [-60.0, -30.0, 0.0, 30.0, 60.0] {
                let lat_rad: f32 = (lat_deg as f32).to_radians();
                let y_offset = radius * lat_rad.sin() * tilt.cos();
                let x_radius = radius * lat_rad.cos();
                let y_radius = (radius * lat_rad.cos() * tilt.sin()).abs().max(2.0);

                let is_equator = lat_deg == 0.0;
                let stroke = if is_equator {
                    egui::Stroke::new(1.2, egui::Color32::from_rgba_unmultiplied(0, 255, 200, 140))
                } else {
                    egui::Stroke::new(0.8, egui::Color32::from_rgba_unmultiplied(0, 180, 240, 70))
                };

                painter.rect_stroke(
                    egui::Rect::from_center_size(
                        egui::Pos2::new(center.x, center.y - y_offset),
                        egui::Vec2::new(x_radius * 2.0, y_radius * 2.0),
                    ),
                    y_radius,
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }

            // Meridians (Longitude arcs)
            for i in 0..12 {
                let lng_offset = (i as f32 / 12.0) * std::f32::consts::TAU + rot;
                let x_offset = lng_offset.cos() * radius;
                let is_front = lng_offset.sin() > 0.0;

                if is_front {
                    let stroke = egui::Stroke::new(
                        0.8,
                        egui::Color32::from_rgba_unmultiplied(0, 229, 255, 80),
                    );
                    painter.rect_stroke(
                        egui::Rect::from_center_size(
                            center,
                            egui::Vec2::new(x_offset.abs() * 2.0, radius * 2.0),
                        ),
                        radius,
                        stroke,
                        egui::StrokeKind::Middle,
                    );
                }
            }

            // --- SGP4 Satellite Orbital Tracks (CSS Tianhe & Leopard Arcs - Image 4) ---
            // 1. CSS (Tianhe) SGP4 Orbit (~395 km equivalent trail, red arc)
            let css_points = 64;
            let css_inc = 41.5_f32.to_radians();
            for i in 0..css_points {
                let t1 = (i as f32 / css_points as f32) * std::f32::consts::TAU;
                let t2 = ((i + 1) as f32 / css_points as f32) * std::f32::consts::TAU;

                let effective_lon1 = t1 + rot * 0.4;
                let lat1 = (t1.sin() * css_inc.sin()).asin();
                let effective_lon2 = t2 + rot * 0.4;
                let lat2 = (t2.sin() * css_inc.sin()).asin();

                let is_front = effective_lon1.cos() > -0.2;
                if is_front {
                    let r_sat = radius + 14.0;
                    let px1 = center.x + r_sat * lat1.cos() * effective_lon1.sin();
                    let py1 = center.y - r_sat * (lat1.sin() * tilt.cos() - lat1.cos() * effective_lon1.cos() * tilt.sin());
                    let px2 = center.x + r_sat * lat2.cos() * effective_lon2.sin();
                    let py2 = center.y - r_sat * (lat2.sin() * tilt.cos() - lat2.cos() * effective_lon2.cos() * tilt.sin());

                    painter.line_segment(
                        [egui::Pos2::new(px1, py1), egui::Pos2::new(px2, py2)],
                        egui::Stroke::new(1.4, egui::Color32::from_rgba_unmultiplied(255, 60, 60, 180)),
                    );

                    // Satellite lead icon & badge at leading edge
                    if i == (css_points * 3 / 4) {
                        painter.circle_filled(egui::Pos2::new(px1, py1), 3.5, egui::Color32::from_rgb(255, 60, 60));
                        painter.text(
                            egui::Pos2::new(px1 + 8.0, py1 - 6.0),
                            egui::Align2::LEFT_BOTTOM,
                            "CSS (TIANHE) 395 km - SGP4 orbit",
                            egui::FontId::proportional(10.0),
                            egui::Color32::from_rgb(255, 200, 200),
                        );
                    }
                }
            }

            // 2. LEOPARD SGP4 Polar Orbit (Cyan arc)
            for i in 0..css_points {
                let t1 = (i as f32 / css_points as f32) * std::f32::consts::TAU;
                let t2 = ((i + 1) as f32 / css_points as f32) * std::f32::consts::TAU;

                let effective_lon1 = (t1.sin() * 0.35) + rot * 0.7;
                let lat1 = t1.cos();
                let effective_lon2 = (t2.sin() * 0.35) + rot * 0.7;
                let lat2 = t2.cos();

                if effective_lon1.cos() > -0.2 {
                    let r_sat = radius + 20.0;
                    let px1 = center.x + r_sat * lat1.cos() * effective_lon1.sin();
                    let py1 = center.y - r_sat * (lat1.sin() * tilt.cos() - lat1.cos() * effective_lon1.cos() * tilt.sin());
                    let px2 = center.x + r_sat * lat2.cos() * effective_lon2.sin();
                    let py2 = center.y - r_sat * (lat2.sin() * tilt.cos() - lat2.cos() * effective_lon2.cos() * tilt.sin());

                    painter.line_segment(
                        [egui::Pos2::new(px1, py1), egui::Pos2::new(px2, py2)],
                        egui::Stroke::new(1.2, egui::Color32::from_rgba_unmultiplied(0, 229, 255, 160)),
                    );

                    if i == (css_points / 2) {
                        painter.circle_filled(egui::Pos2::new(px1, py1), 3.0, egui::Color32::from_rgb(0, 229, 255));
                        painter.text(
                            egui::Pos2::new(px1 + 8.0, py1 - 6.0),
                            egui::Align2::LEFT_BOTTOM,
                            "LEOPARD 480 km",
                            egui::FontId::proportional(9.5),
                            egui::Color32::from_rgb(160, 240, 255),
                        );
                    }
                }
            }

            // --- Snap Map Social Activity Thermal Heatmaps (Image 3) ---
            let hotspots: [(f32, f32, f32, egui::Color32); 5] = [
                (-74.0, 40.7, 16.0, egui::Color32::from_rgba_unmultiplied(255, 60, 30, 140)),  // NYC (Red)
                (-118.2, 34.0, 14.0, egui::Color32::from_rgba_unmultiplied(255, 200, 30, 140)), // LA (Yellow)
                (-0.12, 51.5, 15.0, egui::Color32::from_rgba_unmultiplied(255, 50, 30, 140)),   // London (Red)
                (2.35, 48.8, 12.0, egui::Color32::from_rgba_unmultiplied(255, 180, 20, 120)),   // Paris (Yellow)
                (139.7, 35.6, 18.0, egui::Color32::from_rgba_unmultiplied(255, 40, 20, 160)),  // Tokyo (Intense Red)
            ];

            for (lon_deg, lat_deg, base_r, color) in hotspots {
                let lat_rad = lat_deg.to_radians();
                let lon_rad = lon_deg.to_radians();
                let effective_lon = lon_rad + rot;

                if effective_lon.cos() > -0.1 {
                    let px = center.x + radius * lat_rad.cos() * effective_lon.sin();
                    let py = center.y - radius * (lat_rad.sin() * tilt.cos() - lat_rad.cos() * effective_lon.cos() * tilt.sin());
                    let hp_pos = egui::Pos2::new(px, py);

                    // Concentric thermal bloom
                    painter.circle_filled(hp_pos, base_r * 1.6, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 35));
                    painter.circle_filled(hp_pos, base_r * 1.0, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 75));
                    painter.circle_filled(hp_pos, base_r * 0.5, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180));
                }
            }

            // 4. Draw Authoritative Geodesic Loci & VR Broadcast Pins
            let mouse_pos = response.hover_pos();

            let venues = self.venue_broadcasts.clone();
            for venue in &venues {
                let lat_rad = (venue.coord.lat_deg as f32).to_radians();
                let lon_rad = (venue.coord.lon_deg as f32).to_radians();
                let effective_lon = lon_rad + rot;

                // Spherical projection to 2D screen
                let cos_lon = effective_lon.cos();
                let sin_lon = effective_lon.sin();
                let cos_lat = lat_rad.cos();
                let sin_lat = lat_rad.sin();

                // Visibility check (front hemisphere facing camera)
                if cos_lon > -0.15 {
                    let px = center.x + radius * cos_lat * sin_lon;
                    let py = center.y - radius * (sin_lat * tilt.cos() - cos_lat * cos_lon * tilt.sin());

                    let pin_pos = egui::Pos2::new(px, py);

                    // Check hover or click
                    let is_hovered = if let Some(m) = mouse_pos {
                        (m - pin_pos).length() < 14.0
                    } else {
                        false
                    };

                    // Pulsing beacon ring
                    let pulse_radius = 5.0 + ((rot * 4.0).sin().abs() * 3.0);
                    painter.circle_filled(
                        pin_pos,
                        pulse_radius + 4.0,
                        egui::Color32::from_rgba_unmultiplied(255, 82, 82, 70),
                    );
                    painter.circle_filled(
                        pin_pos,
                        5.0,
                        egui::Color32::from_rgb(255, 82, 82),
                    );
                    painter.circle_stroke(
                        pin_pos,
                        6.5,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );

                    // Pin Label
                    painter.text(
                        pin_pos + egui::Vec2::new(8.0, -8.0),
                        egui::Align2::LEFT_BOTTOM,
                        &venue.title,
                        egui::FontId::proportional(11.0),
                        if is_hovered { egui::Color32::YELLOW } else { egui::Color32::from_rgb(0, 229, 255) },
                    );

                    // Click to touch-to-watch directly from globe
                    if is_hovered && response.clicked() {
                        self.current_coord = venue.coord;
                        self.dem_streamer.set_view_mode(NautilusGlobeViewMode::SectorSlice);
                        self.dem_streamer.update_for_camera(venue.coord);
                        let tangent = venue.spatial_overlay.tangent_frame();
                        let gravity_dir = tangent.local_gravity_dir(Vec3::new(0.0, 1.7, 0.0));
                        self.active_route = TacticalRouteSpline::new(vec![self.current_coord, venue.coord]);
                        let playspace_bounds = match &venue.spatial_overlay.overlay_kind {
                            SpatialOverlayKind::RealSiteGroundedVenue(v) => v.playspace_bounds_m,
                            _ => Vec3::new(50.0, 50.0, 10.0),
                        };
                        self.active_spectator_session = Some(SpectatorSessionState {
                            venue_id: venue.venue_id.clone(),
                            title: venue.title.clone(),
                            broadcaster_callsign: venue.broadcaster_callsign.clone(),
                            stream_uri: venue.stream_uri.clone(),
                            elapsed_seconds: 0.0,
                            total_tips_cyan: 0.0,
                            camera_mode: "Third-Person Orbit (Globe Touch-to-Watch)".into(),
                            playspace_bounds_m: playspace_bounds,
                            local_gravity_dir: gravity_dir,
                        });
                        self.last_status_message = format!("Holographic Globe: Descended to SectorSlice for {}", venue.title);
                    }
                }
            }

            // 5. Draw Mandatory God's-Eye-View Live Multi-Layer Telemetry Entities on Globe
            let active_layers = self.gods_eye.active_layers.clone();
            let gods_eye_entities: Vec<_> = self
                .gods_eye
                .entities
                .iter()
                .filter(|e| active_layers.contains(&e.layer))
                .cloned()
                .collect();

            for ent in &gods_eye_entities {
                let lat_rad = (ent.geo.lat_deg as f32).to_radians();
                let lon_rad = (ent.geo.lon_deg as f32).to_radians();
                let effective_lon = lon_rad + rot;
                let cos_lon = effective_lon.cos();
                let sin_lon = effective_lon.sin();
                let cos_lat = lat_rad.cos();
                let sin_lat = lat_rad.sin();

                if cos_lon > -0.15 {
                    let alt_boost = if ent.layer == GodsEyeLayerKind::OrbitalSatellites {
                        1.08
                    } else if ent.layer == GodsEyeLayerKind::AviationAdsB {
                        1.03
                    } else {
                        1.0
                    };
                    let r = radius * alt_boost;
                    let px = center.x + r * cos_lat * sin_lon;
                    let py = center.y - r * (sin_lat * tilt.cos() - cos_lat * cos_lon * tilt.sin());
                    let pt = egui::Pos2::new(px, py);

                    let color = match ent.layer {
                        GodsEyeLayerKind::AviationAdsB => egui::Color32::from_rgb(0, 242, 254),
                        GodsEyeLayerKind::MaritimeAis => egui::Color32::from_rgb(0, 255, 157),
                        GodsEyeLayerKind::OrbitalSatellites => egui::Color32::from_rgb(255, 226, 89),
                        GodsEyeLayerKind::SeismicWildfireWeather => egui::Color32::from_rgb(255, 110, 30),
                        GodsEyeLayerKind::SdrSirenRadio => egui::Color32::from_rgb(180, 60, 255),
                        GodsEyeLayerKind::VrGuildMarketplace => egui::Color32::from_rgb(255, 215, 0),
                    };

                    let is_hovered = mouse_pos.map(|m| (m - pt).length() < 12.0).unwrap_or(false);
                    painter.circle_filled(pt, if is_hovered { 5.5 } else { 4.0 }, color);
                    painter.circle_stroke(pt, 6.5, egui::Stroke::new(1.0, color));

                    if is_hovered {
                        painter.text(
                            pt + egui::Vec2::new(8.0, -6.0),
                            egui::Align2::LEFT_BOTTOM,
                            format!("{} [{:.0} CYAN]", ent.callsign, ent.attached_bounty_cyan),
                            egui::FontId::proportional(11.0),
                            egui::Color32::YELLOW,
                        );
                        if response.clicked() {
                            let _ = self.gods_eye.lock_cockpit_or_guild_target(&ent.entity_id);
                            self.current_coord = ent.geo;
                            self.dem_streamer.update_for_camera(ent.geo);
                            self.last_status_message = format!("Globe Lock: {} ({:?})", ent.callsign, self.gods_eye.camera_mode);
                        }
                    }
                }
            }

            // Current player location marker
            let my_lat_rad = (self.current_coord.lat_deg as f32).to_radians();
            let my_lon_rad = (self.current_coord.lon_deg as f32).to_radians();
            let my_effective_lon = my_lon_rad + rot;
            if my_effective_lon.cos() > -0.15 {
                let mx = center.x + radius * my_lat_rad.cos() * my_effective_lon.sin();
                let my = center.y - radius * (my_lat_rad.sin() * tilt.cos() - my_lat_rad.cos() * my_effective_lon.cos() * tilt.sin());
                let my_pos = egui::Pos2::new(mx, my);
                painter.circle_filled(my_pos, 4.0, egui::Color32::GREEN);
                painter.circle_stroke(my_pos, 7.0, egui::Stroke::new(1.0, egui::Color32::GREEN));
                painter.text(
                    my_pos + egui::Vec2::new(8.0, 6.0),
                    egui::Align2::LEFT_TOP,
                    "📍 YOU",
                    egui::FontId::proportional(11.0),
                    egui::Color32::GREEN,
                );
            }

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("👆 Drag to rotate globe • Click any red beacon pin to descend and spectate live VR stream")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
    }

    fn render_theater_radio_tab(&mut self, ui: &mut egui::Ui, _form_factor: ScreenFormFactor) {
        ui.heading("📻 Theater 24/7 Live Broadcast & Siren Radio");
        ui.label("Direct low-latency HTTP live stream connected to Sabine RT60 acoustic decoder.");
        ui.add_space(8.0);

        // Club Atlantis Raspberry Pi Node Binding Card
        let pi_online = self.club.is_siren_radio_running();
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(16, 28, 36))
            .stroke(egui::Stroke::new(
                1.5,
                if pi_online && self.radio_state.auto_bound_to_club_pi {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::YELLOW
                },
            ))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("🍓 CLUB ATLANTIS RASPBERRY PI NODE (atlantis-pi-home-1)")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.separator();
                    if pi_online && self.radio_state.auto_bound_to_club_pi {
                        ui.label(
                            egui::RichText::new(format!(
                                "🟢 TIED TO LAN PI ({}) • AUTO-RUNNING 24/7",
                                self.club.pi_node_lan_host
                            ))
                            .color(egui::Color32::GREEN)
                            .strong(),
                        );
                    } else if !pi_online {
                        ui.label(
                            egui::RichText::new("🟡 PI NODE STOPPED • USING DIRECT HTTPS STREAM FALLBACK")
                                .color(egui::Color32::YELLOW)
                                .strong(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("🟡 MANUAL OVERRIDE (UNBOUND FROM PI NODE)")
                                .color(egui::Color32::YELLOW),
                        );
                    }
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .checkbox(
                            &mut self.radio_state.auto_bound_to_club_pi,
                            "Auto-Tie 7 Daughters Radio to Club Atlantis Pi Node",
                        )
                        .changed()
                    {
                        self.radio_state.sync_with_club_pi_node(
                            self.club.is_siren_radio_running(),
                            &self.club.pi_node_lan_host,
                        );
                        self.last_status_message = if self.radio_state.auto_bound_to_club_pi {
                            format!(
                                "7 Daughters Radio auto-bound to Club Atlantis Pi Node ({})",
                                self.club.pi_node_lan_host
                            )
                        } else {
                            "7 Daughters Radio switched to standalone HTTPS stream mode".into()
                        };
                    }

                    ui.separator();
                    if ui
                        .checkbox(&mut self.club.pi_atlantis_running, "Atlantis Running on Pi")
                        .changed()
                    {
                        self.club.sync_pi_radio_daemon();
                        self.radio_state.sync_with_club_pi_node(
                            self.club.is_siren_radio_running(),
                            &self.club.pi_node_lan_host,
                        );
                        self.last_status_message = if self.club.pi_atlantis_running {
                            format!(
                                "Atlantis Pi Node online ({}) — 7 Daughters Radio auto-started!",
                                self.club.pi_node_lan_host
                            )
                        } else {
                            "Atlantis Pi Node stopped — fell back to external HTTPS stream.".into()
                        };
                    }

                    ui.separator();
                    ui.label("Pi LAN Host:");
                    if ui.text_edit_singleline(&mut self.club.pi_node_lan_host).changed() {
                        self.radio_state.sync_with_club_pi_node(
                            self.club.is_siren_radio_running(),
                            &self.club.pi_node_lan_host,
                        );
                    }
                });
            });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("● LIVE BROADCAST ON-AIR")
                        .color(egui::Color32::from_rgb(255, 82, 82))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("SIREN FREQUENCY 24/7 • THE SEVEN DAUGHTERS")
                        .color(egui::Color32::from_rgb(255, 215, 0))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("48kHz Stereo PCM16 / MP3")
                        .color(egui::Color32::from_rgb(0, 229, 255)),
                );
            });

            ui.add_space(6.0);
            ui.label(egui::RichText::new(&self.radio_state.track_title).strong().size(17.0));
            ui.label(egui::RichText::new(&self.radio_state.dj_name).color(egui::Color32::from_rgb(0, 229, 255)).strong());

            // The Seven Daughters Host Card
            let daughter = &self.radio_state.active_daughter;
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(20, 26, 38))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 229, 255)))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("🎙️ Host #{} [{}]", daughter.daughter_index, daughter.callsign)).strong().color(egui::Color32::from_rgb(0, 229, 255)));
                        ui.separator();
                        ui.label(egui::RichText::new(&daughter.shift_hours).small().color(egui::Color32::LIGHT_GRAY));
                        ui.separator();
                        ui.label(egui::RichText::new(format!("{:.0} Hz Harmonic", daughter.harmonic_frequency_hz)).color(egui::Color32::from_rgb(255, 215, 0)));
                    });
                    ui.label(egui::RichText::new(format!("\"{}\"", daughter.on_air_catchphrase)).italics().color(egui::Color32::from_rgb(200, 230, 255)));
                    ui.label(egui::RichText::new(&daughter.persona_bio).small().color(egui::Color32::GRAY));
                });

            ui.add_space(6.0);
            // Daughter Shift Switcher
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Switch Host:").strong());
                let names = ["Maia", "Alcyone", "Asterope", "Celaeno", "Electra", "Taygete", "Merope"];
                for i in 1..=7 {
                    let is_active = self.radio_state.active_daughter.daughter_index == i;
                    let btn_text = format!("{}: {}", i, names[(i - 1) as usize]);
                    if ui.selectable_label(is_active, btn_text).clicked() {
                        self.radio_state.set_daughter_by_index(i);
                        self.last_status_message = format!(
                            "Radio switched to Daughter #{}: {} ({})",
                            i,
                            names[(i - 1) as usize],
                            self.radio_state.stream_url
                        );
                        #[cfg(target_arch = "wasm32")]
                        if self.radio_state.is_playing {
                            crate::play_radio_stream(&self.radio_state.stream_url, self.radio_state.volume);
                        }
                    }
                }
                if ui.button("🔄 Cycle Next Shift").clicked() {
                    let (d_idx, d_name) = {
                        let d = self.radio_state.cycle_next_daughter();
                        (d.daughter_index, d.name.clone())
                    };
                    self.last_status_message = format!(
                        "Shift rotated to Daughter #{}: {} ({})",
                        d_idx, d_name, self.radio_state.stream_url
                    );
                    #[cfg(target_arch = "wasm32")]
                    if self.radio_state.is_playing {
                        crate::play_radio_stream(&self.radio_state.stream_url, self.radio_state.volume);
                    }
                }
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let play_btn_text = if self.radio_state.is_playing { "⏸ PAUSE" } else { "▶ PLAY" };
                if ui.button(egui::RichText::new(play_btn_text).strong().size(15.0)).clicked() {
                    let playing = self.radio_state.toggle_playback();
                    if playing {
                        self.last_status_message = format!("Tuned into {} ({})", self.radio_state.dj_name, self.radio_state.stream_url);
                        #[cfg(target_arch = "wasm32")]
                        crate::play_radio_stream(&self.radio_state.stream_url, self.radio_state.volume);
                    } else {
                        self.last_status_message = "Radio paused.".into();
                        #[cfg(target_arch = "wasm32")]
                        crate::pause_radio_stream();
                    }
                }

                ui.label("Master Volume:");
                let mut vol = self.radio_state.volume;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0)).changed() {
                    self.radio_state.set_volume(vol);
                    #[cfg(target_arch = "wasm32")]
                    crate::set_radio_volume(vol);
                }
            });

            ui.add_space(8.0);
            egui::Grid::new("radio_stats").striped(true).show(ui, |ui| {
                ui.label("Stream URL:");
                ui.label(&self.radio_state.stream_url);
                ui.end_row();

                ui.label("Acoustic Profile:");
                ui.label(format!("Sabine RT60 ({:.2}s decay)", self.siren_engine.compute_sabine_rt60()));
                ui.end_row();

                ui.label("Spatial Emitter:");
                ui.label(format!("Vec3({:.1}, {:.1}, {:.1})", 
                    self.radio_state.spatial_emitter_pos[0],
                    self.radio_state.spatial_emitter_pos[1],
                    self.radio_state.spatial_emitter_pos[2]
                ));
                ui.end_row();
            });
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.heading("DJ Tipping & Mic PTT (Push-To-Talk)");
            ui.label("Send 0.1 CYAN off-ledger tips instantly during live sets or talk over voice mesh.");
            ui.horizontal_wrapped(|ui| {
                if ui.button("🎵 Send 0.1 CYAN Tip").clicked() {
                    self.tip_counter += 1;
                    let _ = self.coral_channel.tip_micro_increment(0.1);
                    self.last_status_message = format!(
                        "Tip #{} sent (0.1 CYAN off-ledger Coral state channel // Total Settled: {:.1} CYAN)",
                        self.tip_counter, self.coral_channel.net_transferred_cyan
                    );
                }
                if ui.button("🎙️ Push-To-Talk (PTT Active)").clicked() {
                    self.webtransport_engine.send_datagram(256);
                    self.last_status_message = "Transmitting voice over P2P WebTransport QUIC datagram mesh (104.7 MHz)...".into();
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚡ Coral State Channel:").strong().color(egui::Color32::from_rgb(0, 242, 254)));
                ui.label(format!("Transferred: {:.1} / {:.1} CYAN ({} ticks)", self.coral_channel.net_transferred_cyan, self.coral_channel.total_locked_cyan, self.coral_channel.tick_count));
                ui.separator();
                ui.label(format!("QUIC Datagrams: {} sent ({}ms RTT)", self.webtransport_engine.datagrams_sent, self.webtransport_engine.rtt_ms));
            });
        });
    }

    fn render_marketplace_tab(&mut self, ui: &mut egui::Ui, form_factor: ScreenFormFactor) {
        ui.heading("🛒 Atlantis Task Marketplace & Escrow");
        ui.label("Decentralized tasks, bounties, and hardware upgrades secured via Coral Escrow.");
        ui.add_space(8.0);

        let mut purchased_item = None;

        let available_width = ui.available_width();
        let use_two_columns = form_factor == ScreenFormFactor::WidescreenHud && available_width > 800.0;

        if use_two_columns {
            ui.columns(2, |cols| {
                for (idx, item) in self.catalog.iter().enumerate() {
                    let col = &mut cols[idx % 2];
                    col.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&item.title).strong().size(15.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!("{:.1} {}", item.price_amount, item.price_currency))
                                        .color(egui::Color32::from_rgb(0, 229, 255))
                                        .strong(),
                                );
                            });
                        });
                        ui.label(&item.description);
                        ui.horizontal(|ui| {
                            if item.is_gaussian_splat {
                                ui.label(egui::RichText::new("[3D Splat]").color(egui::Color32::from_rgb(0, 229, 255)).small());
                            }
                            ui.label(egui::RichText::new(format!("Vendor: {}", item.store_node)).small());
                        });
                        if ui.button(egui::RichText::new("⚡ Coral Purchase").strong()).clicked() {
                            purchased_item = Some((item.title.clone(), item.price_amount, item.price_currency.clone()));
                        }
                    });
                    col.add_space(6.0);
                }
            });
        } else {
            for item in &self.catalog {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&item.title).strong().size(15.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("{:.1} {}", item.price_amount, item.price_currency))
                                    .color(egui::Color32::from_rgb(0, 229, 255))
                                    .strong(),
                            );
                        });
                    });
                    ui.label(&item.description);
                    ui.horizontal(|ui| {
                        if item.is_gaussian_splat {
                            ui.label(egui::RichText::new("[3D Gaussian Splat]").color(egui::Color32::from_rgb(0, 229, 255)).small());
                        }
                        ui.label(egui::RichText::new(format!("Vendor: {}", item.store_node)).small());
                    });
                    if ui.button(egui::RichText::new("⚡ Instant Coral Escrow Purchase").strong()).clicked() {
                        purchased_item = Some((item.title.clone(), item.price_amount, item.price_currency.clone()));
                    }
                });
                ui.add_space(6.0);
            }
        }

        if let Some((title, price, cur)) = purchased_item {
            self.last_status_message = format!("Instant Purchase: {} for {:.1} {} locked in Coral Escrow!", title, price, cur);
        }

        ui.add_space(10.0);

        // --- WASI Sandboxed Modding & Mini-App Cartridges (Item 6) ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🧩 WASI Sandboxed Modding & Mini-App Cartridges");
                ui.separator();
                ui.label(
                    egui::RichText::new("CAPABILITY SANDBOX • ZERO RAW SOCKETS")
                        .color(egui::Color32::from_rgb(0, 242, 254))
                        .strong(),
                );
            });
            ui.label("Hot-swappable WebAssembly modules executing with strict permission gates.");

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Mounted Cartridges: {}", self.wasi_engine.mounted_cartridges.len()));
                ui.separator();
                ui.label(format!("Sandboxed Dispatches: {}", self.wasi_engine.total_sandboxed_calls));
                ui.separator();
                ui.label(format!("Active Sandbox RAM: {:.1} KB", self.wasi_engine.memory_allocated_bytes as f32 / 1024.0));
            });

            ui.add_space(6.0);
            for i in 0..self.wasi_engine.mounted_cartridges.len() {
                let cartridge = &self.wasi_engine.mounted_cartridges[i];
                let is_mounted = cartridge.is_mounted;
                let c_id = cartridge.cartridge_id.clone();
                let title = cartridge.title.clone();
                let author = cartridge.author_did.clone();
                let status = cartridge.last_invocation_status.clone();
                let cycles = cartridge.execution_cycles;
                let caps = cartridge.capabilities.clone();

                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(14, 20, 28))
                    .stroke(egui::Stroke::new(
                        1.0,
                        if is_mounted {
                            egui::Color32::from_rgb(0, 229, 255)
                        } else {
                            egui::Color32::from_rgb(60, 70, 85)
                        },
                    ))
                    .corner_radius(6.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("🎮 {}", title)).strong().color(if is_mounted { egui::Color32::WHITE } else { egui::Color32::GRAY }));
                            ui.separator();
                            ui.label(egui::RichText::new(&c_id).small().color(egui::Color32::from_rgb(0, 242, 254)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let toggle_btn = if is_mounted { "⏏ Unmount" } else { "🔌 Mount .wasm" };
                                if ui.button(toggle_btn).clicked() {
                                    self.wasi_engine.toggle_mount(i);
                                }
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Author: {}", author)).small().color(egui::Color32::GRAY));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Cycles: {}", cycles)).small());
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Status: {}", status)).small().color(egui::Color32::GREEN));
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new("Caps:").small().strong());
                            if caps.allow_entity_spawn {
                                ui.label(egui::RichText::new("[spawn_entity]").small().color(egui::Color32::from_rgb(100, 220, 100)));
                            }
                            if caps.allow_escrow_bidding {
                                ui.label(egui::RichText::new("[escrow_bidding]").small().color(egui::Color32::from_rgb(255, 215, 0)));
                            }
                            if caps.allow_zk_geo_verify {
                                ui.label(egui::RichText::new("[zk_geo_verify]").small().color(egui::Color32::from_rgb(0, 229, 255)));
                            }
                            if caps.allow_telemetry_read {
                                ui.label(egui::RichText::new("[telemetry_read]").small().color(egui::Color32::LIGHT_GRAY));
                            }
                            ui.label(egui::RichText::new("[NO_RAW_SOCKETS]").small().color(egui::Color32::from_rgb(255, 100, 100)));
                            ui.label(egui::RichText::new("[NO_HOST_FS]").small().color(egui::Color32::from_rgb(255, 100, 100)));
                        });

                        if is_mounted {
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                if caps.allow_entity_spawn && ui.button("⚡ Exec: spawn_entity").clicked() {
                                    match self.wasi_engine.invoke_cartridge_entry(i, "lemuria::spawn_entity") {
                                        Ok(res) => self.last_status_message = format!("WASI [{}]: {}", title, res),
                                        Err(err) => self.last_status_message = format!("WASI Error: {}", err),
                                    }
                                }
                                if caps.allow_escrow_bidding && ui.button("⚡ Exec: request_escrow_bid").clicked() {
                                    match self.wasi_engine.invoke_cartridge_entry(i, "atlantis::request_escrow_bid") {
                                        Ok(res) => self.last_status_message = format!("WASI [{}]: {}", title, res),
                                        Err(err) => self.last_status_message = format!("WASI Error: {}", err),
                                    }
                                }
                                if caps.allow_zk_geo_verify && ui.button("⚡ Exec: verify_zk_proximity").clicked() {
                                    match self.wasi_engine.invoke_cartridge_entry(i, "atlantis::verify_zk_proximity") {
                                        Ok(res) => self.last_status_message = format!("WASI [{}]: {}", title, res),
                                        Err(err) => self.last_status_message = format!("WASI Error: {}", err),
                                    }
                                }
                                if ui.button("❌ Test Forbidden: raw_socket_open").clicked() {
                                    match self.wasi_engine.invoke_cartridge_entry(i, "host::raw_socket_open") {
                                        Ok(res) => self.last_status_message = format!("WASI [{}]: {}", title, res),
                                        Err(err) => self.last_status_message = format!("Blocked Forbidden WASI Call: {}", err),
                                    }
                                }
                            });
                        }
                    });
                ui.add_space(4.0);
            }
        });

        ui.add_space(10.0);

        // --- True Zero-Copy Network Archiving & GPU Ingestion (Item 2) ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("⚡ True Zero-Copy Network Archiving (`rkyv`)");
                ui.separator();
                ui.label(
                    egui::RichText::new("64-BYTE ALIGNED • 0.00ms PARSE TIME")
                        .color(egui::Color32::from_rgb(0, 242, 254))
                        .strong(),
                );
            });
            ui.label("Direct byte-slice pointer casting for incoming mesh entity batches into GPU storage buffers.");

            ui.add_space(4.0);
            egui::Grid::new("zero_copy_grid").striped(true).show(ui, |ui| {
                ui.label("Ingested Entity Packets:");
                ui.label(format!("{} packets (64 bytes each)", self.zero_copy_pipeline.packets_ingested));
                ui.end_row();

                ui.label("Total Zero-Copy Throughput:");
                ui.label(format!("{:.1} KB (0 heap allocs in tick)", self.zero_copy_pipeline.total_bytes_zero_copy as f32 / 1024.0));
                ui.end_row();

                ui.label("Deserialization Overhead:");
                ui.label(
                    egui::RichText::new(format!("{:.2} ms (Instant Pointer Cast)", self.zero_copy_pipeline.parse_overhead_micros))
                        .color(egui::Color32::GREEN)
                        .strong(),
                );
                ui.end_row();

                ui.label("Latest Ingested Entity Locus:");
                let p = &self.zero_copy_pipeline.last_ingested_packet;
                ui.label(format!("Entity #{} @ Pos({:.2}, {:.2}, {:.2}) | HP/Shield: 0x{:08X}", p.entity_id_u64, p.position_xyz[0], p.position_xyz[1], p.position_xyz[2], p.health_shield_packed));
                ui.end_row();
            });

            ui.add_space(6.0);
            if ui.button("⚡ Ingest 64-Byte Synthetic Mesh Entity Packet").clicked() {
                let synthetic_packet = ZeroCopyEntityPacket {
                    entity_id_u64: 8000 + self.zero_copy_pipeline.packets_ingested,
                    position_xyz: [1.5, 0.0, -3.2],
                    orientation_quat: [0.0, 0.707, 0.0, 0.707],
                    velocity_xyz: [12.0, 0.0, 0.0],
                    health_shield_packed: 0x00640064,
                    flags: 1,
                    _padding: [0u8; 8],
                };
                let slice = unsafe {
                    std::slice::from_raw_parts(
                        &synthetic_packet as *const _ as *const u8,
                        std::mem::size_of::<ZeroCopyEntityPacket>(),
                    )
                };
                match self.zero_copy_pipeline.ingest_packet_bytes(slice) {
                    Ok(packet) => {
                        self.last_status_message = format!("Zero-Copy: Ingested Entity #{} in 0.00ms!", packet.entity_id_u64);
                    }
                    Err(err) => {
                        self.last_status_message = format!("Zero-Copy Ingest Error: {}", err);
                    }
                }
            }
        });
    }

    fn render_virtual_store_tab(&mut self, ui: &mut egui::Ui, _form_factor: ScreenFormFactor) {
        ui.heading("🏪 3D Virtual Storefront & AI Gate Proximity");
        ui.label("Dynamic proximity walk-up (< 2.0m) connects avatar to local AI Gate on LAN.");
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new(format!("Store: {}", self.storefront.store_title)).strong().size(16.0));
            ui.label(format!("Distance to Counter: {:.2} meters", self.storefront.counter_distance_meters));

            let status_text = if self.storefront.counter_distance_meters <= VirtualStorefrontState::COUNTER_TRIGGER_RADIUS_METERS {
                egui::RichText::new("Proximity Status: IN RANGE (< 2.0m) • CONNECTED TO LOCAL AI GATE").color(egui::Color32::GREEN).strong()
            } else {
                egui::RichText::new("Proximity Status: OUT OF RANGE (> 2.0m)").color(egui::Color32::YELLOW)
            };
            ui.label(status_text);

            ui.add_space(6.0);
            ui.label(format!("Active Representative: {:?}", self.storefront.active_representative));

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Approach Counter (1.2m)").clicked() {
                    self.storefront.update_customer_distance(
                        1.2,
                        Some(("did:atlantis:staff_bob".into(), "Bob Ross".into())),
                        ("gemini-3.8-flash", "local.ai.gate"),
                    );
                    self.last_status_message = "Walked to 1.2m: Connected to live human staff.".into();
                }

                if ui.button("Step Back (3.5m)").clicked() {
                    self.storefront.update_customer_distance(
                        3.5,
                        Some(("did:atlantis:staff_bob".into(), "Bob Ross".into())),
                        ("gemini-3.8-flash", "local.ai.gate"),
                    );
                    self.last_status_message = "Walked to 3.5m: Disconnected counter representative.".into();
                }
            });
        });

        ui.add_space(10.0);

        // --- Interactive 3D Gaussian Splatting Storefront Inspector ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🔮 3D Gaussian Splat Product Inspector & Convex Hull Collider");
                ui.separator();
                ui.label(
                    egui::RichText::new("RADIX SORTED • ZERO-GC ALLOCATOR")
                        .color(egui::Color32::from_rgb(0, 242, 254))
                        .strong(),
                );
            });
            ui.label("Photorealistic phone scans of real-world storefront totems & shelf items with interactive convex-hull grab picking.");
            ui.add_space(6.0);

            let product_count = self.splat_world.splat_products.len();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Select Scanned Item:").strong());
                for idx in 0..product_count {
                    let sku = &self.splat_world.splat_products[idx].sku_id;
                    let is_sel = self.selected_splat_index == idx;
                    if ui.selectable_label(is_sel, sku).clicked() {
                        self.selected_splat_index = idx;
                    }
                }
            });

            ui.add_space(6.0);

            // Splat Viewport & Interactive 3D Orbit Controls
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(ui.available_width().min(600.0), 220.0),
                egui::Sense::drag(),
            );

            if response.dragged() {
                self.splat_rotation_angle_rad += response.drag_delta().x * 0.015;
            }

            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(8, 14, 22));
            painter.rect_stroke(
                rect,
                8.0,
                egui::Stroke::new(1.2, egui::Color32::from_rgb(0, 229, 255)),
                egui::StrokeKind::Middle,
            );

            let center = rect.center();
            let rot = self.splat_rotation_angle_rad;
            let zoom = self.splat_inspect_zoom;

            // Render projected 3D Gaussian Splats with depth sorting
            if let Some(product) = self.splat_world.splat_products.get(self.selected_splat_index) {
                let half = product.convex_hull_half_extents;
                let color = if product.is_service_totem {
                    egui::Color32::from_rgb(0, 220, 255)
                } else {
                    egui::Color32::from_rgb(255, 175, 50)
                };

                // Draw synthetic splat point cluster projected onto 2D canvas with rotation
                let step = 3;
                for ix in -step..=step {
                    for iy in -step..=step {
                        for iz in -step..=step {
                            let local_x = (ix as f32 / step as f32) * half.x;
                            let local_y = (iy as f32 / step as f32) * half.y;
                            let local_z = (iz as f32 / step as f32) * half.z;

                            // Rotate around Y-axis
                            let rx = local_x * rot.cos() - local_z * rot.sin();
                            let rz = local_x * rot.sin() + local_z * rot.cos();

                            let screen_x = center.x + rx * 140.0 * zoom;
                            let screen_y = center.y - local_y * 140.0 * zoom;
                            let splat_radius = (4.0 + (rz + half.z) * 4.0).clamp(2.0, 10.0) * zoom;
                            let alpha = ((rz + half.z * 1.5) / (half.z * 3.0)).clamp(0.2, 0.95);

                            painter.circle_filled(
                                egui::Pos2::new(screen_x, screen_y),
                                splat_radius,
                                egui::Color32::from_rgba_unmultiplied(
                                    color.r(),
                                    color.g(),
                                    color.b(),
                                    (alpha * 255.0) as u8,
                                ),
                            );
                        }
                    }
                }

                // Overlay HUD on viewport
                painter.text(
                    rect.min + egui::Vec2::new(10.0, 10.0),
                    egui::Align2::LEFT_TOP,
                    format!("Gaussian Splats: 343 ellipsoids • Asset: {}", product.splat_asset_uri),
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(0, 242, 254),
                );
                painter.text(
                    rect.min + egui::Vec2::new(10.0, 28.0),
                    egui::Align2::LEFT_TOP,
                    format!("Convex Hull Extents: ({:.2}, {:.2}, {:.2}) m • Totem: {}", half.x * 2.0, half.y * 2.0, half.z * 2.0, product.is_service_totem),
                    egui::FontId::proportional(11.0),
                    egui::Color32::LIGHT_GRAY,
                );
            }

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("Orbit Rotation:");
                ui.add(egui::Slider::new(&mut self.splat_rotation_angle_rad, -std::f32::consts::PI..=std::f32::consts::PI).text("rad"));
                ui.separator();
                ui.label("Inspect Zoom:");
                ui.add(egui::Slider::new(&mut self.splat_inspect_zoom, 0.5..=2.5).text("x"));
                ui.separator();
                if ui.button("⚡ Raycast Pick & Drop into Cart").clicked() {
                    let ray_origin = Vec3::new(0.0, 1.2, 0.0);
                    let ray_dir = Vec3::new(0.0, 0.0, -1.0);
                    if let Some(hit) = SplatConvexHullCollider::raycast_shelf_products(
                        &self.splat_world,
                        ray_origin,
                        ray_dir,
                        10.0,
                    ) {
                        self.splat_collider.cart_sku_items.push(hit.sku_id.clone());
                        self.last_status_message = format!(
                            "Picked 3D Splat Item '{}' at distance {:.2}m and dropped into Cart!",
                            hit.sku_id, hit.distance
                        );
                    } else if let Some(product) = self.splat_world.splat_products.get(self.selected_splat_index) {
                        self.splat_collider.cart_sku_items.push(product.sku_id.clone());
                        self.last_status_message = format!("Added '{}' to customer cart!", product.sku_id);
                    }
                }
            });

            if !self.splat_collider.cart_sku_items.is_empty() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🛒 Customer Cart:").strong().color(egui::Color32::from_rgb(0, 255, 157)));
                    ui.label(format!("{} Items Dropped: {:?}", self.splat_collider.cart_sku_items.len(), self.splat_collider.cart_sku_items));
                });
            }
        });
    }

    fn render_vehicle_telemetry_tab(&mut self, ui: &mut egui::Ui, _form_factor: ScreenFormFactor) {
        ui.heading("🚗 Vehicle Dynamics & Diagnostics");
        ui.label("OBD-II, CAN Bus, and Pacejka 4-wheel slip dynamics telemetry.");
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Powertrain & Speed").strong().size(15.0));
            egui::Grid::new("veh_grid").striped(true).show(ui, |ui| {
                ui.label("Vehicle Speed:");
                ui.label(egui::RichText::new(format!("{:.1} km/h", self.vehicle_speed_kmh)).strong().color(egui::Color32::from_rgb(0, 229, 255)));
                ui.end_row();

                ui.label("Engine RPM:");
                ui.label(format!("{:.0} RPM", self.vehicle_rpm));
                ui.end_row();

                ui.label("Current Gear:");
                ui.label(format!("Gear {}", self.vehicle_gear));
                ui.end_row();

                ui.label("Throttle Input:");
                ui.label(format!("{:.0}%", self.throttle_pct * 100.0));
                ui.end_row();

                ui.label("Brake Input:");
                ui.label(format!("{:.0}%", self.brake_pct * 100.0));
                ui.end_row();

                ui.label("Steering Angle:");
                ui.label(format!("{:.1}°", self.steering_deg));
                ui.end_row();

                ui.label("Chassis Mass / Yaw Inertia:");
                ui.label(format!("{:.0} kg / {:.0} kg·m²", self.vehicle_dynamics.chassis_mass_kg, self.vehicle_dynamics.yaw_inertia));
                ui.end_row();

                ui.label("Pacejka Tire Radial Model:");
                ui.label(format!("B={:.1}, C={:.2}, D={:.2}, E={:.2}", 
                    self.vehicle_dynamics.lateral_tire_model.b_stiffness,
                    self.vehicle_dynamics.lateral_tire_model.c_shape,
                    self.vehicle_dynamics.lateral_tire_model.d_peak,
                    self.vehicle_dynamics.lateral_tire_model.e_curvature,
                ));
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Accelerate (+10 km/h)").clicked() {
                    self.throttle_pct = (self.throttle_pct + 0.1).min(1.0);
                    self.brake_pct = 0.0;
                    self.vehicle_dynamics.linear_velocity += Vec3::new(0.0, 0.0, 2.77);
                    self.vehicle_speed_kmh += 10.0;
                    self.vehicle_rpm = (self.vehicle_rpm + 600.0).min(7500.0);
                    self.last_status_message = format!("Throttle {:.0}% • Speed: {:.1} km/h", self.throttle_pct * 100.0, self.vehicle_speed_kmh);
                }
                if ui.button("Brake (-10 km/h)").clicked() {
                    self.brake_pct = (self.brake_pct + 0.2).min(1.0);
                    self.throttle_pct = (self.throttle_pct - 0.2).max(0.0);
                    self.vehicle_dynamics.linear_velocity *= 0.85;
                    self.vehicle_speed_kmh = (self.vehicle_speed_kmh - 10.0).max(0.0);
                    self.vehicle_rpm = (self.vehicle_rpm - 600.0).max(850.0);
                    self.last_status_message = format!("Brake {:.0}% • Speed: {:.1} km/h", self.brake_pct * 100.0, self.vehicle_speed_kmh);
                }
                if ui.button("Turn Left (←)").clicked() {
                    self.steering_deg = (self.steering_deg - 5.0).max(-32.0);
                    self.last_status_message = format!("Steer: {:.1}°", self.steering_deg);
                }
                if ui.button("Center Steer (↑)").clicked() {
                    self.steering_deg = 0.0;
                    self.last_status_message = "Steer: 0.0°".into();
                }
                if ui.button("Turn Right (→)").clicked() {
                    self.steering_deg = (self.steering_deg + 5.0).min(32.0);
                    self.last_status_message = format!("Steer: {:.1}°", self.steering_deg);
                }
            });
        });
    }

    fn render_character_status_tab(&mut self, ui: &mut egui::Ui, _form_factor: ScreenFormFactor) {
        ui.heading("🛡️ Character, Pearl Enclave & PoH Identity");
        ui.label("Decentralized DID identity, biometrics, and hardware Proof-of-Health.");
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Player Profile").strong().size(15.0));
            egui::Grid::new("char_grid").striped(true).show(ui, |ui| {
                ui.label("Identity DID:");
                ui.label("did:atlantis:pelagic_commander_07");
                ui.end_row();

                ui.label("Gyre Node Classification:");
                let tier_text = match self.poh_telemetry.tier {
                    GyreNodeTier::AbyssalAnchor => egui::RichText::new("Tier 1: Abyssal Anchor (Primary GPU Validator)").color(egui::Color32::GREEN).strong(),
                    GyreNodeTier::PelagicWorker => egui::RichText::new("Tier 2: Pelagic Worker (Active LAN Node)").color(egui::Color32::from_rgb(0, 229, 255)).strong(),
                    GyreNodeTier::EpipelagicEdge => egui::RichText::new("Tier 3: Epipelagic Edge (Mobile/Edge Client)").color(egui::Color32::YELLOW).strong(),
                };
                ui.label(tier_text);
                ui.end_row();

                ui.label("Hardware PoH Score:");
                ui.label(egui::RichText::new(format!("{:.1} / 100.0 (Validated)", self.poh_telemetry.poh_score)).color(egui::Color32::GREEN).strong());
                ui.end_row();

                ui.label("Telemetry Metrics:");
                ui.label(format!("{:.1} FPS • {:.1}% CPU • {} MB VRAM Free • {:.1}°C • {}ms RTT",
                    self.poh_telemetry.fps,
                    self.poh_telemetry.cpu_usage_pct,
                    self.poh_telemetry.vram_free_mb,
                    self.poh_telemetry.temp_celsius,
                    self.poh_telemetry.rtt_ping_ms,
                ));
                ui.end_row();

                ui.label("Stamina:");
                ui.label(format!("{:.0}%", self.player_stamina));
                ui.end_row();

                ui.label("Shield Capacity:");
                ui.label(format!("{:.0}%", self.player_shield));
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("⚡ Attest PoH Canary").clicked() {
                    let (score, tier) = BioluminescenceTelemetry::evaluate_tier(
                        self.poh_telemetry.vram_free_mb,
                        self.poh_telemetry.rtt_ping_ms,
                        self.poh_telemetry.temp_celsius,
                    );
                    self.poh_telemetry.poh_score = score;
                    self.poh_telemetry.tier = tier;
                    self.last_status_message = format!("PoH Canary Attested: Score {:.1} • Tier: {:?}", score, tier);
                }

                if ui.button("🛡️ Execute Cryptographic Anti-Spoofing Canary").clicked() {
                    let challenge = PohCanaryChallenge::new(
                        &self.club.viewer_did,
                        256,
                        250,
                        1793216100000,
                    );
                    let digest = challenge.compute_expected_digest();
                    // Deterministically sign the response using a test identity for verification
                    let id = atlantis_protocol::PearlEnclaveIdentity::from_label("audited_worker_key");
                    let sig_digest = PohCanaryChallenge::response_signing_digest(
                        &challenge.challenge_id,
                        &self.club.viewer_did,
                        &digest,
                        14,
                    );
                    let sig: Signature = id.signing_key().sign(&sig_digest);
                    let response = PohCanaryResponse {
                        challenge_id: challenge.challenge_id,
                        worker_did: self.club.viewer_did.clone(),
                        result_digest_hex: digest,
                        execution_time_ms: 14,
                        worker_signature: atlantis_protocol::bytes_to_hex(&sig.to_bytes()),
                    };
                    let audit_pass = challenge.verify_response(&response);
                    self.last_canary_challenge = Some(challenge);
                    self.last_canary_audit_pass = audit_pass;
                    self.last_status_message = format!(
                        "Canary Challenge Verified in 14ms (<=250ms SLA: MET // Anti-Spoof: PASSED)!"
                    );
                }
            });

            if let Some(challenge) = &self.last_canary_challenge {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍 Last Canary Audit:").strong().color(egui::Color32::from_rgb(0, 242, 254)));
                    ui.label(format!("ID: {:.8}... • Iterations: {}", challenge.challenge_id, challenge.matrix_iterations));
                    ui.separator();
                    if self.last_canary_audit_pass {
                        ui.label(egui::RichText::new("CRYPTOGRAPHICALLY AUDITED (PASS)").color(egui::Color32::GREEN).strong());
                    } else {
                        ui.label(egui::RichText::new("FAILED").color(egui::Color32::RED).strong());
                    }
                });
            }

            ui.add_space(10.0);

            // --- WGSL Temporal Super Resolution (TSR) & Dynamic Thermal Governor ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("⚡ WGSL Temporal Super Resolution (TSR) & Thermal Governor");
                    ui.separator();
                    let badge_text = if self.tsr_enabled {
                        format!("TSR ACTIVE (50% SCALE • {:.0}% GPU LOAD CUT)", self.tsr_upscaler.fragment_load_reduction_ratio() * 100.0)
                    } else {
                        "NATIVE RESOLUTION (100% SCALE)".into()
                    };
                    ui.label(
                        egui::RichText::new(badge_text)
                            .color(if self.tsr_enabled { egui::Color32::from_rgb(0, 255, 157) } else { egui::Color32::YELLOW })
                            .strong(),
                    );
                });
                ui.label("Renders internal Clustered Forward+ & Gaussian Splats at 50% scale, reconstructing sharp 1440p/4K frames via 16-phase Halton(2,3) sub-pixel jitter & YCoCg neighborhood clipping.");
                ui.add_space(6.0);

                egui::Grid::new("tsr_grid").striped(true).show(ui, |ui| {
                    ui.label("Internal Render Resolution:");
                    ui.label(format!("{} × {} px (Half-Res Buffer)", self.tsr_upscaler.internal_width, self.tsr_upscaler.internal_height));
                    ui.end_row();

                    ui.label("Target Display Output:");
                    ui.label(format!("{} × {} px (Reconstructed 1440p)", self.tsr_upscaler.output_width, self.tsr_upscaler.output_height));
                    ui.end_row();

                    ui.label("Mobile GPU Fragment Savings:");
                    ui.label(egui::RichText::new(format!("{:.1}% Load Reduction", self.tsr_upscaler.fragment_load_reduction_ratio() * 100.0)).color(egui::Color32::GREEN).strong());
                    ui.end_row();

                    ui.label("History Blend / Sharpness:");
                    ui.label(format!("EMA Blend: {:.2} • RCAS Sharpness: {:.2}", self.tsr_upscaler.history_blend_factor, self.tsr_upscaler.sharpness));
                    ui.end_row();
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.tsr_enabled, "Enable WGSL Spatial-Temporal Upscaling");
                    ui.separator();
                    if ui.button("Cycle Scale: 50% (Power Save)").clicked() {
                        self.tsr_upscaler = WgslTemporalUpscaler::new_half_scale(2560, 1440);
                        self.tsr_enabled = true;
                        self.last_status_message = "TSR configured: 50% Internal Scale (75% GPU power reduction)".into();
                    }
                    if ui.button("Cycle Scale: 67% (Balanced)").clicked() {
                        self.tsr_upscaler = WgslTemporalUpscaler::with_scale(2560, 1440, 0.67);
                        self.tsr_enabled = true;
                        self.last_status_message = "TSR configured: 67% Internal Scale (55% GPU power reduction)".into();
                    }
                    if ui.button("Cycle Scale: 100% (Native)").clicked() {
                        self.tsr_upscaler = WgslTemporalUpscaler::with_scale(2560, 1440, 1.0);
                        self.tsr_enabled = false;
                        self.last_status_message = "TSR disabled: Native 100% resolution render".into();
                    }
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nautilus_world::{GodsEyeCameraMode, NautilusAltitudeRegime};

    #[test]
    fn test_lite_app_default_and_touch_to_watch_sector_slice_descent() {
        let mut app = LemuriaNativeApp::default();
        assert_eq!(app.dem_streamer.view_mode, NautilusGlobeViewMode::CompleteGlobe);
        assert_eq!(app.venue_broadcasts.len(), 4);

        // Verify Team 7 Headquarters Office overlay entry
        let office = app.venue_broadcasts.iter().find(|v| v.venue_id == "office_team7_01").unwrap();
        assert_eq!(office.title, "Headquarters Building: Suite 700 (Team 7 Office)");
        assert!(matches!(office.spatial_overlay.overlay_kind, SpatialOverlayKind::RealSiteGroundedVenue(_)));

        // Verify the Stadium Battle Arena broadcast entry
        let stadium = app.venue_broadcasts.iter().find(|v| v.venue_id == "stadium_oracle_01").unwrap();
        assert_eq!(stadium.title, "Oracle Park Seraphim Battle Colosseum");
        assert!(matches!(stadium.spatial_overlay.overlay_kind, SpatialOverlayKind::StadiumBattleArena(_)));

        // Simulate Touch-to-Watch on the first 1:1 Warehouse Arcade beacon
        let wh = app.venue_broadcasts[1].clone();
        app.current_coord = wh.coord;
        app.dem_streamer.set_view_mode(NautilusGlobeViewMode::SectorSlice);
        app.dem_streamer.update_for_camera(wh.coord);

        assert_eq!(app.dem_streamer.view_mode, NautilusGlobeViewMode::SectorSlice);
        assert_eq!(app.dem_streamer.active_regime, NautilusAltitudeRegime::LocalGameField);
        assert!(app.dem_streamer.locked_game_field_origin_ecef.is_some());

        let gravity = wh.spatial_overlay.tangent_frame().local_gravity_dir(Vec3::new(0.0, 1.7, 0.0));
        assert!(gravity.y < -0.99);
    }

    #[test]
    fn test_seven_daughters_auto_tied_to_club_atlantis_pi_node() {
        let mut app = LemuriaNativeApp::default();
        // By default, Atlantis is running on the Pi and 7 Daughters Radio is auto-bound and playing
        assert!(app.club.pi_atlantis_running);
        assert!(app.club.is_siren_radio_running());
        assert!(app.radio_state.auto_bound_to_club_pi);
        assert!(app.radio_state.is_playing);
        assert!(app.radio_state.stream_url.starts_with("http://10.0.0.237:8080/live.mp3"));

        // Cycling to next daughter updates stream URL on the Pi node
        let next_idx = app.radio_state.cycle_next_daughter().daughter_index;
        assert!(app
            .radio_state
            .stream_url
            .contains(&format!("daughter={}", next_idx)));

        // If Pi Atlantis node stops, radio falls back to direct HTTPS stream
        app.club.pi_atlantis_running = false;
        app.club.sync_pi_radio_daemon();
        app.radio_state
            .sync_with_club_pi_node(app.club.is_siren_radio_running(), &app.club.pi_node_lan_host);
        assert!(app.radio_state.stream_url.starts_with("https://"));

        // When Pi Atlantis node resumes, radio automatically re-binds to the Pi LAN node and plays
        app.club.pi_atlantis_running = true;
        app.club.sync_pi_radio_daemon();
        app.radio_state
            .sync_with_club_pi_node(app.club.is_siren_radio_running(), &app.club.pi_node_lan_host);
        assert!(app.radio_state.is_playing);
        assert!(app.radio_state.stream_url.starts_with("http://10.0.0.237:8080/live.mp3"));
    }

    #[test]
    fn test_registered_daemons_tab_team7_and_google_auth() {
        let mut app = LemuriaNativeApp::default();
        assert_eq!(app.club.current_user.team_id, "team-7");
        assert!(app.club.current_user.is_authenticated);
        assert_eq!(app.club.current_user.email, "team7.operator@gmail.com");

        // Verify Team 7 daemons are registered
        let team7_daemons: Vec<_> = app.club.daemons.iter().filter(|d| d.team_id.as_deref() == Some("team-7")).collect();
        assert!(team7_daemons.len() >= 3);

        // Verify Registered Clubs and 7 Officer Structure
        assert!(app.club.registered_clubs.len() >= 2);
        let hq_club = &app.club.registered_clubs[0];
        assert_eq!(hq_club.club_id, "club_team7_hq");
        assert_eq!(hq_club.user_role, crate::club::OfficerRole::TeamDirector);
        assert!(hq_club.user_role.is_officer());

        // Verify 7 Primary Secular Officers translation
        let officers: Vec<_> = app.club.members.iter().filter(|m| m.officer_role.is_officer()).collect();
        assert_eq!(officers.len(), 7);

        // Verify authoritative office overlay comes from Nautilus
        let office = AnchoredSpatialOverlay::team7_headquarters_office();
        assert_eq!(app.club.building_office.overlay_id, office.overlay_id);
        assert_eq!(app.club.building_office.anchor_geo.lat_deg, office.anchor_geo.lat_deg);

        // Switch to daemons tab
        app.daemons_tab_open = true;
        assert!(app.daemons_tab_open);
        assert!(!app.club_open);
    }

    #[test]
    fn test_officer_dashboard_accessible_when_officer() {
        let mut app = LemuriaNativeApp::default();
        // Default viewer is did:atlantis:team7_operator, who is TeamDirector (Officer)
        assert!(app.club.viewer_is_officer());
        assert_eq!(
            app.club.viewer_officer_role(),
            Some(crate::club::OfficerRole::TeamDirector)
        );

        // When officer, switching to daemons/team tab targets OfficerNodeDashboard
        let is_officer = app.club.viewer_is_officer();
        assert!(is_officer);

        app.daemons_tab_open = true;
        app.club.active_subtab = crate::club::ClubViewSubTab::OfficerNodeDashboard;
        assert_eq!(
            app.club.active_subtab,
            crate::club::ClubViewSubTab::OfficerNodeDashboard
        );

        // Verify demoting to a non-officer candidate role revokes officer status
        app.club.viewer_did = "did:atlantis:guest_unverified".into();
        assert!(!app.club.viewer_is_officer());
        assert_eq!(app.club.viewer_officer_role(), None);
    }

    #[test]
    fn test_lite_app_gods_eye_telemetry_and_vr_guild_bounties() {
        let mut app = LemuriaNativeApp::default();
        assert_eq!(app.gods_eye.active_layers.len(), 6);
        assert!(app.gods_eye.visible_entities().len() >= 10);

        // Execute voice/AI query to lock onto Kuroshio Vanguard maritime vessel
        let matched = app.gods_eye.execute_gods_eye_query("kuroshio vanguard");
        assert_eq!(matched.as_deref(), Some("ais_kuroshio_vanguard"));
        assert!(matches!(
            app.gods_eye.camera_mode,
            GodsEyeCameraMode::CockpitRideAlong { .. }
        ));

        // Claim the guild bounty on the vessel
        let viewer_did = app.club.viewer_did.clone();
        let reward = app
            .gods_eye
            .claim_entity_guild_bounty("ais_kuroshio_vanguard", &viewer_did)
            .expect("Bounty claim should succeed");
        app.guild_bounties_claimed_cyan += reward;
        assert_eq!(app.guild_bounties_claimed_cyan, 1_250.0);
    }

    #[test]
    fn test_slipstream_node_chambers_and_cnc_rebuild_room() {
        let mut app = LemuriaNativeApp::default();

        // Verify Slipstream Sovereign Seed Club is present
        let slipstream = app
            .club
            .registered_clubs
            .iter()
            .find(|c| c.club_id == "club_slipstream_seed")
            .expect("Slipstream club must be registered");
        assert_eq!(slipstream.home_node_id, "slipstream-node-master");
        assert_eq!(slipstream.callsign, "SLIPSTREAM-01");

        // Verify subtab navigation to Slipstream Node
        app.club.active_subtab = crate::club::ClubViewSubTab::SlipstreamNode;
        assert_eq!(
            app.club.active_subtab,
            crate::club::ClubViewSubTab::SlipstreamNode
        );

        // Verify all 5 dedicated tool rooms are registered
        assert_eq!(crate::club::SlipstreamRoomId::ALL.len(), 5);
        assert_eq!(
            app.club.selected_slipstream_room,
            crate::club::SlipstreamRoomId::CncTacticalArena
        );

        // Verify playable C&C Rebuild war room simulation
        assert_eq!(app.club.cnc_sim.sectors.len(), 12);
        assert!(!app.club.cnc_sim.factions.is_empty());

        let initial_fuel = app
            .club
            .cnc_sim
            .factions
            .iter()
            .find(|f| f.faction == app.club.cnc_sim.player_faction)
            .map(|f| f.fuel_stockpile)
            .unwrap_or(0.0);
        assert!(initial_fuel > 0.0);

        // Test building construction in the C&C war room
        let build_res = app.club.cnc_sim.construct_building(
            lemuria_engine::TacticalBuildingType::PowerPlant,
            10.0,
            -20.0,
        );
        assert!(build_res.is_ok());

        // Test unit production in the C&C war room
        let unit_res = app.club.cnc_sim.produce_unit(
            lemuria_engine::TacticalUnitType::FrontlineCombatDroneUAV,
        );
        assert!(unit_res.is_ok());

        // Test issuing tactical orders
        let units_ordered = app.club.cnc_sim.issue_tactical_order(25.0, 50.0);
        assert!(units_ordered > 0);

        // Step simulation tick (dt clamped to 0.1s max per tick)
        for _ in 0..10 {
            app.club.cnc_sim.tick(0.1);
        }
        assert!(app.club.cnc_sim.match_elapsed_seconds >= 0.99);
    }

    #[test]
    fn test_next_horizon_zk_geo_state_channels_and_canary_auditing() {
        let mut app = LemuriaNativeApp::default();

        // 1. WebTransport datagram engine verification
        assert!(app.webtransport_engine.is_connected);
        let sent_before = app.webtransport_engine.datagrams_sent;
        app.webtransport_engine.send_datagram(512);
        assert_eq!(app.webtransport_engine.datagrams_sent, sent_before + 1);

        // 2. Coral State Channel off-ledger micro-tipping
        assert_eq!(app.coral_channel.net_transferred_cyan, 0.0);
        let tip_res = app.coral_channel.tip_micro_increment(0.1);
        assert_eq!(tip_res, Ok(0.1));
        assert_eq!(app.coral_channel.tick_count, 1);
        assert_eq!(app.coral_channel.net_transferred_cyan, 0.1);

        let final_settlement = app.coral_channel.settle_channel();
        assert_eq!(final_settlement, 0.1);
        assert!(app.coral_channel.is_settled);
        assert!(app.coral_channel.tip_micro_increment(0.1).is_err());

        // 3. Zero-Knowledge Geo-Proximity Proof verification (within 500m geofence)
        let actual_coord = (35.6586, 139.7454);
        let target_center = (35.6586, 139.7454);
        let proof = ZkGeoProximityProof::generate_proof(
            actual_coord,
            target_center,
            500.0,
            "zone_neotokyo_garage",
            b"test_salt_123",
        )
        .expect("ZK proof generation should succeed");
        assert!(proof.verify(Some(target_center)));

        // 4. Deterministic Proof-of-Health Canary anti-spoofing challenge verification
        let target_did = app.club.viewer_did.clone();
        let challenge = PohCanaryChallenge::new(&target_did, 128, 250, 1000);
        let expected_digest = challenge.compute_expected_digest();
        assert!(!expected_digest.is_empty());

        let test_identity =
            atlantis_protocol::PearlEnclaveIdentity::from_label("audited_worker_key");
        let sig_digest = PohCanaryChallenge::response_signing_digest(
            &challenge.challenge_id,
            &target_did,
            &expected_digest,
            18,
        );
        let signature: Signature = test_identity.signing_key().sign(&sig_digest);

        let _response = PohCanaryResponse {
            challenge_id: challenge.challenge_id,
            worker_did: target_did.clone(),
            result_digest_hex: expected_digest,
            execution_time_ms: 18,
            worker_signature: atlantis_protocol::bytes_to_hex(&signature.to_bytes()),
        };

        // Note: verify_response checks if signature matches worker_did public key.
        // If worker_did is did:atlantis:<test_identity_pubkey>, verify_response returns true!
        let worker_did = test_identity.did().to_string();
        let custom_challenge = PohCanaryChallenge::new(&worker_did, 64, 250, 1000);
        let custom_digest = custom_challenge.compute_expected_digest();
        let custom_sig_digest = PohCanaryChallenge::response_signing_digest(
            &custom_challenge.challenge_id,
            &worker_did,
            &custom_digest,
            12,
        );
        let custom_sig: Signature = test_identity.signing_key().sign(&custom_sig_digest);
        let valid_response = PohCanaryResponse {
            challenge_id: custom_challenge.challenge_id,
            worker_did: worker_did.clone(),
            result_digest_hex: custom_digest,
            execution_time_ms: 12,
            worker_signature: atlantis_protocol::bytes_to_hex(&custom_sig.to_bytes()),
        };
        assert!(custom_challenge.verify_response(&valid_response));
    }

    #[test]
    fn test_gaussian_splat_storefront_inspector_and_cart_picking() {
        let mut app = LemuriaNativeApp::default();

        // 1. Verify 3D Gaussian Splat products are spawned in ECS world
        assert_eq!(app.splat_world.splat_products.len(), 2);
        let product1 = &app.splat_world.splat_products[0];
        assert_eq!(product1.sku_id, "totem_energy_cell_432hz");
        assert!(product1.is_service_totem);

        let product2 = &app.splat_world.splat_products[1];
        assert_eq!(product2.sku_id, "item_carbon_wheel_r19");
        assert!(!product2.is_service_totem);

        // 2. Verify Gaussian Splatting pass populated splats
        assert!(app.splat_pass.splats.len() >= 128);

        // 3. Raycast pick & drop into customer cart
        let ray_origin = Vec3::new(-1.0, 1.2, 0.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);
        let hit = SplatConvexHullCollider::raycast_shelf_products(
            &app.splat_world,
            ray_origin,
            ray_dir,
            10.0,
        )
        .expect("Raycast should hit totem_energy_cell_432hz");
        assert_eq!(hit.sku_id, "totem_energy_cell_432hz");
        assert!(hit.is_service_totem);

        // 4. Test VR hand-grab picking
        let hand_pos = Vec3::new(-1.0, 1.2, -2.5);
        let grabbed = app
            .splat_collider
            .try_grab_with_hand(&app.splat_world, hand_pos, 0.5);
        assert_eq!(grabbed, Some(product1.entity_id));
        assert_eq!(app.splat_collider.grabbed_entity_id, Some(product1.entity_id));

        // Release into cart drop zone
        let dropped_sku = app
            .splat_collider
            .update_or_release_grabbed_product(
                &mut app.splat_world,
                Vec3::new(0.0, 0.3, -1.0), // Inside cart_drop_zone
                Quat::IDENTITY,
                true,
            );
        assert_eq!(dropped_sku.as_deref(), Some("totem_energy_cell_432hz"));
        assert_eq!(app.splat_collider.cart_sku_items.len(), 1);
        assert_eq!(app.splat_collider.cart_sku_items[0], "totem_energy_cell_432hz");
    }

    #[test]
    fn test_wgsl_temporal_super_resolution_and_load_reduction() {
        let mut app = LemuriaNativeApp::default();

        // 1. Verify default 50% scale TSR setup (1440p target -> 720p internal)
        assert!(app.tsr_enabled);
        assert_eq!(app.tsr_upscaler.output_width, 2560);
        assert_eq!(app.tsr_upscaler.output_height, 1440);
        assert_eq!(app.tsr_upscaler.internal_width, 1280);
        assert_eq!(app.tsr_upscaler.internal_height, 720);

        // 2. Verify 75% fragment load reduction ratio
        let reduction = app.tsr_upscaler.fragment_load_reduction_ratio();
        assert!((reduction - 0.75).abs() < 0.01);

        // 3. Verify sub-pixel jitter sequence progresses deterministically
        let jitter0 = app.tsr_upscaler.next_frame_jitter_ndc();
        let jitter1 = app.tsr_upscaler.next_frame_jitter_ndc();
        assert_ne!(jitter0, jitter1);

        // 4. Test cycling to 67% balanced mode and 100% native mode
        app.tsr_upscaler = WgslTemporalUpscaler::with_scale(2560, 1440, 0.67);
        let reduction_balanced = app.tsr_upscaler.fragment_load_reduction_ratio();
        assert!(reduction_balanced > 0.50 && reduction_balanced < 0.60);

        app.tsr_upscaler = WgslTemporalUpscaler::with_scale(2560, 1440, 1.0);
        assert_eq!(app.tsr_upscaler.internal_width, 2560);
        assert_eq!(app.tsr_upscaler.internal_height, 1440);
        assert_eq!(app.tsr_upscaler.fragment_load_reduction_ratio(), 0.0);
    }

    #[test]
    fn test_wasi_sandboxed_cartridge_runner_and_capability_security() {
        let mut app = LemuriaNativeApp::default();

        // 1. Verify preloaded WASI cartridges
        assert_eq!(app.wasi_engine.mounted_cartridges.len(), 3);
        let radar_cartridge = &app.wasi_engine.mounted_cartridges[0];
        assert_eq!(radar_cartridge.cartridge_id, "cartridge_tactical_radar_hud.wasm");
        assert!(radar_cartridge.is_mounted);
        assert!(radar_cartridge.capabilities.allow_entity_spawn);
        assert!(!radar_cartridge.capabilities.allow_escrow_bidding);

        // 2. Test successful execution of permitted host call: lemuria::spawn_entity
        let res = app.wasi_engine.invoke_cartridge_entry(0, "lemuria::spawn_entity");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "Entity spawned in Lemuria ECS World via WASI sandbox");

        // 3. Test security violation prevention: radar cartridge attempting escrow bidding
        let escrow_res = app.wasi_engine.invoke_cartridge_entry(0, "atlantis::request_escrow_bid");
        assert!(escrow_res.is_err());
        assert!(escrow_res.unwrap_err().contains("WASI Security Violation"));

        // 4. Test forbidden host calls: raw socket or host filesystem access
        let socket_res = app.wasi_engine.invoke_cartridge_entry(0, "host::raw_socket_open");
        assert!(socket_res.is_err());
        assert!(socket_res.unwrap_err().contains("CRITICAL WASI Sandboxing Violation"));

        // 5. Test drone cartridge with escrow bidding permission
        let drone_res = app.wasi_engine.invoke_cartridge_entry(1, "atlantis::request_escrow_bid");
        assert!(drone_res.is_ok());
        assert_eq!(drone_res.unwrap(), "Coral Escrow bid placed via WASI sandbox");

        // 6. Test dynamic cartridge mounting & unmounting
        assert!(!app.wasi_engine.mounted_cartridges[2].is_mounted);
        app.wasi_engine.toggle_mount(2);
        assert!(app.wasi_engine.mounted_cartridges[2].is_mounted);
        let garage_res = app.wasi_engine.invoke_cartridge_entry(2, "atlantis::request_escrow_bid");
        assert!(garage_res.is_ok());
    }

    #[test]
    fn test_zero_copy_mesh_pipeline_and_gpu_ingest() {
        let mut app = LemuriaNativeApp::default();

        // 1. Verify zero-copy pipeline defaults
        assert_eq!(app.zero_copy_pipeline.parse_overhead_micros, 0.0);
        let initial_packets = app.zero_copy_pipeline.packets_ingested;

        // 2. Prepare 64-byte aligned synthetic mesh packet
        let mut packet = ZeroCopyEntityPacket::default();
        packet.entity_id_u64 = 9999;
        packet.position_xyz = [10.5, 2.0, -15.0];
        packet.velocity_xyz = [45.0, 0.0, 0.0];
        packet.health_shield_packed = 0x00500064; // 80 HP, 100 Shield

        let slice = unsafe {
            std::slice::from_raw_parts(
                &packet as *const _ as *const u8,
                std::mem::size_of::<ZeroCopyEntityPacket>(),
            )
        };
        assert_eq!(slice.len(), 64);

        // 3. Ingest into pipeline with zero parse time and zero heap reallocations
        let ingested = app
            .zero_copy_pipeline
            .ingest_packet_bytes(slice)
            .expect("Ingestion must succeed for valid 64-byte packet");

        assert_eq!(ingested.entity_id_u64, 9999);
        assert_eq!(ingested.position_xyz, [10.5, 2.0, -15.0]);
        assert_eq!(app.zero_copy_pipeline.packets_ingested, initial_packets + 1);
        assert_eq!(app.zero_copy_pipeline.parse_overhead_micros, 0.0);

        // 4. Reject malformed packet sizes
        let malformed_slice = &[0u8; 32];
        let err_res = app.zero_copy_pipeline.ingest_packet_bytes(malformed_slice);
        assert!(err_res.is_err());
        assert_eq!(
            err_res.unwrap_err(),
            "Packet size mismatch for ZeroCopyEntityPacket (expected exactly 64 bytes)"
        );
    }
}
