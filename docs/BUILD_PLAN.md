# HeavenlyBond Lite Build Plan

> Historical design reference. Reviewed 2026-11-11: Flutter/TithX descriptions and completion claims below do not establish current runtime behavior. Use the [active README](../README.md) and [platform runtime status](../../PLATFORM_RUNTIME_STATUS.md) for the Rust client and remaining integrations.

This document outlines the step-by-step roadmap to build out the remaining features, transitioning from the current MVP to the full HeavenlyBond Lite vision.

## Sprint 1: Foundation (Current State)

*   **Data Models:** Freezed models for Task, Offer, UserProfile, Team, Credential.
*   **Firebase Integration:** Basic Firestore security rules for role-based access.
*   **Navigation:** Persistent bottom navigation using `go_router`.
*   **Board UI:** Initial feed of tasks showing minimal details (Bounty, Location, Title, Status).

## Sprint 2: Task Creation Wizard

*   **Goal:** Allow users to create comprehensive tasks.
*   **Tasks:**
    *   Build out `CreateTaskPage` into a multi-step wizard.
    *   **Step 1: Basic Info** (Title, Description, Category).
    *   **Step 2: Logistics** (Location map picker, Desired Date, Number of Workers).
    *   **Step 3: Requirements** (Skills tags, Equipment list, Credential requirements list).
    *   **Step 4: Funding** (Set Bounty/Budget).
    *   **Step 5: Review & Publish** (Submits to Firestore, transitions state to `PUBLISHED`).

## Sprint 3: Bidding & Offers Mechanism

*   **Goal:** Enable providers to place bids and task creators to manage them.
*   **Tasks:**
    *   Implement "Make an Offer" modal on `TaskDetailPage`.
    *   Build out `BidsPage` to show:
        *   **My Offers:** Bids the user has placed (Status: Pending, Accepted, Rejected, Countered).
        *   **Offers Received:** Bids received on tasks the user created.
    *   Implement `OfferDetailPage` to allow Creators to view bid details, user profiles of bidders, and Accept/Reject/Counter.
    *   **Cloud Functions:** Implement secure Cloud Functions for Creators to read bids (since direct Firestore reads are restricted for privacy) and to handle state transitions (e.g., Accepting an offer changes Task status to `PROVIDER_SELECTED`).

## Sprint 4: Task Execution & Verification

*   **Goal:** Track task progress and verify completion for payment release.
*   **Tasks:**
    *   UI updates for Providers to mark a task as `IN_PROGRESS` and `SUBMITTED_FOR_VERIFICATION`.
    *   Image upload integration for Providers to upload completion photos.
    *   UI for Creators to review completion photos and Approve (`APPROVED`) or Dispute (`DISPUTED`).
    *   Stripe Connect integration placeholder for holding funds in escrow and releasing upon approval (`PAYMENT_RELEASED`).

## Sprint 5: Credentials & Licensing System

*   **Goal:** Allow providers to prove qualifications for regulated tasks.
*   **Tasks:**
    *   Build out `CredentialsPage` (accessed via Profile).
    *   Document upload via `file_picker` or `image_picker`.
    *   Admin dashboard (or Cloud Functions) for Admin verification of uploaded documents.
    *   Enforce required credentials before allowing a user to submit an offer on specific tasks.

## Sprint 6: Teams & Guilds Foundation

*   **Goal:** Allow users to form teams/guilds to tackle larger tasks and build complex organizations.
*   **Tasks:**
    *   Build out `TeamsPage` and `TeamDetailPage`.
    *   Create Team functionality (Name, Description).
    *   Invite users to Team (creates `Membership` records).
    *   Allow an Offer to be submitted *on behalf of a team* rather than an individual.
    *   **Completed:** Implemented Constellation Map using the 3D Fluorescent Engine.
    *   **Completed:** Expanded Team data model to support branching (`parentIds`), sister clubs (`sisterClubIds`), and partnerships (`treatyIds`).
    *   **Completed:** Dynamic permission resolution (child cliq members inherit mother org access).

## Sprint 7: Official Projects & Event Coordination

*   **Goal:** Transform complex tasks into managed projects.
*   **Tasks:**
    *   Implement the project state machine transition (`TASK` → `PROJECT`).
    *   Build out Project management UI for the assigned Project Administrator.
    *   Sub-task creation within a project.
    *   Role assignment within the project (Security Team, Food Team, Setup Crew).
    *   Project budget tracking and sub-bounty distribution.

## Ongoing: Refinement & Testing

*   Implement unit and widget tests for key workflows.
*   Refine UI/UX based on community feedback.
*   Implement Push Notifications for bid updates, state changes, and messages.