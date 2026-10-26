//! Club and Enclave Governance module:
//! - Registered clubs and lodges the user belongs to.
//! - Translation of Freemason lodge structure and by-laws into secular team/club governance
//!   without occult-natured titles, featuring the 7 primary governance officers:
//!     1. Team Director (Presiding Officer)
//!     2. First Vice Director (Operations & Strategy)
//!     3. Second Vice Director (Welfare, Morale & Harmony)
//!     4. Secretary (Records, Minutes, DID Registrations)
//!     5. Treasurer (Tide Ledger, Coral Escrow & Dues)
//!     6. Chief Marshal (Ceremonies, Protocol & Communications)
//!     7. Outer Guard / Sentinel (Perimeter Security, Canary Audits & Entry Verification)
//! - Dedicated tab for user's registered daemons.
//! - Office data authoritative from `nautilus_world::AnchoredSpatialOverlay::team7_headquarters_office()`.

use eframe::egui;
use nautilus_world::{AnchoredSpatialOverlay, Wgs84GeoCoord};

/// The Seven Primary Governance Officers translated from the lodge operational hierarchy
/// into a secular, meritocratic guild framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficerRole {
    /// 1. Presiding Officer (Directs meetings, preserves order, signs official warrants).
    TeamDirector,
    /// 2. First Vice Director (Directs tactical operations, manages resources and schedules).
    FirstViceDirector,
    /// 3. Second Vice Director (Oversees member welfare, discord mediation, and fellowship).
    SecondViceDirector,
    /// 4. Secretary (Custody of charter records, Pearl DIDs, meeting minutes, and correspondence).
    Secretary,
    /// 5. Treasurer (Custody of funds, Coral escrow reconciliations, and double-entry accounting).
    Treasurer,
    /// 6. Chief Marshal (Coordinates inter-node protocol, introductions, and orderly communications).
    ChiefMarshal,
    /// 7. Outer Guard / Sentinel (Hardware attestation, edge verification, and admission control).
    Sentinel,
    /// General Member without an active officer chair.
    GuildMember,
    /// Guest / Candidate under evaluation.
    CandidateGuest,
}

impl OfficerRole {
    pub const SEVEN_OFFICERS: [OfficerRole; 7] = [
        OfficerRole::TeamDirector,
        OfficerRole::FirstViceDirector,
        OfficerRole::SecondViceDirector,
        OfficerRole::Secretary,
        OfficerRole::Treasurer,
        OfficerRole::ChiefMarshal,
        OfficerRole::Sentinel,
    ];

    pub const ALL: [OfficerRole; 9] = [
        OfficerRole::TeamDirector,
        OfficerRole::FirstViceDirector,
        OfficerRole::SecondViceDirector,
        OfficerRole::Secretary,
        OfficerRole::Treasurer,
        OfficerRole::ChiefMarshal,
        OfficerRole::Sentinel,
        OfficerRole::GuildMember,
        OfficerRole::CandidateGuest,
    ];

