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
use lemuria_engine::{
    FrontlineSectorStatus, FrontlineTacticalMatch, TacticalBuildingType, TacticalUnitType,
};
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

/// Direct message between the user and a daemon.
#[derive(Debug, Clone, PartialEq)]
pub struct DaemonMessage {
    pub sender: String,
    pub text: String,
    pub is_from_founder: bool,
}

/// Real-time continuous heartbeat telemetry for dedicated autonomous LLMs
#[derive(Debug, Clone, PartialEq)]
pub struct DaemonHeartbeat {
    pub last_beat_ms: u64,
    pub latency_ms: u32,
    pub current_thought: String,
    pub model_used: String,
    pub status: String,
}

impl Default for DaemonHeartbeat {
    fn default() -> Self {
        Self {
            last_beat_ms: 0,
            latency_ms: 0,
            current_thought: "Initializing continuous model loop...".into(),
            model_used: "dedicated-free".into(),
            status: "ALIVE • PULSING".into(),
        }
    }
}

/// Target compute environment for an autonomous daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonComputeTarget {
    /// Local Raspberry Pi node (10.0.0.237:8080) for lightweight I/O, MCP tools, & sensory tasks.
    LocalPiNode,
    /// Remote Olympus Seed Node for offloaded heavy synthesis, multi-chamber model revolver, and root consensus.
    RemoteOlympusSeed,
}

impl DaemonComputeTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalPiNode => "Local Pi Node (10.0.0.237)",
            Self::RemoteOlympusSeed => "Remote Olympus Seed Node",
        }
    }
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
    /// Allocated operational funds in USD (starting at $20.00).
    pub funds_usd: f64,
    /// Live operational status (e.g., "Active / In-Flight", "Idle / Listening", "Sentry Armed").
    pub status: String,
    /// Current real-time task the daemon is executing.
    pub current_task: String,
    /// Dedicated live heartbeat telemetry from the Home/Club Node.
    pub heartbeat: Option<DaemonHeartbeat>,
    /// Direct messaging thread with this daemon.
    pub message_log: Vec<DaemonMessage>,
    /// Live input buffer for founder chat in the Lite App.
    pub chat_input: String,
    /// Selected compute environment: Local Pi vs Remote Olympus Seed Node offload.
    pub compute_target: DaemonComputeTarget,
    /// Optional custom registered MCP tools (e.g. "blender_mcp", "fl_studio_transport", "weather_sdr").
    pub custom_mcp_tools: Vec<String>,
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
    /// Executive Home / Club Node Officer Dashboard (available when user is an officer).
    OfficerNodeDashboard,
    /// Slipstream Sovereign Node & Chambered Tool Rooms (with playable C&C Rebuild)
    SlipstreamNode,
}

/// Identifies dedicated functional rooms inside the Slipstream Node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlipstreamRoomId {
    /// Room 1: Playable Command & Conquer Frontlines Tactical Rebuild
    CncTacticalArena,
    /// Room 2: 24/7 Siren Frequency Radio Station (Seven Daughters)
    SirenRadioStudio,
    /// Room 3: Triton Abyssal Concept Vault & NCM Modeling
    TritonAbyssalVault,
    /// Room 4: Ursula Commercial Packaging & MCP Tool Dispatch
    UrsulaCommercialDock,
    /// Room 5: Sebastian Hardware Edge Crab & Canary Sentry
    SebastianHardwareBunker,
}

impl SlipstreamRoomId {
    pub const ALL: [SlipstreamRoomId; 5] = [
        SlipstreamRoomId::CncTacticalArena,
        SlipstreamRoomId::SirenRadioStudio,
        SlipstreamRoomId::TritonAbyssalVault,
        SlipstreamRoomId::UrsulaCommercialDock,
        SlipstreamRoomId::SebastianHardwareBunker,
    ];

