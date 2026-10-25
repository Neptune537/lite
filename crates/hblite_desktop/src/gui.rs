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

use atlantis_protocol::{BioluminescenceTelemetry, GyreNodeTier};
use eframe::egui;
use glam::Vec3;
use lemuria_engine::{
    AcousticRoomProfile, DeterministicPhysicsStepper, DoubleBufferedFrameAllocator,
    PacejkaVehicleDynamics, SirenAcousticEngine,
};
use lemuria_leptos_client::{
    MarketplaceItem, MarketplaceTab, TheaterRadioState, VirtualStorefrontState,
};
use nautilus_world::{TacticalRouteSpline, Wgs84GeoCoord};

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
}

pub struct LemuriaNativeApp {
    pub selected_tab: MarketplaceTab,
    pub club_open: bool,
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

    // Real Nautilus World Geodesic Navigation
    pub current_coord: Wgs84GeoCoord,
    pub active_route: TacticalRouteSpline,

    // Live Atlantis Protocol Telemetry
    pub poh_telemetry: BioluminescenceTelemetry,

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

        let mut radio_state = TheaterRadioState::default();
        radio_state.stream_url = "https://stream.nightride.fm/nightride.mp3".into();
        radio_state.is_playing = false;

        Self {
            selected_tab: MarketplaceTab::TacticalMap,
            club_open: false,
            club: ClubState::default(),
            catalog: MarketplaceItem::default_catalog(),
            radio_state,
            storefront,
            last_status_message: "LEMURIA V2 INITIALIZED • ALL ARCHITECTURE PILLARS MOUNTED".into(),
            tip_counter: 0,
            venue_broadcasts: vec![
                LiveVenueBroadcast {
                    venue_id: "venue_wh_01".into(),
                    title: "Sector-7 Old Iron Warehouse Arcade".into(),
                    venue_type_label: "🏭 1:1 Warehouse Virtual Arcade".into(),
                    coord: Wgs84GeoCoord::new(35.6528, 139.7532, 12.0),
                    broadcaster_callsign: "Jules Vance (VR)".into(),
                    stream_uri: "webtransport://relay.kuroshio.mesh:4433/broadcast/wh_01".into(),
                    rt60_seconds: 1.45,
                    active_viewers: 42,
                },
                LiveVenueBroadcast {
                    venue_id: "venue_park_02".into(),
                    title: "Yoyogi Tactical Field & Holo-Park".into(),
                    venue_type_label: "🌳 1:1 Public Municipal Park Arena".into(),
                    coord: Wgs84GeoCoord::new(35.6717, 139.6949, 38.0),
                    broadcaster_callsign: "Seraph-9 [Mech Pilot]".into(),
                    stream_uri: "webtransport://relay.kuroshio.mesh:4433/broadcast/park_02".into(),
                    rt60_seconds: 0.25,
                    active_viewers: 118,
                },
            ],
            active_spectator_session: None,
            frame_allocator,
            physics_stepper,
            siren_engine,
            vehicle_dynamics,
            current_coord,
            active_route,
            poh_telemetry,
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
        self.frame_allocator.flip_frame();

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
                                egui::RichText::new("⏸ HEAVENLYBOND")
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
                            ui.label(egui::RichText::new("[LEMURIA ENGINE]").color(egui::Color32::LIGHT_GRAY).small());
                        });
                    });
                }
                _ => {
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new("⏸ HEAVENLYBOND // PAUSE MENU")
                                .color(egui::Color32::from_rgb(0, 229, 255))
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("[LEMURIA V2 ARCHITECTURE • ZERO FLUTTER]")
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
                    ui.label(egui::RichText::new(format!("Alloc: {} KB | 60Hz Rapier | {:.0}x{:.0} px", heap_kb, screen_width, screen_height)).small().color(egui::Color32::GRAY));
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
            let is_selected = !self.club_open && self.selected_tab == tab;
            if ui.selectable_label(is_selected, label).clicked() {
                self.selected_tab = tab;
                self.club_open = false;
                self.last_status_message = format!("Tab switched: {}", full_label);
            }
        }

        let club_label = if horizontal { "👥 Club" } else { "👥 My Club" };
        if ui.selectable_label(self.club_open, club_label).clicked() {
            self.club_open = true;
            self.last_status_message = "Tab switched: 👥 My Club".into();
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
                ui.label(egui::RichText::new("Nautilus DemTerrainTileStreamer (LOD 15)").color(egui::Color32::GREEN));
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
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
                if ui.button("🌐 Open External Turn-by-Turn GPS").clicked() {
                    if let Some(url) = self.active_route.destination_google_maps_url() {
                        self.last_status_message = format!("Generated Navigation URL: {}", url);
                    }
                }
            });
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
            ui.label("Touch any real-world site beacon to descend from orbit into 500m Karman SectorSlice LOD & watch in-situ VR gameplay.");
            ui.add_space(6.0);

            let venues = self.venue_broadcasts.clone();
            for venue in venues {
                let dist_km = self.current_coord.haversine_distance_meters(&venue.coord) / 1000.0;
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
                            ui.label(format!("Anchor: {:.4}°, {:.4}°", venue.coord.lat_deg, venue.coord.lon_deg));
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("🥽 Touch to Watch (Descend to Venue)").strong().color(egui::Color32::BLACK))
                                .clicked()
                            {
                                self.current_coord = venue.coord;
                                self.active_route = TacticalRouteSpline::new(vec![self.current_coord, venue.coord]);
                                self.active_spectator_session = Some(SpectatorSessionState {
                                    venue_id: venue.venue_id.clone(),
                                    title: venue.title.clone(),
                                    broadcaster_callsign: venue.broadcaster_callsign.clone(),
                                    stream_uri: venue.stream_uri.clone(),
                                    elapsed_seconds: 12.4,
                                    total_tips_cyan: 0.0,
                                    camera_mode: "Third-Person Orbit (Dual-Quat Kimodo)".into(),
                                });
                                self.last_status_message = format!("Touched venue beacon! Joined spectator stream: {}", venue.title);
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
                        ui.label(format!("Camera Angle: {} | Active RT60 Ingest: 48kHz Stereo", session.camera_mode));

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("🪙 Tip Broadcaster 0.1 CYAN").clicked() {
                                session.total_tips_cyan += 0.1;
                                self.tip_counter += 1;
                                self.active_spectator_session = Some(session.clone());
                                self.last_status_message = format!("Tipped 0.1 CYAN to {} via Tide Ledger State Channel!", session.broadcaster_callsign);
                            }
                            ui.label(egui::RichText::new(format!("Tipped This Session: {:.1} CYAN", session.total_tips_cyan)).color(egui::Color32::from_rgb(255, 215, 0)));

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
        ui.group(|ui| {
            ui.heading("Active Nautilus Quests in Sector");
            ui.label("• [Bounty] Escort Pelagic Transport to Kuroshio Gate (Reward: 450 CYAN)");
            ui.label("• [Contract] Perform Sabine RT60 acoustic survey at Totem #04 (Reward: 120 USDC)");
        });
    }

    fn render_theater_radio_tab(&mut self, ui: &mut egui::Ui, _form_factor: ScreenFormFactor) {
        ui.heading("📻 Theater 24/7 Live Broadcast & Siren Radio");
        ui.label("Direct low-latency HTTP live stream connected to Sabine RT60 acoustic decoder.");
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
                for i in 1..=7 {
                    let names = ["Maia", "Alcyone", "Asterope", "Celaeno", "Electra", "Taygete", "Merope"];
                    let urls = [
                        "https://stream.nightride.fm/chillsynth.mp3", // Maia: 432 Hz Dawn Oceanic Ambient
                        "https://stream.nightride.fm/nightride.mp3",  // Alcyone: 528 Hz Cyberpunk D&B / Synthwave
                        "https://ice2.somafm.com/secretagent-128-mp3", // Asterope: 639 Hz Tactical Deep House
                        "https://stream.nightride.fm/darksynth.mp3",   // Celaeno: 741 Hz Glitch / Industrial Bass
                        "https://stream.nightride.fm/datawave.mp3",    // Electra: 852 Hz Vocal Trance / Eurobeat
                        "https://ice2.somafm.com/groovesalad-128-mp3", // Taygete: 963 Hz Darkwave / Abyssal Drone
                        "https://radio.plaza.one/mp3",                 // Merope: 396 Hz Lo-Fi Starlight Harbor
                    ];
                    let is_active = self.radio_state.active_daughter.daughter_index == i;
                    let btn_text = format!("{}: {}", i, names[(i - 1) as usize]);
                    if ui.selectable_label(is_active, btn_text).clicked() {
                        self.radio_state.set_daughter_by_index(i);
                        self.radio_state.stream_url = urls[(i - 1) as usize].to_string();
                        self.last_status_message = format!("Radio switched to Daughter #{}: {} ({})", i, names[(i - 1) as usize], self.radio_state.stream_url);
                        #[cfg(target_arch = "wasm32")]
                        if self.radio_state.is_playing {
                            crate::play_radio_stream(&self.radio_state.stream_url, self.radio_state.volume);
                        }
                    }
                }
                if ui.button("🔄 Cycle Next Shift").clicked() {
                    let d = self.radio_state.cycle_next_daughter();
                    self.last_status_message = format!("Shift rotated to Daughter #{}: {}", d.daughter_index, d.name);
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
                    self.last_status_message = format!("Tip #{} sent (0.1 CYAN off-ledger Coral state channel)", self.tip_counter);
                }
                if ui.button("🎙️ Push-To-Talk (PTT Active)").clicked() {
                    self.last_status_message = "Transmitting voice over P2P WebRTC mesh channel 104.7...".into();
                }
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
            });
        });
    }
}