    pub fn title(self) -> &'static str {
        match self {
            OfficerRole::TeamDirector => "Team Director (Presiding Officer)",
            OfficerRole::FirstViceDirector => "First Vice Director (Operations)",
            OfficerRole::SecondViceDirector => "Second Vice Director (Welfare & Harmony)",
            OfficerRole::Secretary => "Secretary (Records & Registry)",
            OfficerRole::Treasurer => "Treasurer (Finance & Coral Escrow)",
            OfficerRole::ChiefMarshal => "Chief Marshal (Protocol & Herald)",
            OfficerRole::Sentinel => "Sentinel (Outer Guard & Entry Sentry)",
            OfficerRole::GuildMember => "Guild Member",
            OfficerRole::CandidateGuest => "Candidate / Guest",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            OfficerRole::TeamDirector => "Director",
            OfficerRole::FirstViceDirector => "1st Vice Dir",
            OfficerRole::SecondViceDirector => "2nd Vice Dir",
            OfficerRole::Secretary => "Secretary",
            OfficerRole::Treasurer => "Treasurer",
            OfficerRole::ChiefMarshal => "Marshal",
            OfficerRole::Sentinel => "Sentinel",
            OfficerRole::GuildMember => "Member",
            OfficerRole::CandidateGuest => "Candidate",
        }
    }

    pub fn duty_description(self) -> &'static str {
        match self {
            OfficerRole::TeamDirector => "Convenes and presides over assembly sessions; signs official guild warrants and authorizes node graduations.",
            OfficerRole::FirstViceDirector => "Oversees operational labor and tactical execution; acts as presiding officer in the Director's absence.",
            OfficerRole::SecondViceDirector => "Fosters fellowship, monitors team morale, and supervises recess periods and community events.",
            OfficerRole::Secretary => "Maintains the official roll of members, registers DID credentials, and files accurate session logs.",
            OfficerRole::Treasurer => "Manages double-entry ledger accounts, disburses verified escrow releases, and collects platform dues.",
            OfficerRole::ChiefMarshal => "Enforces parliamentary decorum, conducts protocol intros, and coordinates communications relays.",
            OfficerRole::Sentinel => "Guards the network perimeter, executes <=250ms canary challenges, and inspects arriving credentials.",
            OfficerRole::GuildMember => "Participates in guild deliberations, casts votes on resolutions, and contributes to task pools.",
            OfficerRole::CandidateGuest => "Prospective member undergoing identity and Proof-of-Health verification prior to graduation.",
        }
    }

    pub fn is_officer(self) -> bool {
        matches!(
            self,
            OfficerRole::TeamDirector
                | OfficerRole::FirstViceDirector
                | OfficerRole::SecondViceDirector
                | OfficerRole::Secretary
                | OfficerRole::Treasurer
                | OfficerRole::ChiefMarshal
                | OfficerRole::Sentinel
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Suspended,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Suspended => "suspended",
        }
    }
}

/// A registered member of a team or club.
#[derive(Debug, Clone)]
pub struct Member {
    pub did: String,
    pub name: String,
    pub officer_role: OfficerRole,
    pub status: Status,
    pub date_joined: &'static str,
}

/// A registered club / guild enclave that the user belongs to.
#[derive(Debug, Clone)]
pub struct RegisteredClub {
    pub club_id: String,
    pub name: String,
    pub callsign: String,
    pub home_node_id: String,
    pub office_geo: Wgs84GeoCoord,
    pub suite_label: String,
    pub user_role: OfficerRole,
    pub member_count: usize,
    pub active_daemons: usize,
    pub charter_bylaws_summary: &'static str,
    pub is_primary: bool,
}

/// An autonomous AI daemon bound to a club node and user DID session.
#[derive(Debug, Clone)]
pub struct Daemon {
    pub name: String,
    pub home_node: String,
    pub running: bool,
    /// Team or owner ID registered to this daemon (e.g. "team-7" or DID).
    pub team_id: Option<String>,
    pub registered_to: Option<String>,
    pub role_description: Option<String>,
    pub ping_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthAccount {
    pub email: String,
    pub display_name: String,
    pub team_id: String,
    pub user_did: String,
    pub provider: String,
    pub is_authenticated: bool,
}

impl Default for AuthAccount {
    fn default() -> Self {
        Self {
            email: "team7.operator@gmail.com".into(),
            display_name: "Team 7 Operator".into(),
            team_id: "team-7".into(),
            user_did: "did:atlantis:team7_operator".into(),
            provider: "Google OAuth2 (OpenID Connect)".into(),
            is_authenticated: true,
        }
    }
}

/// Sub-view navigation inside the Clubs & Governance command space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClubViewSubTab {
    /// List and inspect all registered clubs/lodges the user belongs to.
    RegisteredClubs,
    /// Team 7 Building Office and secular bylaws based on the 7 officers structure.
    Team7OfficeAndBylaws,
    /// Dedicated register of AI Daemons bound to the user's account and home nodes.
    RegisteredDaemons,
}

pub struct ClubState {
    /// DID of the local user; officer status is derived from the roster.
    pub viewer_did: String,
    pub current_user: AuthAccount,
    pub active_subtab: ClubViewSubTab,
    pub registered_clubs: Vec<RegisteredClub>,
    pub selected_club_id: String,
    pub home_nodes: Vec<String>,
    pub members: Vec<Member>,
    pub daemons: Vec<Daemon>,
    /// Building office data coming authoritatively from Nautilus.
    pub building_office: AnchoredSpatialOverlay,
    /// LAN IP:Port of the Club's Atlantis Raspberry Pi node on the local network.
    pub pi_node_lan_host: String,
    /// True whenever the Atlantis Home Node process is active on the Raspberry Pi.
    pub pi_atlantis_running: bool,
    /// Filter for Team 7 registered daemons only in Daemons tab
    pub team_filter_only: bool,
    /// Input buffer for new daemon registration
    pub new_daemon_name: String,
    pub new_daemon_node: String,
}