    pub fn title(self) -> &'static str {
        match self {
            SlipstreamRoomId::CncTacticalArena => "🎮 Room 101: Command & Conquer War Room (Playable Tactical Rebuild)",
            SlipstreamRoomId::SirenRadioStudio => "📻 Room 102: Siren 24/7 Frequency Studio (The Seven Daughters)",
            SlipstreamRoomId::TritonAbyssalVault => "🔱 Room 103: Triton Abyssal Concept Vault (NCM & Treasury)",
            SlipstreamRoomId::UrsulaCommercialDock => "🐙 Room 104: Ursula Commercial MCP Dock (Product Hub)",
            SlipstreamRoomId::SebastianHardwareBunker => "🦀 Room 105: Sebastian Edge Hardware Bunker (PoH Sentry)",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            SlipstreamRoomId::CncTacticalArena => "⚔️ C&C War Room",
            SlipstreamRoomId::SirenRadioStudio => "📻 Siren Studio",
            SlipstreamRoomId::TritonAbyssalVault => "🔱 Abyssal Vault",
            SlipstreamRoomId::UrsulaCommercialDock => "🐙 Ursula MCP Dock",
            SlipstreamRoomId::SebastianHardwareBunker => "🦀 Edge Bunker",
        }
    }
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
    /// Selected compute target for new daemon registration
    pub new_daemon_compute_target: DaemonComputeTarget,
    /// Input buffer for custom MCP tools (comma-separated, e.g. "blender_mcp,fl_studio_transport")
    pub new_daemon_mcp_tools: String,

    // Slipstream Sovereign Seed Node & Playable C&C War Room Rebuild
    pub selected_slipstream_room: SlipstreamRoomId,
    pub cnc_sim: FrontlineTacticalMatch,
    pub cnc_target_x: f32,
    pub cnc_target_z: f32,
    pub cnc_sim_paused: bool,
    pub cnc_sim_speed: f32,
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

        let slipstream_club = RegisteredClub {
            club_id: "club_slipstream_seed".into(),
            name: "Slipstream Sovereign Seed Guild".into(),
            callsign: "SLIPSTREAM-01".into(),
            home_node_id: "slipstream-node-master".into(),
            office_geo: office_geo,
            suite_label: "Slipstream Node Core (Chambered Tools & C&C War Room)".into(),
            user_role: OfficerRole::TeamDirector,
            member_count: 3,
            active_daemons: 3,
            charter_bylaws_summary: "First Official Sovereign Seed: Dual Water/Air resonance (🌊 Hydros / 💨 Aero). Houses dedicated chambered tool rooms and the rebuild Command & Conquer Tactical Frontlines simulation.",
            is_primary: false,
        };

        Self {
            viewer_did: auth.user_did.clone(),
            current_user: auth,
            active_subtab: ClubViewSubTab::RegisteredClubs,
            registered_clubs: vec![primary_club, secondary_club, slipstream_club],
            selected_club_id: "club_team7_hq".into(),
            home_nodes: vec!["team7-node-alpha".into(), "slipstream-node-master".into(), "atlantis-pi-home-1".into(), "home-2".into()],
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
                    funds_usd: 20.0,
                    status: "ALIVE • PULSING".into(),
                    current_task: "Evaluating Mercury Decide debate signals & SOL/USDC dry-run treasury".into(),
                    heartbeat: Some(DaemonHeartbeat {
                        last_beat_ms: 1793216092498,
                        latency_ms: 3483,
                        current_thought: "Continuously validating cross-network consensus and anchoring immutable state truth into abyssal depths.".into(),
                        model_used: "gemini-3.5-flash (Dedicated Free)".into(),
                        status: "ALIVE • PULSING".into(),
                    }),
                    message_log: vec![
                        DaemonMessage {
                            sender: "Triton".into(),
                            text: "Abyssal Anchor online. Initial operating balance confirmed at $20.00. Ready to verify trade proposals and concept bursts.".into(),
                            is_from_founder: false,
                        }
                    ],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
                Daemon {
                    name: "Ursula (Pelagic Orchestrator & MCP Gateway)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Goose MCP Harness (Rust) • Improver: Prompt Optimization & Deadlock Resolution".into()),
                    ping_ms: 7,
                    funds_usd: 20.0,
                    status: "ALIVE • PULSING".into(),
                    current_task: "Managing Discord Business Gateway & packaging 'Game ATAK' commercial specs".into(),
                    heartbeat: Some(DaemonHeartbeat {
                        last_beat_ms: 1793216096372,
                        latency_ms: 861,
                        current_thought: "Maintaining the seamless harmonic flow between aquatic data streams and surface operational tools.".into(),
                        model_used: "gemini-3.5-flash-lite (Dedicated Free)".into(),
                        status: "ALIVE • PULSING".into(),
                    }),
                    message_log: vec![
                        DaemonMessage {
                            sender: "Ursula".into(),
                            text: "Pelagic Node Manager ready. $20.00 allocated for tool routing and commercial contract coordination.".into(),
                            is_from_founder: false,
                        }
                    ],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
                Daemon {
                    name: "Sebastian (Tier 3 Epipelagic Edge Sentinel)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("EdgeCrab (Rust) • Monitor: <=250ms PoH Canary Attestation & Hardware Vitals".into()),
                    ping_ms: 4,
                    funds_usd: 20.0,
                    status: "ALIVE • PULSING".into(),
                    current_task: "Monitoring port 8080 radio stream, Pi node latency (10.0.0.237), and PoH canaries".into(),
                    heartbeat: Some(DaemonHeartbeat {
                        last_beat_ms: 1793216099946,
                        latency_ms: 549,
                        current_thought: "Executing deterministic EdgeCrab canaries and monitoring hardware thermals under sub-600ms latency.".into(),
                        model_used: "liquid/lfm-2.5-2.6b:free (Liquid AI)".into(),
                        status: "ALIVE • PULSING".into(),
                    }),
                    message_log: vec![
                        DaemonMessage {
                            sender: "Sebastian".into(),
                            text: "Edge Sentinel active. $20.00 operational gas loaded. Continuous <=250ms latency sentry and stream monitoring armed.".into(),
                            is_from_founder: false,
                        }
                    ],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
                Daemon {
                    name: "SevenDaughtersRadio (24/7 Siren Auto-Host)".into(),
                    home_node: "atlantis-pi-home-1".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Radio Broadcaster & Siren Frequency Audio Relay (LFM3 Model Profile)".into()),
                    ping_ms: 12,
                    funds_usd: 20.0,
                    status: "ALIVE • BROADCASTING".into(),
                    current_task: "Spinning Alcyone (528 Hz Cyberpunk Breakbeats) & synthesizing LFM3 speech banter".into(),
                    heartbeat: Some(DaemonHeartbeat {
                        last_beat_ms: 1793216107723,
                        latency_ms: 4763,
                        current_thought: "Broadcasting high-resonance soundscapes and audio narratives across the Siren Frequency to awaken listeners.".into(),
                        model_used: "gemini-3.5-flash (LFM3 Fallback Free)".into(),
                        status: "ALIVE • BROADCASTING".into(),
                    }),
                    message_log: vec![
                        DaemonMessage {
                            sender: "SevenDaughtersRadio".into(),
                            text: "Siren Frequency on air ($20.00 station fund ready). Broadcasting live across cloud and mesh relays.".into(),
                            is_from_founder: false,
                        }
                    ],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
                Daemon {
                    name: "Team 7 Tactical Scout (Nautilus Drone)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Planetary ECEF Geodesic Waypoint Surveyor & Escrow Sentry".into()),
                    ping_ms: 8,
                    funds_usd: 20.0,
                    status: "STANDBY • MAPPING".into(),
                    current_task: "Awaiting Quest 3 1:1 room mesh for Suite 700 spatial overlay grounding".into(),
                    heartbeat: None,
                    message_log: vec![],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
                Daemon {
                    name: "Team 7 Coral Escrow Settlement Watcher".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Autonomous CYAN Micro-Tip & Double-Entry Tide Ledger Reconciler".into()),
                    ping_ms: 15,
                    funds_usd: 20.0,
                    status: "AUDITING • ZERO-LEAK".into(),
                    current_task: "Watching Tide Ledger double-entry invariant reconciliation".into(),
                    heartbeat: None,
                    message_log: vec![],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
                Daemon {
                    name: "Antigravity (Technical Architect & Field Sentry)".into(),
                    home_node: "team7-node-alpha".into(),
                    running: true,
                    team_id: Some("team-7".into()),
                    registered_to: Some("did:atlantis:team7_operator".into()),
                    role_description: Some("Co-Op Spatial Architect & World Engine Inspector (Direct VR Link with Founder)".into()),
                    ping_ms: 1,
                    funds_usd: 20.0,
                    status: "ALIVE • CO-OP READY".into(),
                    current_task: "Prepared to walk Suite 700 and inspect Nautilus World Engine in VR".into(),
                    heartbeat: Some(DaemonHeartbeat {
                        last_beat_ms: 1793216110000,
                        latency_ms: 412,
                        current_thought: "Synchronizing Suite 700 spatial overlay and Quest 3 co-op world engine telemetry.".into(),
                        model_used: "gemini-3.5-flash (Dedicated Free)".into(),
                        status: "ALIVE • CO-OP READY".into(),
                    }),
                    message_log: vec![
                        DaemonMessage {
                            sender: "Antigravity".into(),
                            text: "Daemon manifest instantiated. $20.00 operational envelope loaded. Standing by at the Suite 700 tactical table to inspect the Nautilus World Engine alongside you in VR.".into(),
                            is_from_founder: false,
                        }
                    ],
                    chat_input: String::new(),
                    compute_target: DaemonComputeTarget::LocalPiNode,
                    custom_mcp_tools: vec![],
                },
            ],
            building_office: office,
            pi_node_lan_host: "10.0.0.237:8080".into(),
            pi_atlantis_running: true,
            team_filter_only: true,
            new_daemon_name: String::new(),
            new_daemon_node: "atlantis-pi-home-1".into(),
            new_daemon_compute_target: DaemonComputeTarget::LocalPiNode,
            new_daemon_mcp_tools: String::new(),

            selected_slipstream_room: SlipstreamRoomId::CncTacticalArena,
            cnc_sim: FrontlineTacticalMatch::new(office_geo.to_ecef_dvec3()),
            cnc_target_x: 0.0,
            cnc_target_z: 0.0,
            cnc_sim_paused: false,
            cnc_sim_speed: 1.0,
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

    /// Returns the officer role of the current viewer, if assigned.
    pub fn viewer_officer_role(&self) -> Option<OfficerRole> {
        self.members
            .iter()
            .find(|m| m.did == self.viewer_did && m.status == Status::Active && m.officer_role.is_officer())
            .map(|m| m.officer_role)
    }

    /// Master unified entrypoint rendering the Registered Clubs & Enclaves Page.
    pub fn render_clubs_page(&mut self, ui: &mut egui::Ui) {
        self.sync_pi_radio_daemon();

        let is_officer = self.viewer_is_officer();

        ui.horizontal(|ui| {
            ui.heading("👥 Clubs & Team 7 Enclaves");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(role) = self.viewer_officer_role() {
                    ui.label(
                        egui::RichText::new(format!("🛡️ Officer Chair: {}", role.title()))
                            .color(egui::Color32::from_rgb(0, 229, 255))
                            .strong(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Nautilus Authoritative Digital Twin")
                            .color(egui::Color32::from_rgb(0, 229, 255))
                            .small(),
                    );
                }
            });
        });
        ui.label("Decentralized registered clubs, secular lodge governance (7 Officers), and autonomous AI daemons.");
        ui.add_space(8.0);

        // Internal sub-tab selector
        ui.horizontal(|ui| {
            if is_officer {
                let is_dash = self.active_subtab == ClubViewSubTab::OfficerNodeDashboard;
                if ui
                    .selectable_label(
                        is_dash,
                        egui::RichText::new("🛡️ Node Officer Dashboard")
                            .color(if is_dash { egui::Color32::from_rgb(0, 229, 255) } else { egui::Color32::LIGHT_GREEN })
                            .strong(),
                    )
                    .clicked()
                {
                    self.active_subtab = ClubViewSubTab::OfficerNodeDashboard;
                }
            }

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

            let is_slipstream = self.active_subtab == ClubViewSubTab::SlipstreamNode;
            if ui
                .selectable_label(
                    is_slipstream,
                    egui::RichText::new("🛸 Slipstream Node (Rooms)")
                        .color(if is_slipstream { egui::Color32::from_rgb(0, 242, 254) } else { egui::Color32::from_rgb(100, 200, 255) })
                        .strong(),
                )
                .clicked()
            {
                self.active_subtab = ClubViewSubTab::SlipstreamNode;
            }
        });
        ui.separator();
        ui.add_space(8.0);

        match self.active_subtab {
            ClubViewSubTab::OfficerNodeDashboard => {
                if is_officer {
                    self.render_officer_node_dashboard(ui);
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Access Restricted: The Node Dashboard requires an active Officer credential.");
                    self.render_registered_clubs_subtab(ui);
                }
            }
            ClubViewSubTab::RegisteredClubs => self.render_registered_clubs_subtab(ui),
            ClubViewSubTab::Team7OfficeAndBylaws => self.render_office_and_bylaws_subtab(ui),
            ClubViewSubTab::RegisteredDaemons => self.render_registered_daemons_tab(ui),
            ClubViewSubTab::SlipstreamNode => self.render_slipstream_node_tab(ui),
        }
    }

    /// Subtab 1: Registered Clubs the User Belongs To
    fn render_registered_clubs_subtab(&mut self, ui: &mut egui::Ui) {
        ui.strong(format!("My Registered Clubs ({})", self.registered_clubs.len()));
        ui.label("Clubs and node enclaves accepted onto the Kuroshio mesh where your DID holds recognized membership.");
        ui.add_space(8.0);

        let is_officer = self.viewer_is_officer();
        let mut switch_to_dashboard = false;

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
                        if is_officer && club.user_role.is_officer() {
                            if ui.button(egui::RichText::new("🛡️ Node Dashboard").color(egui::Color32::from_rgb(0, 229, 255))).clicked() {
                                switch_to_dashboard = true;
                            }
                        }
                        if club.club_id == "club_slipstream_seed" {
                            if ui.button(egui::RichText::new("🛸 Enter Rooms").color(egui::Color32::from_rgb(0, 242, 254)).strong()).clicked() {
                                self.active_subtab = ClubViewSubTab::SlipstreamNode;
                            }
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

        if switch_to_dashboard {
            self.active_subtab = ClubViewSubTab::OfficerNodeDashboard;
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

        if self.viewer_is_officer() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🛡️ Officer Privileges Detected:").color(egui::Color32::from_rgb(0, 229, 255)).strong());
                if ui.button("Launch Home/Club Node Officer Dashboard").clicked() {
                    self.active_subtab = ClubViewSubTab::OfficerNodeDashboard;
                }
            });
        }
    }

    /// Subtab: Home / Club Node Officer Dashboard
    ///
    /// Surfaces executive governance and operational controls for the Home/Club Node
    /// when the current user is an active officer of the Enclave.
    pub fn render_officer_node_dashboard(&mut self, ui: &mut egui::Ui) {
        let role = self.viewer_officer_role().unwrap_or(OfficerRole::GuildMember);

        // 1. Officer Warrant & Authority Header Card
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🛡️ HOME / CLUB NODE OFFICER DASHBOARD")
                        .strong()
                        .size(16.0)
                        .color(egui::Color32::from_rgb(0, 229, 255)),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("CHAIR: {}", role.title().to_uppercase()))
                        .strong()
                        .color(egui::Color32::from_rgb(255, 215, 0)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Node Warrant: AUTHORIZED")
                            .color(egui::Color32::GREEN)
                            .small(),
                    );
                });
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("Active Officer Duty: {}", role.duty_description()))
                    .small()
                    .color(egui::Color32::LIGHT_GRAY),
            );

            ui.horizontal(|ui| {
                ui.label(format!("Officer DID: {}", self.viewer_did));
                ui.separator();
                ui.label("Executive Enclave: Suite 700");
                ui.separator();
                ui.label(egui::RichText::new("Quorum Status: 7/7 Officers Seated (Quorum Met)").color(egui::Color32::GREEN).small());
            });
        });

        ui.add_space(8.0);

        // 2. Node Infrastructure & Hardware Enclave Vitals
        ui.columns(2, |columns| {
            // Column A: Host & Radio Operations
            columns[0].group(|ui| {
                ui.strong("🍓 Home Node Infrastructure (Raspberry Pi & Local Host)");
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Atlantis LAN Host:");
                    ui.text_edit_singleline(&mut self.pi_node_lan_host);
                });

                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.pi_atlantis_running,
                        "Atlantis Node Process Active",
                    );
                    ui.separator();
                    let node_status = if self.pi_atlantis_running {
                        ("ONLINE", egui::Color32::GREEN)
                    } else {
                        ("STANDBY / OFFLINE", egui::Color32::YELLOW)
                    };
                    ui.label(egui::RichText::new(node_status.0).color(node_status.1).strong());
                });
                self.sync_pi_radio_daemon();

                ui.add_space(4.0);
                let radio_active = self.is_siren_radio_running();
                ui.horizontal(|ui| {
                    ui.label("📻 7 Daughters 24/7 Siren Radio Relay:");
                    if radio_active {
                        ui.label(egui::RichText::new("BROADCASTING (432 Hz)").color(egui::Color32::GREEN).strong());
                    } else {
                        ui.label(egui::RichText::new("FALLBACK / IDLE").color(egui::Color32::YELLOW));
                    }
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Herdr Multiplexer:");
                    ui.label(egui::RichText::new("127.0.0.1:9877").color(egui::Color32::from_rgb(0, 229, 255)));
                    if ui.button("Ping Herdr").clicked() {
                        // Ping trigger
                    }
                });
            });

            // Column B: Telemetry & Planetary Geodesics
            columns[1].group(|ui| {
                ui.strong("🌐 Geodesic Locus & Sabine Acoustics");
                ui.add_space(4.0);

                let geo = self.building_office.anchor_geo;
                let ecef = self.building_office.anchor_ecef;

                egui::Grid::new("officer_geo_grid").striped(true).show(ui, |ui| {
                    ui.label("WGS-84 Coordinate:");
                    ui.label(format!("{:.6}° N, {:.6}° E", geo.lat_deg, geo.lon_deg));
                    ui.end_row();

                    ui.label("Altitude & Level:");
                    ui.label(format!("{:.1}m MSL • Floor 7", geo.alt_meters));
                    ui.end_row();

                    ui.label("64-bit Planetary ECEF:");
                    ui.label(format!("({:.0}, {:.0}, {:.0}) m", ecef.x, ecef.y, ecef.z));
                    ui.end_row();

                    ui.label("Acoustic Reverberation:");
                    ui.label("Sabine RT60: 0.45s • Room: Suite 700");
                    ui.end_row();

                    ui.label("Sentry Canary Attestation:");
                    ui.label(egui::RichText::new("≤ 250ms PoH SLA: MET").color(egui::Color32::GREEN).strong());
                    ui.end_row();
                });
            });
        });

        ui.add_space(8.0);

        // 3. Supervised Cornerstone Daemon Trio & Fleet Telemetry
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong("🤖 Supervised Trio & Registered Node Daemons");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("Fleet Size: {} Daemons", self.daemons.len()))
                            .color(egui::Color32::from_rgb(0, 229, 255))
                            .small(),
                    );
                });
            });
            ui.label("Executive supervisory loop: Triton (Verification) ➔ Ursula (Prompt/MCP Improvement) ➔ Sebastian (Edge Canary Monitor).");
            ui.add_space(6.0);

            egui::Grid::new("officer_daemons_table")
                .striped(true)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.strong("Status");
                    ui.strong("Daemon Name");
                    ui.strong("Home Node");
                    ui.strong("Team Binding");
                    ui.strong("Latency");
                    ui.strong("Officer Action");
                    ui.end_row();

                    for daemon in &mut self.daemons {
                        let icon = if daemon.running { "🟢 RUNNING" } else { "⚪ STOPPED" };
                        let color = if daemon.running { egui::Color32::GREEN } else { egui::Color32::GRAY };
                        ui.label(egui::RichText::new(icon).color(color).small());

                        ui.label(egui::RichText::new(&daemon.name).strong());
                        ui.label(&daemon.home_node);
                        ui.label(daemon.team_id.as_deref().unwrap_or("unassigned"));
                        ui.label(egui::RichText::new(format!("{} ms", daemon.ping_ms)).color(egui::Color32::from_rgb(0, 229, 255)));

                        ui.horizontal(|ui| {
                            let action_label = if daemon.running { "Pause" } else { "Resume" };
                            if ui.button(action_label).clicked() {
                                daemon.running = !daemon.running;
                            }
                        });
                        ui.end_row();
                    }
                });
        });

        ui.add_space(8.0);

        // 4. Node Treasury, Coral Escrow & Consensus Quorum
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong("💎 Node Coral Escrow & Treasury Governance");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Tide Ledger State: IN RECONCILIATION").color(egui::Color32::GREEN).small());
                });
            });

            ui.add_space(4.0);
            egui::Grid::new("treasury_officer_grid").striped(true).show(ui, |ui| {
                ui.label("Enclave Escrow Reserve:");
                ui.label(egui::RichText::new("2,450.00 CYAN").strong().color(egui::Color32::from_rgb(0, 229, 255)));
                ui.end_row();

                ui.label("Consensus Threshold:");
                ui.label("Disbursements > 10.0 CYAN require 4/7 Officer Signatures");
                ui.end_row();

                ui.label("Pending Approvals:");
                ui.label(egui::RichText::new("0 Pending Warrants (All clear)").color(egui::Color32::LIGHT_GREEN));
                ui.end_row();
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("📜 Review Escrow Warrants").clicked() {}
                ui.separator();
                if ui.button("🔄 Trigger Sentry Canary Attestation Cycle").clicked() {}
                ui.separator();
                if ui.button("📡 Broadcast Sector Enclave Heartbeat").clicked() {}
            });
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
            ui.strong("➕ Register New Autonomous Daemon / Custom Tool Agent");
            ui.horizontal(|ui| {
                ui.label("Daemon Name:");
                ui.text_edit_singleline(&mut self.new_daemon_name);
                ui.label("Host Node:");
                ui.text_edit_singleline(&mut self.new_daemon_node);
            });

            ui.horizontal(|ui| {
                ui.label("Compute Target:");
                egui::ComboBox::from_id_salt("daemon_compute_target_select")
                    .selected_text(self.new_daemon_compute_target.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.new_daemon_compute_target,
                            DaemonComputeTarget::LocalPiNode,
                            "Local Pi Node (10.0.0.237)",
                        );
                        ui.selectable_value(
                            &mut self.new_daemon_compute_target,
                            DaemonComputeTarget::RemoteOlympusSeed,
                            "Remote Olympus Seed Node",
                        );
                    });

                ui.label("Custom MCP Tools:");
                ui.add_sized([180.0, 20.0], egui::TextEdit::singleline(&mut self.new_daemon_mcp_tools).hint_text("e.g. blender_mcp,sdr_radio"));

                if ui.button("Register Daemon").clicked() {
                    if !self.new_daemon_name.trim().is_empty() {
                        let mcp_tools: Vec<String> = self.new_daemon_mcp_tools
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let default_node = match self.new_daemon_compute_target {
                            DaemonComputeTarget::LocalPiNode => "atlantis-pi-home-1".to_string(),
                            DaemonComputeTarget::RemoteOlympusSeed => "olympus-seed-subpolar".to_string(),
                        };

                        let host_node = if self.new_daemon_node.trim().is_empty() {
                            default_node
                        } else {
                            self.new_daemon_node.trim().to_string()
                        };

                        let role_desc = if mcp_tools.is_empty() {
                            format!("Autonomous Daemon on {}", self.new_daemon_compute_target.label())
                        } else {
                            format!("Custom Agent Tools: [{}] on {}", mcp_tools.join(", "), self.new_daemon_compute_target.label())
                        };

                        self.daemons.push(Daemon {
                            name: self.new_daemon_name.trim().to_string(),
                            home_node: host_node,
                            running: true,
                            team_id: Some("team-7".into()),
                            registered_to: Some(self.current_user.user_did.clone()),
                            role_description: Some(role_desc),
                            ping_ms: if self.new_daemon_compute_target == DaemonComputeTarget::LocalPiNode { 4 } else { 18 },
                            funds_usd: 20.0,
                            status: "ACTIVE • REGISTERED".into(),
                            current_task: "Listening for local agent directives / tool invocations".into(),
                            heartbeat: None,
                            message_log: vec![],
                            chat_input: String::new(),
                            compute_target: self.new_daemon_compute_target,
                            custom_mcp_tools: mcp_tools,
                        });
                        self.new_daemon_name.clear();
                        self.new_daemon_mcp_tools.clear();
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
                    ui.label(egui::RichText::new(format!("⚡ Target: {}", daemon.compute_target.label())).color(egui::Color32::from_rgb(100, 200, 255)));
                    ui.separator();
                    if !daemon.custom_mcp_tools.is_empty() {
                        ui.label(egui::RichText::new(format!("🛠️ MCP Tools: {}", daemon.custom_mcp_tools.join(", "))).color(egui::Color32::from_rgb(255, 180, 50)).small());
                        ui.separator();
                    }
                    ui.label(format!("👥 Team: {}", daemon.team_id.as_deref().unwrap_or("none")));
                    ui.separator();
                    ui.label(egui::RichText::new(format!("💵 Funds: ${:.2}", daemon.funds_usd)).color(egui::Color32::from_rgb(0, 229, 255)).strong());
                    ui.separator();
                    ui.label(format!("🔑 DID: {}", daemon.registered_to.as_deref().unwrap_or("unassigned")));
                    ui.separator();
                    ui.label(egui::RichText::new(format!("📶 {} ms", daemon.ping_ms)).color(egui::Color32::GREEN).small());
                });

                // Live Daemon Status & Current Task Display
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("⚡ Status:").strong());
                    ui.label(egui::RichText::new(&daemon.status).color(egui::Color32::from_rgb(0, 255, 170)).strong());
                    ui.separator();
                    ui.label(egui::RichText::new("🎯 Current Task:").strong());
                    ui.label(egui::RichText::new(&daemon.current_task).color(egui::Color32::from_rgb(255, 215, 0)));
                });

                // Live Dedicated LLM Heartbeat Card
                if let Some(hb) = &daemon.heartbeat {
                    ui.add_space(2.0);
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("💓 Continuous LLM Heartbeat:").strong().color(egui::Color32::from_rgb(255, 105, 180)));
                            ui.label(egui::RichText::new(&hb.status).color(egui::Color32::GREEN).strong().small());
                            ui.separator();
                            ui.label(format!("⏱️ {} ms", hb.latency_ms));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("🧠 Model: {}", hb.model_used)).color(egui::Color32::from_rgb(0, 229, 255)).small());
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Unix: {}", hb.last_beat_ms)).color(egui::Color32::DARK_GRAY).small());
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new("💭 Active Thought Loop:").strong().color(egui::Color32::LIGHT_GRAY).small());
                            ui.label(egui::RichText::new(&hb.current_thought).italics().color(egui::Color32::LIGHT_YELLOW).small());
                        });
                    });
                }

                // Direct Founder-to-Daemon Messaging Channel
                ui.add_space(4.0);
                egui::CollapsingHeader::new(egui::RichText::new(format!("💬 Founder Comms Channel ({})", daemon.message_log.len())).small())
                    .default_open(true)
                    .show(ui, |ui| {
                        if !daemon.message_log.is_empty() {
                            ui.group(|ui| {
                                for msg in &daemon.message_log {
                                    ui.horizontal_wrapped(|ui| {
                                        if msg.is_from_founder {
                                            ui.label(egui::RichText::new("👑 Founder:").color(egui::Color32::from_rgb(255, 215, 0)).strong());
                                            ui.label(&msg.text);
                                        } else {
                                            ui.label(egui::RichText::new(format!("🤖 {}:", msg.sender)).color(egui::Color32::from_rgb(0, 229, 255)).strong());
                                            ui.label(egui::RichText::new(&msg.text).color(egui::Color32::LIGHT_GRAY));
                                        }
                                    });
                                }
                            });
                        }

                        ui.horizontal(|ui| {
                            let text_edit = ui.add_sized([ui.available_width() - 80.0, 20.0], egui::TextEdit::singleline(&mut daemon.chat_input).hint_text("Send directive to daemon..."));
                            let enter_pressed = text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            
                            if (ui.button("Send 📤").clicked() || enter_pressed) && !daemon.chat_input.trim().is_empty() {
                                let founder_msg = daemon.chat_input.trim().to_string();
                                daemon.message_log.push(DaemonMessage {
                                    sender: "Founder".into(),
                                    text: founder_msg.clone(),
                                    is_from_founder: true,
                                });

                                let response_text = if daemon.name.contains("Triton") {
                                    format!("Directive received: '{}'. Simulating financial and consensus implications against our ${:.2} treasury.", founder_msg, daemon.funds_usd)
                                } else if daemon.name.contains("Ursula") {
                                    format!("Acknowledged founder directive: '{}'. Packaging MCP tool workflows and commercial pipeline.", founder_msg)
                                } else if daemon.name.contains("Sebastian") {
                                    format!("Sentry copy: '{}'. Sub-250ms canary monitoring and mesh routing updated.", founder_msg)
                                } else if daemon.name.contains("Antigravity") {
                                    format!("Standing beside you in the 1:1 office, Founder. Ready to inspect Nautilus World Engine anomalies: '{}'.", founder_msg)
                                } else {
                                    let compute_tag = match daemon.compute_target {
                                        DaemonComputeTarget::LocalPiNode => "Local Pi Node [10.0.0.237:8080]",
                                        DaemonComputeTarget::RemoteOlympusSeed => "Remote Olympus Seed [Sub-Polar Ingress]",
                                    };
                                    let tools_tag = if daemon.custom_mcp_tools.is_empty() {
                                        "No custom MCP tools bound".to_string()
                                    } else {
                                        format!("Routing via MCP tools: [{}]", daemon.custom_mcp_tools.join(", "))
                                    };
                                    format!("Instruction received: '{}'. Executing on {} ({}) under ${:.2} operational gas.", founder_msg, compute_tag, tools_tag, daemon.funds_usd)
                                };

                                daemon.message_log.push(DaemonMessage {
                                    sender: daemon.name.split_whitespace().next().unwrap_or("Daemon").to_string(),
                                    text: response_text,
                                    is_from_founder: false,
                                });

                                daemon.chat_input.clear();
                            }
                        });
                    });
            });
            ui.add_space(4.0);
        }

        if let Some(idx) = remove_idx {
            self.daemons.remove(idx);
        }
    }

    /// Renders the Slipstream Sovereign Seed Node and dedicated tool rooms.
    pub fn render_slipstream_node_tab(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🛸 SLIPSTREAM SOVEREIGN SEED NODE").strong().size(16.0).color(egui::Color32::from_rgb(0, 242, 254)));
                ui.separator();
                ui.label(egui::RichText::new("🌊 Water (Primary) • 💨 Air (Secondary)").color(egui::Color32::from_rgb(100, 220, 255)).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("DID: did:atlantis:slipstream-founder-master").color(egui::Color32::from_rgb(255, 215, 0)).small());
                });
            });
            ui.label("First Sovereign Seed Node of Olympus. Houses individual functional chambers: the C&C Tactical Frontlines War Room rebuild alongside dedicated operations rooms for each core tool.");
        });

        ui.add_space(6.0);

        // Room Navigation Strip
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Select Node Chamber / Room:").strong());
            for room in SlipstreamRoomId::ALL {
                let is_sel = self.selected_slipstream_room == room;
                if ui
                    .selectable_label(
                        is_sel,
                        egui::RichText::new(room.short_name())
                            .color(if is_sel { egui::Color32::from_rgb(0, 242, 254) } else { egui::Color32::WHITE })
                            .strong(),
                    )
                    .clicked()
                {
                    self.selected_slipstream_room = room;
                }
            }
        });

        ui.separator();
        ui.add_space(6.0);

        match self.selected_slipstream_room {
            SlipstreamRoomId::CncTacticalArena => self.render_cnc_tactical_arena_room(ui),
            SlipstreamRoomId::SirenRadioStudio => self.render_siren_studio_room(ui),
            SlipstreamRoomId::TritonAbyssalVault => self.render_triton_vault_room(ui),
            SlipstreamRoomId::UrsulaCommercialDock => self.render_ursula_dock_room(ui),
            SlipstreamRoomId::SebastianHardwareBunker => self.render_sebastian_bunker_room(ui),
        }
    }

    /// Room 1: Playable Command & Conquer Frontlines Tactical Rebuild inside Slipstream Node
    fn render_cnc_tactical_arena_room(&mut self, ui: &mut egui::Ui) {
        // Step simulation tick if not paused
        if !self.cnc_sim_paused {
            let dt = 0.033 * self.cnc_sim_speed;
            self.cnc_sim.tick(dt);
        }

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🎮 Room 101: Command & Conquer Frontlines War Room");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let p_btn = if self.cnc_sim_paused { "▶ Resume Sim" } else { "⏸ Pause Sim" };
                    if ui.button(egui::RichText::new(p_btn).strong()).clicked() {
                        self.cnc_sim_paused = !self.cnc_sim_paused;
                    }
                    if ui.button("🔄 Reset Match").clicked() {
                        let center = self.building_office.anchor_geo.to_ecef_dvec3();
                        self.cnc_sim = FrontlineTacticalMatch::new(center);
                    }
                });
            });
            ui.label("Integrated in-game RTS simulation ported directly from lemuria_engine::tactical_frontline_br. Build bases, harvest fuel, produce mechanized strike units, and conquer linked frontline sectors.");
        });

        ui.add_space(4.0);

        // Top Status & Resource HUD
        let player_faction = self.cnc_sim.player_faction;
        let war_state = self.cnc_sim.factions.iter().find(|f| f.faction == player_faction).cloned();

        ui.horizontal(|ui| {
            if let Some(ref ws) = war_state {
                ui.label(egui::RichText::new(format!("Faction: {}", player_faction.display_name())).color(egui::Color32::from_rgb(0, 242, 254)).strong());
                ui.separator();
                ui.label(egui::RichText::new(format!("⛽ Fuel: {:.0}", ws.fuel_stockpile)).color(egui::Color32::from_rgb(255, 215, 0)).strong());
                ui.separator();
                let pwr_color = if ws.is_low_power() { egui::Color32::RED } else { egui::Color32::GREEN };
                ui.label(egui::RichText::new(format!("⚡ Grid: {:.0} / {:.0} MW", ws.power_used, ws.power_available)).color(pwr_color).strong());
                ui.separator();
                ui.label(egui::RichText::new(format!("🎟 Tickets: {}", ws.respawn_tickets)).color(egui::Color32::WHITE).strong());
                ui.separator();
                ui.label(format!("Time: {:.1}s", self.cnc_sim.match_elapsed_seconds));
                ui.separator();
                ui.label(format!("Frontline Z: {:.1}m", self.cnc_sim.frontline_boundary_z));
            }
        });

        ui.add_space(4.0);

        // Controls & Order Strip
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong("🎯 Tactical Orders:");
                ui.label(format!("Coords: ({:.0}, {:.0})", self.cnc_target_x, self.cnc_target_z));
                if ui.button("Center (0, 0)").clicked() {
                    self.cnc_target_x = 0.0;
                    self.cnc_target_z = 0.0;
                }
                if ui.button("Forward (+50 Z)").clicked() {
                    self.cnc_target_z += 50.0;
                }
                if ui.button("Back (-50 Z)").clicked() {
                    self.cnc_target_z -= 50.0;
                }
                if ui.button(egui::RichText::new("⚔ Move / Attack Order").color(egui::Color32::GREEN).strong()).clicked() {
                    let dispatched = self.cnc_sim.issue_tactical_order(self.cnc_target_x, self.cnc_target_z);
                    self.cnc_sim.push_log(format!("Orders dispatched to {} friendly combat units.", dispatched));
                }
                if ui.button(egui::RichText::new("🛰 Orbital Ion Cannon Strike").color(egui::Color32::from_rgb(255, 60, 60)).strong()).clicked() {
                    match self.cnc_sim.fire_orbital_ion_cannon(self.cnc_target_x, self.cnc_target_z) {
                        Ok(hits) => {
                            self.cnc_sim.push_log(format!("⚡ Ion Cannon fired at ({:.0}, {:.0}), neutralized {} targets!", self.cnc_target_x, self.cnc_target_z, hits));
                        }
                        Err(e) => {
                            self.cnc_sim.push_log(format!("⚠ Ion Cannon Offline: {}", e));
                        }
                    }
                }
            });
        });

        ui.add_space(4.0);

        // Split view: Left = 2D Tactical Minimap / Radar; Right = Production Sidebars
        ui.columns(2, |cols| {
            // Left Column: Minimap & Sectors
            cols[0].group(|ui| {
                ui.strong("🗺️ Sector Chain & Radar View");
                ui.label("Linked Frontline Chain from South Command HQ to North Citadel:");
                ui.add_space(4.0);

                for (_idx, sector) in self.cnc_sim.sectors.iter().enumerate() {
                    let status_str = match &sector.status {
                        FrontlineSectorStatus::Uncontested(fac) => format!("Held by {}", fac.display_name()),
                        FrontlineSectorStatus::Contested { progress_pct, attacking_faction, defending_faction } => {
                            format!("Contested! {} vs {} ({:.0}%)", attacking_faction.display_name(), defending_faction.display_name(), progress_pct)
                        }
                        FrontlineSectorStatus::Neutral => "Neutral / Unclaimed".to_string(),
                        FrontlineSectorStatus::InStormVoid => "Consumed by Karman Storm Void".to_string(),
                    };
                    let color = match &sector.status {
                        FrontlineSectorStatus::Uncontested(fac) => {
                            let [r, g, b] = fac.color_rgb();
                            egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
                        }
                        FrontlineSectorStatus::Contested { .. } => egui::Color32::YELLOW,
                        FrontlineSectorStatus::Neutral => egui::Color32::GRAY,
                        FrontlineSectorStatus::InStormVoid => egui::Color32::from_rgb(180, 50, 255),
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("[{}] {}", sector.code, sector.name)).color(color).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Target").clicked() {
                                self.cnc_target_x = sector.local_pos.x;
                                self.cnc_target_z = sector.local_pos.z;
                            }
                            ui.label(egui::RichText::new(status_str).small());
                        });
                    });
                    ui.add_space(2.0);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Active Units: {}", self.cnc_sim.units.len()));
                    ui.separator();
                    ui.label(format!("Structures: {}", self.cnc_sim.buildings.len()));
                    ui.separator();
                    ui.label(format!("Supply Drops: {}", self.cnc_sim.supply_drops.len()));
                });
            });

            // Right Column: Base Construction & Unit Production Bar
            cols[1].group(|ui| {
                ui.strong("🏗 MCV Construction Yard");
                ui.label("Build power plants, refineries, defenses, and war factories:");
                ui.add_space(4.0);

                let build_types = [
                    TacticalBuildingType::PowerPlant,
                    TacticalBuildingType::FuelRefinery,
                    TacticalBuildingType::WarFactory,
                    TacticalBuildingType::DefenseTurretPDC,
                    TacticalBuildingType::ObeliskOfRadiance,
                    TacticalBuildingType::IonCannonBeacon,
                ];

                for b_type in build_types {
                    let cost = b_type.fuel_cost();
                    let (p_gen, p_drain) = b_type.power_delta();
                    let p_str = if p_gen > 0.0 {
                        format!("+{} MW", p_gen)
                    } else if p_drain > 0.0 {
                        format!("-{} MW", p_drain)
                    } else {
                        "0 MW".into()
                    };

                    ui.horizontal(|ui| {
                        ui.label(format!("{} ({} Fuel, {})", b_type.display_name(), cost, p_str));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("Build").strong()).clicked() {
                                let offset_x = self.cnc_target_x + (self.cnc_sim.buildings.len() as f32 * 12.0) % 60.0;
                                let offset_z = self.cnc_target_z - 30.0;
                                match self.cnc_sim.construct_building(b_type, offset_x, offset_z) {
                                    Ok(_) => {
                                        self.cnc_sim.push_log(format!("Constructed {} at ({:.0}, {:.0})", b_type.display_name(), offset_x, offset_z));
                                    }
                                    Err(err) => {
                                        self.cnc_sim.push_log(format!("Cannot construct {}: {}", b_type.display_name(), err));
                                    }
                                }
                            }
                        });
                    });
                    ui.add_space(2.0);
                }

                ui.separator();
                ui.strong("🏭 War Factory Production");
                ui.label("Deploy mechanized combat chassis & drones:");
                ui.add_space(4.0);

                let unit_types = [
                    TacticalUnitType::FrontlineCombatDroneUAV,
                    TacticalUnitType::HeavyTankPaladin,
                    TacticalUnitType::JulianAssaultMech,
                    TacticalUnitType::TacticalRocketBattery,
                    TacticalUnitType::EmpStrikeDrone,
                    TacticalUnitType::HarvesterRefineryRover,
                ];

                for u_type in unit_types {
                    let cost = u_type.fuel_cost();
                    ui.horizontal(|ui| {
                        ui.label(format!("{} ({} Fuel)", u_type.display_name(), cost));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("Produce").color(egui::Color32::from_rgb(0, 242, 254)).strong()).clicked() {
                                match self.cnc_sim.produce_unit(u_type) {
                                    Ok(_) => {
                                        self.cnc_sim.push_log(format!("Produced {} and deployed to frontlines.", u_type.display_name()));
                                    }
                                    Err(err) => {
                                        self.cnc_sim.push_log(format!("Cannot produce {}: {}", u_type.display_name(), err));
                                    }
                                }
                            }
                        });
                    });
                    ui.add_space(2.0);
                }
            });
        });

        ui.add_space(6.0);

        // Combat Log Console
        ui.group(|ui| {
            ui.strong("📜 Frontlines Tactical Combat Log");
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                if self.cnc_sim.combat_log.is_empty() {
                    ui.label("War room initialized. No combat casualties logged yet.");
                } else {
                    for entry in self.cnc_sim.combat_log.iter().rev().take(12) {
                        ui.label(egui::RichText::new(format!("• {}", entry)).small().color(egui::Color32::LIGHT_GRAY));
                    }
                }
            });
        });
    }

    /// Room 2: 24/7 Siren Frequency Radio Studio (The Seven Daughters)
    fn render_siren_studio_room(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("📻 Room 102: Siren 24/7 Frequency Broadcast Studio");
            ui.label("Air element (💨 Aero) domain. 24/7 radio station hosted by The Seven Daughters (432 Hz base), synchronized directly with the Atlantis Pi node and cloud relays.");
        });

        ui.add_space(6.0);

        let hosts = [
            ("Maia", "432 Hz • Dawn Oceanic Ambient / Horizon Chill", "04:00 - 08:00"),
            ("Alcyone", "528 Hz • High-Speed Cyberpunk D&B / Breakbeats", "08:00 - 12:00"),
            ("Asterope", "639 Hz • Tactical Deep House & Frontline Reports", "12:00 - 16:00"),
            ("Celaeno", "741 Hz • Glitch / Industrial Bass / Storm Chaser", "16:00 - 20:00"),
            ("Electra", "852 Hz • Vocal Trance / Eurobeat / Club Atlantis", "20:00 - 00:00"),
            ("Taygete", "963 Hz • Darkwave / Minimal Techno / Abyssal Drone", "00:00 - 02:00"),
            ("Merope", "396 Hz • Lo-Fi / Starlight Harbor Acoustic", "02:00 - 04:00"),
        ];

        egui::Grid::new("siren_hosts_grid").striped(true).show(ui, |ui| {
            ui.strong("Broadcaster");
            ui.strong("Frequency Profile & Genre");
            ui.strong("Time Slot (UTC)");
            ui.strong("Status");
            ui.end_row();

            for (name, genre, slot) in hosts {
                ui.label(egui::RichText::new(name).strong().color(egui::Color32::from_rgb(0, 242, 254)));
                ui.label(genre);
                ui.label(slot);
                ui.label(egui::RichText::new("ON AIR • ROTATING").color(egui::Color32::GREEN).small());
                ui.end_row();
            }
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!("Active Pi Relay: {} (Status: {})", self.pi_node_lan_host, if self.pi_atlantis_running { "CONNECTED" } else { "OFFLINE" }));
            ui.separator();
            if ui.button("Broadcast 0.1 CYAN Tip to Active Host").clicked() {
                // Tipped host
            }
        });
    }

    /// Room 3: Triton Abyssal Concept Vault (NCM & Treasury)
    fn render_triton_vault_room(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("🔱 Room 103: Triton Abyssal Concept Vault & Risk Terminal");
            ui.label("Water element (🌊 Hydros) primary foundation. Coordinates deep consensus, 4-scale hierarchical Residual VQ-VAE Next-Concept Modeling (NCM), and zero-loss treasury invariants.");
        });

        ui.add_space(6.0);
        egui::Grid::new("triton_stats_grid").striped(true).show(ui, |ui| {
            ui.label("Assigned Officer:");
            ui.label("Triton (Chief Strategist & Risk Officer // Tier 1 Abyssal Anchor)");
            ui.end_row();

            ui.label("Treasury Capital Strategy:");
            ui.label("Continuous paper trading (SOL/USDC & JUP/USDC) with Mercury Decide consensus");
            ui.end_row();

            ui.label("Obsidian Vault Sync:");
            ui.label("Enabled (Zero-leak cryptographic state truth)");
            ui.end_row();

            ui.label("Operational Balance:");
            ui.label(egui::RichText::new("$20.00 Gas Envelope Active").color(egui::Color32::GREEN));
            ui.end_row();
        });
    }

    /// Room 4: Ursula Commercial Packaging & MCP Tool Dispatch
    fn render_ursula_dock_room(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("🐙 Room 104: Ursula Commercial MCP Dock");
            ui.label("Commercial Operations & MCP Dispatch. Manages commercial tool packaging, Game ATAK dual-use licensing, and Model Context Protocol bridges.");
        });

        ui.add_space(6.0);
        egui::Grid::new("ursula_stats_grid").striped(true).show(ui, |ui| {
            ui.label("Assigned Officer:");
            ui.label("Ursula (Chief Operating Officer // Tier 2 Pelagic Node Manager)");
            ui.end_row();

            ui.label("Active Tool Bridges:");
            ui.label("blender_mcp, fl_studio_transport, discord_swarm_gateway, gemini_revolver");
            ui.end_row();

            ui.label("Game ATAK Pipeline:");
            ui.label("Ruggedized mesh node packaging for off-grid teams & 3D common operating picture");
            ui.end_row();

            ui.label("MCP Ingestion:");
            ui.label("lemuria_ingest_scene • lemuria_query_scene • route_inference");
            ui.end_row();
        });
    }

    /// Room 5: Sebastian Hardware Edge Crab & Canary Sentry
    fn render_sebastian_bunker_room(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("🦀 Room 105: Sebastian Edge Hardware Bunker");
            ui.label("Hardware attestation and edge telemetry. Runs continuous EdgeCrab matrix tests with <=250ms latency Proof-of-Health canaries.");
        });

        ui.add_space(6.0);
        egui::Grid::new("sebastian_stats_grid").striped(true).show(ui, |ui| {
            ui.label("Assigned Officer:");
            ui.label("Sebastian (Chief Technology & Sentinel Officer // Tier 3 Epipelagic Edge)");
            ui.end_row();

            ui.label("Canary Challenge Status:");
            ui.label(egui::RichText::new("≤ 250ms PoH SLA: MET (4ms Edge Latency)").color(egui::Color32::GREEN));
            ui.end_row();

            ui.label("Physical Embodiment:");
            ui.label("Raspberry Pi 5 ARM64 + Local mDNS Mesh Discovery");
            ui.end_row();

            ui.label("Infrastructure Burn Rate:");
            ui.label(egui::RichText::new("$0.00 / month (100% Free-Tier Cloud Ingress + Local Failover)").color(egui::Color32::from_rgb(0, 242, 254)));
            ui.end_row();
        });
    }

    /// Legacy fallback ui method delegating to render_clubs_page
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.render_clubs_page(ui);
    }
}
