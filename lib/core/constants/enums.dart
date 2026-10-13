// ── Task status state machine ─────────────────────────────────────────────────
//
// Valid transitions are enforced server-side in Cloud Functions.
// This enum is the single source of truth used by both Flutter and Functions.
//
enum TaskStatus {
  draft,
  published,
  fundingOpen,
  bidding,
  providerSelected,
  teamForming,
  scheduled,
  inProgress,
  submittedForVerification,
  approved,
  paymentReleased,
  completed,
  cancelled,
  disputed,
  expired;

  /// Human-readable label shown in the UI.
  String get label => switch (this) {
    draft => 'Draft',
    published => 'Open',
    fundingOpen => 'Accepting Funds',
    bidding => 'Bidding Open',
    providerSelected => 'Provider Selected',
    teamForming => 'Team Forming',
    scheduled => 'Scheduled',
    inProgress => 'In Progress',
    submittedForVerification => 'Pending Review',
    approved => 'Approved',
    paymentReleased => 'Payment Released',
    completed => 'Completed',
    cancelled => 'Cancelled',
    disputed => 'Disputed',
    expired => 'Expired',
  };

  bool get isTerminal => switch (this) {
    completed || cancelled || expired => true,
    _ => false,
  };

  bool get isActive => switch (this) {
    inProgress || submittedForVerification || approved => true,
    _ => false,
  };

  bool get isOpen => switch (this) {
    published || fundingOpen || bidding => true,
    _ => false,
  };
}

// ── Offer status ──────────────────────────────────────────────────────────────
enum OfferStatus {
  pending,
  accepted,
  rejected,
  countered,
  withdrawn,
  expired;

  String get label => switch (this) {
    pending => 'Pending',
    accepted => 'Accepted',
    rejected => 'Rejected',
    countered => 'Countered',
    withdrawn => 'Withdrawn',
    expired => 'Expired',
  };
}

// ── Credential types ──────────────────────────────────────────────────────────
enum CredentialType {
  driversLicense,
  cdl,
  electricalLicense,
  contractorLicense,
  insurance,
  foodHandler,
  backgroundCheck,
  other;

  String get label => switch (this) {
    driversLicense => "Driver's License",
    cdl => 'Commercial Driver License (CDL)',
    electricalLicense => 'Electrical License',
    contractorLicense => 'Contractor License',
    insurance => 'Insurance Certificate',
    foodHandler => 'Food Handler Certification',
    backgroundCheck => 'Background Check',
    other => 'Other',
  };
}

// ── Credential verification status ───────────────────────────────────────────
enum CredentialStatus {
  uploaded,
  underReview,
  verified,
  rejected,
  expired;

  String get label => switch (this) {
    uploaded => 'Uploaded',
    underReview => 'Under Review',
    verified => 'Verified',
    rejected => 'Rejected',
    expired => 'Expired',
  };
}

// ── Membership roles ──────────────────────────────────────────────────────────
enum MemberRole { owner, admin, member, viewer }

// ── Membership status ─────────────────────────────────────────────────────────
enum MemberStatus { active, invited, suspended }

// ── Project status ────────────────────────────────────────────────────────────
enum ProjectStatus { task, project, adminSelected, requirementsMet, official }

// ── Task category ─────────────────────────────────────────────────────────────
enum TaskCategory {
  cleanup,
  moving,
  delivery,
  event,
  repair,
  landscaping,
  other;

  String get label => switch (this) {
    cleanup => 'Cleanup',
    moving => 'Moving',
    delivery => 'Delivery',
    event => 'Event',
    repair => 'Repair',
    landscaping => 'Landscaping',
    other => 'Other',
  };

  String get emoji => switch (this) {
    cleanup => '🧹',
    moving => '📦',
    delivery => '🚚',
    event => '🎉',
    repair => '🔧',
    landscaping => '🌿',
    other => '📋',
  };
}

// ── War status ────────────────────────────────────────────────────────────────
enum WarStatus {
  pending,
  active,
  ceasefire,
  victory,
  defeat;

  String get label => switch (this) {
    pending => 'Pending',
    active => 'Active',
    ceasefire => 'Ceasefire',
    victory => 'Victory',
    defeat => 'Defeat',
  };

  String get emoji => switch (this) {
    pending => '⏳',
    active => '⚔️',
    ceasefire => '🕊️',
    victory => '🏆',
    defeat => '💀',
  };
}

// ── Listing type ──────────────────────────────────────────────────────────────
enum ListingType {
  forSale,
  wantedToBuy,
  bookService;

  String get label => switch (this) {
    forSale => 'For Sale',
    wantedToBuy => 'Wanted',
    bookService => 'Book Service',
  };

  String get emoji => switch (this) {
    forSale => '💰',
    wantedToBuy => '🛒',
    bookService => '📅',
  };
}