impl Default for ClubState {
    fn default() -> Self {
        let auth = AuthAccount::default();
        let office = AnchoredSpatialOverlay::team7_headquarters_office();
        let office_geo = office.anchor_geo;

        let primary_club = RegisteredClub {
            club_id: "club_team7_hq".into(),
            name: "Team 7 Headquarters Enclave".into(),
            callsign: "TEAM-7-HQ".into(),
            home_node_id: "team7-node-alpha".into(),
            office_geo,
            suite_label: "Suite 700 (Executive Triad Enclave)".into(),
            user_role: OfficerRole::TeamDirector,
            member_count: 7,
            active_daemons: 4,
            charter_bylaws_summary: "Secular Guild Charter of Team 7: Governed by the Seven Officers. Enforces strict zero-leak privacy vaults, democratic consensus, and automated Proof-of-Health sentry checks.",
            is_primary: true,
        };

        let secondary_club = RegisteredClub {
            club_id: "club_atlantis_pi".into(),
            name: "Club Atlantis Home Node Guild".into(),
            callsign: "ATLANTIS-PI".into(),
            home_node_id: "atlantis-pi-home-1".into(),
            office_geo: Wgs84GeoCoord::new(35.6601, 139.7290, 35.0),
            suite_label: "Harbor Station 1 (Pi Radio Broadcast Hub)".into(),
            user_role: OfficerRole::Treasurer,
            member_count: 14,
            active_daemons: 2,
            charter_bylaws_summary: "Siren Radio & Home Node Harbor: Broadcasts the 24/7 Siren Frequency Theater Radio and oversees local area mesh discovery.",
            is_primary: false,
        };

        Self {
            viewer_did: auth.user_did.clone(),
            current_user: auth,
            active_subtab: ClubViewSubTab::RegisteredClubs,
            registered_clubs: vec![primary_club, secondary_club],
            selected_club_id: "club_team7_hq".into(),
            home_nodes: vec!["team7-node-alpha".into(), "atlantis-pi-home-1".into(), "home-2".into()],
            members: vec![
                Member {
                    did: "did:atlantis:team7_operator".into(),
                    name: "Team 7 Operator (You)".into(),
                    officer_role: OfficerRole::TeamDirector,
                    status: Status::Active,
                    date_joined: "2026-10-01",
                },
                Member {
                    did: "did:atlantis:officer_alysa".into(),
                    name: "Alysa Vance".into(),
                    officer_role: OfficerRole::FirstViceDirector,
                    status: Status::Active,
                    date_joined: "2026-10-05",
                },
                Member {
                    did: "did:atlantis:officer_corin".into(),
                    name: "Corin Reed".into(),
                    officer_role: OfficerRole::SecondViceDirector,
                    status: Status::Active,
                    date_joined: "2026-10-08",
                },
                Member {
                    did: "did:atlantis:officer_maya".into(),
                    name: "Maya Lin".into(),
                    officer_role: OfficerRole::Secretary,
                    status: Status::Active,
                    date_joined: "2026-10-02",
                },
                Member {
                    did: "did:atlantis:officer_tariq".into(),
                    name: "Tariq Duval".into(),
                    officer_role: OfficerRole::Treasurer,
                    status: Status::Active,
                    date_joined: "2026-10-03",
                },
                Member {
                    did: "did:atlantis:officer_jules".into(),
                    name: "Jules Vance".into(),
                    officer_role: OfficerRole::ChiefMarshal,
                    status: Status::Active,
                    date_joined: "2026-10-06",
                },
                Member {
                    did: "did:atlantis:officer_kaito".into(),
                    name: "Kaito Tanaka".into(),
                    officer_role: OfficerRole::Sentinel,
                    status: Status::Active,
                    date_joined: "2026-10-07",
                },
            ],
            daemons: vec![
                Daemon {
                    name: "Triton (Tier 1 Abyssal Anchor & NCM Engine)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Harness² (Rust/Candle) • Verifier: Sonar Relay & 4-Scale NCM Consensus".into()),
                    ping_ms: 6,
                },
                Daemon {
                    name: "Ursula (Pelagic Orchestrator & MCP Gateway)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Goose MCP Harness (Rust) • Improver: Prompt Optimization & Deadlock Resolution".into()),
                    ping_ms: 7,
                },
                Daemon {
                    name: "Sebastian (Tier 3 Epipelagic Edge Sentinel)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("EdgeCrab (Rust) • Monitor: <=250ms PoH Canary Attestation & Hardware Vitals".into()),
                    ping_ms: 4,
                },
                Daemon {
                    name: "SevenDaughtersRadio (24/7 Siren Auto-Host)".into(),
                    home_node: "atlantis-pi-home-1".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Radio Broadcaster & Siren Frequency Audio Relay".into()),
                    ping_ms: 12,
                },
                Daemon {
                    name: "Team 7 Tactical Scout (Nautilus Drone)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Planetary ECEF Geodesic Waypoint Surveyor & Escrow Sentry".into()),
                    ping_ms: 8,
                },
                Daemon {
                    name: "Team 7 Coral Escrow Settlement Watcher".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Autonomous CYAN Micro-Tip & Double-Entry Tide Ledger Reconciler".into()),
                    ping_ms: 15,
                },
            ],
            building_office: office,
            pi_node_lan_host: "10.0.0.237:8080".into(),
            pi_atlantis_running: true,
            team_filter_only: true,
            new_daemon_name: String::new(),
            new_daemon_node: "team7-node-alpha".into(),
        }
    }
}

