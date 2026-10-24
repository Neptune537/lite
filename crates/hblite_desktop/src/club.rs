//! Club tab: members + AI daemons on the club's home nodes.
//! The officer admin section is only rendered for registered officers.
//!
//! Roster data is local seed state until a membership store and node
//! registry exist. Server-side rules must still enforce officer-only writes.

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Owner,
    Officer,
    Member,
    Viewer,
}

impl Role {
    pub const ALL: [Role; 4] = [Role::Owner, Role::Officer, Role::Member, Role::Viewer];

    pub fn label(self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Officer => "officer",
            Role::Member => "member",
            Role::Viewer => "viewer",
        }
    }

    pub fn is_officer(self) -> bool {
        matches!(self, Role::Owner | Role::Officer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Suspended,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Suspended => "suspended",
        }
    }
}

pub struct Member {
    pub did: String,
    pub name: String,
    pub role: Role,
    pub status: Status,
}

pub struct Daemon {
    pub name: String,
    pub home_node: String,
    pub running: bool,
}

pub struct ClubState {
    /// DID of the local user; officer status is derived from the roster.
    pub viewer_did: String,
    pub home_nodes: Vec<String>,
    pub members: Vec<Member>,
    pub daemons: Vec<Daemon>,
}

impl Default for ClubState {
    fn default() -> Self {
        Self {
            viewer_did: "did:atlantis:you".into(),
            home_nodes: vec!["home-1".into(), "home-2".into()],
            members: vec![
                Member {
                    did: "did:atlantis:you".into(),
                    name: "You".into(),
                    role: Role::Owner,
                    status: Status::Active,
                },
                Member {
                    did: "did:atlantis:staff_bob".into(),
                    name: "Bob Ross".into(),
                    role: Role::Member,
                    status: Status::Active,
                },
            ],
            daemons: vec![
                Daemon { name: "Scout".into(), home_node: "home-1".into(), running: true },
                Daemon { name: "Ledger".into(), home_node: "home-2".into(), running: false },
            ],
        }
    }
}

impl ClubState {
    /// True only for an active owner/officer of the club.
    pub fn viewer_is_officer(&self) -> bool {
        self.members.iter().any(|m| {
            m.did == self.viewer_did && m.status == Status::Active && m.role.is_officer()
        })
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("👥 My Club");
        ui.separator();

        ui.strong("Home Nodes");
        for n in &self.home_nodes {
            let count = self.daemons.iter().filter(|d| &d.home_node == n).count();
            ui.label(format!("🖧 {n} — {count} daemon(s)"));
        }

        ui.add_space(8.0);
        ui.strong(format!("Members ({})", self.members.len()));
        for m in &self.members {
            ui.label(format!("👤 {} — {} • {}", m.name, m.role.label(), m.status.label()));
        }

        ui.add_space(8.0);
        ui.strong("AI Daemons");
        for d in &self.daemons {
            let state = if d.running { "running" } else { "stopped" };
            ui.label(format!("🤖 {} @ {} — {state}", d.name, d.home_node));
        }

        // Revealed only to registered officers.
        if !self.viewer_is_officer() {
            return;
        }
        ui.add_space(12.0);
        ui.separator();
        ui.heading("🛡️ Officer Admin");

        let viewer = self.viewer_did.clone();
        let mut remove: Option<usize> = None;
        for (i, m) in self.members.iter_mut().enumerate() {
            ui.collapsing(format!("{} ({})", m.name, m.role.label()), |ui| {
                if m.role == Role::Owner || m.did == viewer {
                    ui.label("Owner / self cannot be modified here.");
                    return;
                }
                ui.horizontal(|ui| {
                    for r in Role::ALL {
                        if r != Role::Owner {
                            ui.selectable_value(&mut m.role, r, r.label());
                        }
                    }
                });
                ui.horizontal(|ui| {
                    let toggle = if m.status == Status::Suspended { "Reinstate" } else { "Suspend" };
                    if ui.button(toggle).clicked() {
                        m.status = if m.status == Status::Suspended {
                            Status::Active
                        } else {
                            Status::Suspended
                        };
                    }
                    if ui.button("Remove").clicked() {
                        remove = Some(i);
                    }
                });
            });
        }
        if let Some(i) = remove {
            self.members.remove(i);
        }

        ui.add_space(8.0);
        ui.strong("Daemon Control");
        for d in &mut self.daemons {
            ui.checkbox(&mut d.running, format!("{} @ {}", d.name, d.home_node));
        }
    }
}