impl ClubState {
    /// True only for an active officer of the club.
    pub fn viewer_is_officer(&self) -> bool {
        self.members.iter().any(|m| {
            m.did == self.viewer_did && m.status == Status::Active && m.officer_role.is_officer()
        })
    }

    /// Synchronizes the 24/7 radio state with the Raspberry Pi status.
    pub fn sync_pi_radio_daemon(&mut self) {
        for d in &mut self.daemons {
            if d.name.starts_with("SevenDaughtersRadio") && d.home_node == "atlantis-pi-home-1" {
                d.running = self.pi_atlantis_running;
            }
        }
    }

    /// Returns true if the Club's 7 Daughters Radio daemon is currently running on the Atlantis Pi node.
    pub fn is_siren_radio_running(&self) -> bool {
        self.pi_atlantis_running
            && self
                .daemons
                .iter()
                .any(|d| d.name.starts_with("SevenDaughtersRadio") && d.running)
    }

    /// Master unified entrypoint rendering the Registered Clubs & Enclaves Page.
    pub fn render_clubs_page(&mut self, ui: &mut egui::Ui) {
        self.sync_pi_radio_daemon();

        ui.horizontal(|ui| {
            ui.heading("👥 Clubs & Team 7 Enclaves");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Nautilus Authoritative Digital Twin").color(egui::Color32::from_rgb(0, 229, 255)).small());
            });
        });
        ui.label("Decentralized registered clubs, secular lodge governance (7 Officers), and autonomous AI daemons.");
        ui.add_space(8.0);

        // Internal sub-tab selector
        ui.horizontal(|ui| {
            let is_clubs = self.active_subtab == ClubViewSubTab::RegisteredClubs;
            if ui.selectable_label(is_clubs, "🏛️ Registered Clubs").clicked() {
                self.active_subtab = ClubViewSubTab::RegisteredClubs;
            }

            let is_bylaws = self.active_subtab == ClubViewSubTab::Team7OfficeAndBylaws;
            if ui.selectable_label(is_bylaws, "🏢 Office & 7 Officers By-Laws").clicked() {
                self.active_subtab = ClubViewSubTab::Team7OfficeAndBylaws;
            }

            let is_daemons = self.active_subtab == ClubViewSubTab::RegisteredDaemons;
            if ui.selectable_label(is_daemons, "🤖 Registered Daemons").clicked() {
                self.active_subtab = ClubViewSubTab::RegisteredDaemons;
            }
        });
        ui.separator();
        ui.add_space(8.0);

        match self.active_subtab {
            ClubViewSubTab::RegisteredClubs => self.render_registered_clubs_subtab(ui),
            ClubViewSubTab::Team7OfficeAndBylaws => self.render_office_and_bylaws_subtab(ui),
            ClubViewSubTab::RegisteredDaemons => self.render_registered_daemons_tab(ui),
        }
    }

    /// Subtab 1: Registered Clubs the User Belongs To
    fn render_registered_clubs_subtab(&mut self, ui: &mut egui::Ui) {
        ui.strong(format!("My Registered Clubs ({})", self.registered_clubs.len()));
        ui.label("Clubs and node enclaves accepted onto the Kuroshio mesh where your DID holds recognized membership.");
        ui.add_space(8.0);

        for club in &self.registered_clubs {
            let is_selected = self.selected_club_id == club.club_id;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let badge = if club.is_primary { "⭐ PRIMARY" } else { "🔗 AFFILIATE" };
                    ui.label(egui::RichText::new(&club.name).strong().size(15.0));
                    ui.separator();
                    ui.label(egui::RichText::new(badge).color(egui::Color32::from_rgb(255, 215, 0)).small());
                    ui.separator();
                    ui.label(egui::RichText::new(format!("Role: {}", club.user_role.short_label())).color(egui::Color32::from_rgb(0, 229, 255)));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(if is_selected { "Active Focus" } else { "Select Club" }).clicked() {
                            // focus club
                        }
                    });
                });

                ui.add_space(4.0);
                ui.label(egui::RichText::new(club.charter_bylaws_summary).small().color(egui::Color32::LIGHT_GRAY));

                ui.add_space(4.0);
                egui::Grid::new(format!("club_grid_{}", club.club_id)).striped(true).show(ui, |ui| {
                    ui.label("Callsign / Home Node:");
                    ui.label(format!("{} • Node: {}", club.callsign, club.home_node_id));
                    ui.end_row();

                    ui.label("Physical Locus (Nautilus):");
                    ui.label(format!("{:.4}° N, {:.4}° E • {}", club.office_geo.lat_deg, club.office_geo.lon_deg, club.suite_label));
                    ui.end_row();

                    ui.label("Enclave Census:");
                    ui.label(format!("{} Registered Officers/Members • {} Live Daemons", club.member_count, club.active_daemons));
                    ui.end_row();
                });
            });
            ui.add_space(6.0);
        }

        ui.add_space(8.0);
        ui.group(|ui| {
            ui.strong("🍓 Local Network Infrastructure");
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.pi_atlantis_running,
                    "Atlantis Daemon Running on Raspberry Pi Node (10.0.0.237:8080)",
                );
                ui.separator();
                ui.label("LAN Host:");
                ui.text_edit_singleline(&mut self.pi_node_lan_host);
            });
            self.sync_pi_radio_daemon();
        });
    }

    /// Subtab 2: Team 7 Building Office & Secular 7 Officers By-Laws
    fn render_office_and_bylaws_subtab(&mut self, ui: &mut egui::Ui) {
        // Authoritative Nautilus Building Office Card
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🏢 Headquarters Building Office:").strong().size(15.0));
                ui.label(egui::RichText::new("SUITE 700 • ACTIVE ENCLAVE").color(egui::Color32::from_rgb(0, 229, 255)).strong());
            });
            let geo = self.building_office.anchor_geo;
            let ecef = self.building_office.anchor_ecef;
            ui.label(format!("Nautilus Geospatial Anchor: {:.6}° N, {:.6}° E, {:.1}m MSL (Floor 7)", geo.lat_deg, geo.lon_deg, geo.alt_meters));
            ui.label(format!("64-bit Planetary ECEF: ({:.0}, {:.0}, {:.0}) m • Unmutated Lemuria Core World Model", ecef.x, ecef.y, ecef.z));
            ui.label(egui::RichText::new("Acoustic Profile: Executive Suite (Sabine RT60 0.45s) • Direct Fiber to Pi Node (10.0.0.237)").small().color(egui::Color32::LIGHT_GRAY));
            
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Herdr Multiplexer:").strong());
                ui.label(egui::RichText::new("127.0.0.1:9877 (RUNNING)").color(egui::Color32::GREEN));
                ui.separator();
                ui.label(egui::RichText::new("Trio Supervised Loop:").strong());
                ui.label(egui::RichText::new("🔱 Triton [Verify] ➔ 🐙 Ursula [Improve] ➔ 🦀 Sebastian [Monitor]").color(egui::Color32::from_rgb(255, 215, 0)));
            });
        });

        ui.add_space(12.0);
        ui.heading("📜 The Seven Primary Officers & Secular By-Laws");
        ui.label("Operational structure translated from constitutional lodge governance without occult nomenclature.");
        ui.add_space(6.0);

        // 7 Officers Roster Matrix
        for officer in OfficerRole::SEVEN_OFFICERS {
            let assigned_member = self.members.iter().find(|m| m.officer_role == officer);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(officer.title()).strong().color(egui::Color32::from_rgb(0, 229, 255)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(m) = assigned_member {
                            ui.label(egui::RichText::new(format!("Incumbent: {} ({})", m.name, m.status.label())).color(egui::Color32::GREEN));
                        } else {
                            ui.label(egui::RichText::new("VACANT CHAIR").color(egui::Color32::YELLOW));
                        }
                    });
                });
                ui.label(officer.duty_description());
            });
            ui.add_space(2.0);
        }

        ui.add_space(10.0);
        ui.heading("📋 Standing By-Laws of the Enclave");
        ui.group(|ui| {
            ui.strong("Article I — Supreme Geodetic Origin");
            ui.label("Every room, terminal, and device in this office is anchored to authoritative 64-bit ECEF coordinates via Nautilus. Local drift without planetary anchors is prohibited.");
            ui.add_space(4.0);
            ui.strong("Article II — Selective Disclosure & Privacy Boundary");
            ui.label("High-precision telemetry and private sensor feeds remain sealed in the Club Home Node vault. Only coarse sector geohashes or ZK-Geo proofs may pass to the public mesh.");
            ui.add_space(4.0);
            ui.strong("Article III — Sevenfold Officer Consensus");
            ui.label("Official disbursements exceeding 10.0 CYAN, member expulsions, and warrant amendments require quorum of at least four of the Seven Officers, certified by the Secretary.");
            ui.add_space(4.0);
            ui.strong("Article IV — Proof-of-Health Canary Vigilance");
            ui.label("The Sentinel officer and Sebastian edge sentinel shall enforce <=250ms canary challenges. Nodes failing two successive cycles enter autonomous safety cooldown.");
        });
    }

    /// Subtab 3: Dedicated Daemons Registered to Me View
    pub fn render_registered_daemons_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏢 Team 7 Building Office & Cornerstone Daemons");
        ui.label("Private enclave office situated in your headquarters building. Supervised trio running on Herdr (127.0.0.1:9877).");
        ui.add_space(8.0);

        // Building Office Enclave Card
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🏢 Headquarters Building Office:").strong().size(15.0));
                ui.label(egui::RichText::new("SUITE 700 • ACTIVE ENCLAVE").color(egui::Color32::from_rgb(0, 229, 255)).strong());
            });
            let geo = self.building_office.anchor_geo;
            ui.label(format!("Physical Location: {:.4}° N, {:.4}° E • Roppongi Headquarters Tower • Floor 7", geo.lat_deg, geo.lon_deg));
            ui.label(egui::RichText::new("Acoustic Profile: Executive Office (Sabine RT60 0.45s) • Direct Fiber to Pi Node (10.0.0.237)").small().color(egui::Color32::LIGHT_GRAY));
            
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Herdr Multiplexer:").strong());
                ui.label(egui::RichText::new("127.0.0.1:9877 (OK)").color(egui::Color32::GREEN));
                ui.separator();
                ui.label(egui::RichText::new("Trio Loop:").strong());
                ui.label(egui::RichText::new("🔱 Triton [Verify] ➔ 🐙 Ursula [Improve] ➔ 🦀 Sebastian [Monitor]").color(egui::Color32::from_rgb(255, 215, 0)));
            });
        });

        ui.add_space(8.0);

        // Account & Auth Card
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔐 Authentication Status:").strong());
                if self.current_user.is_authenticated {
                    ui.label(egui::RichText::new("ONLINE • AUTHENTICATED").color(egui::Color32::GREEN).strong());
                } else {
                    ui.label(egui::RichText::new("OFFLINE • GUEST").color(egui::Color32::YELLOW).strong());
                }
            });
            egui::Grid::new("auth_info_grid").striped(true).show(ui, |ui| {
                ui.label("Google Identity / Email:");
                ui.label(egui::RichText::new(&self.current_user.email).color(egui::Color32::from_rgb(0, 229, 255)).strong());
                ui.end_row();

                ui.label("Display Name:");
                ui.label(&self.current_user.display_name);
                ui.end_row();

                ui.label("Registered Team / Guild:");
                ui.label(egui::RichText::new(format!("{} (Team 7)", self.current_user.team_id)).color(egui::Color32::from_rgb(255, 215, 0)).strong());
                ui.end_row();

                ui.label("Decentralized DID:");
                ui.label(&self.current_user.user_did);
                ui.end_row();

                ui.label("Identity Provider:");
                ui.label(&self.current_user.provider);
                ui.end_row();
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(if self.current_user.is_authenticated { "Sign Out" } else { "Sign In via Google" }).clicked() {
                    self.current_user.is_authenticated = !self.current_user.is_authenticated;
                }
                ui.separator();
                ui.checkbox(&mut self.team_filter_only, "Filter: Only Show Daemons Registered to My Team (Team 7)");
            });
        });

        ui.add_space(10.0);

        // Register New AI Daemon
        ui.group(|ui| {
            ui.strong("➕ Register New AI Daemon to Team 7");
            ui.horizontal(|ui| {
                ui.label("Daemon Name:");
                ui.text_edit_singleline(&mut self.new_daemon_name);
                ui.label("Host Node:");
                ui.text_edit_singleline(&mut self.new_daemon_node);

                if ui.button("Register Daemon").clicked() {
                    if !self.new_daemon_name.trim().is_empty() {
                        self.daemons.push(Daemon {
                            name: self.new_daemon_name.trim().to_string(),
                            home_node: if self.new_daemon_node.trim().is_empty() {
                                "team7-node-alpha".into()
                            } else {
                                self.new_daemon_node.trim().to_string()
                            },
                            running: true,
                            team_id: Some("team-7".into()),
                            registered_to: Some(self.current_user.user_did.clone()),
                            role_description: Some("Custom Team 7 Autonomous Worker".into()),
                            ping_ms: 10,
                        });
                        self.new_daemon_name.clear();
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Daemon Roster List
        let is_auth = self.current_user.is_authenticated;
        let my_did = self.current_user.user_did.clone();
        let team_id = self.current_user.team_id.clone();

        let filtered_indices: Vec<usize> = self.daemons
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                if !self.team_filter_only {
                    return true;
                }
                if !is_auth {
                    return false;
                }
                d.team_id.as_deref() == Some(team_id.as_str())
                    || d.registered_to.as_deref() == Some(my_did.as_str())
            })
            .map(|(i, _)| i)
            .collect();

        ui.heading(format!("Active Registered Daemons ({})", filtered_indices.len()));

        if !is_auth {
            ui.colored_label(egui::Color32::YELLOW, "⚠ You are signed out. Please sign in via Google to view daemons bound to Team 7.");
            return;
        }

        if filtered_indices.is_empty() {
            ui.label("No daemons currently registered to Team 7.");
            return;
        }

        let mut remove_idx: Option<usize> = None;

        for &i in &filtered_indices {
            let daemon = &mut self.daemons[i];
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let state_icon = if daemon.running { "🟢" } else { "⚪" };
                    ui.label(egui::RichText::new(format!("{state_icon} {}", daemon.name)).strong().size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Deregister").clicked() {
                            remove_idx = Some(i);
                        }
                        let is_running = daemon.running;
                        ui.checkbox(&mut daemon.running, if is_running { "RUNNING" } else { "STOPPED" });
                    });
                });

                if let Some(desc) = &daemon.role_description {
                    ui.label(egui::RichText::new(desc).small().color(egui::Color32::LIGHT_GRAY));
                }

                ui.horizontal(|ui| {
                    ui.label(format!("🏠 Node: {}", daemon.home_node));
                    ui.separator();
                    ui.label(format!("👥 Team: {}", daemon.team_id.as_deref().unwrap_or("none")));
                    ui.separator();
                    ui.label(format!("🔑 Registered DID: {}", daemon.registered_to.as_deref().unwrap_or("unassigned")));
                    ui.separator();
                    ui.label(egui::RichText::new(format!("📶 {} ms", daemon.ping_ms)).color(egui::Color32::GREEN).small());
                });
            });
            ui.add_space(4.0);
        }

        if let Some(idx) = remove_idx {
            self.daemons.remove(idx);
        }
    }

    /// Legacy fallback ui method delegating to render_clubs_page
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.render_clubs_page(ui);
    }
}
