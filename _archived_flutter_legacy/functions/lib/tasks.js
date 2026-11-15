"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.expireStaleTasksCron = exports.onTaskStatusChanged = void 0;
const firestore_1 = require("firebase-functions/v2/firestore");
const scheduler_1 = require("firebase-functions/v2/scheduler");
const firestore_2 = require("firebase-admin/firestore");
const sendPush_1 = require("./sendPush");
const db = (0, firestore_2.getFirestore)();
// ── Valid task state transitions ──────────────────────────────────────────────
const VALID_TRANSITIONS = {
    draft: ['published', 'cancelled'],
    published: ['fundingOpen', 'bidding', 'cancelled', 'expired'],
    fundingOpen: ['bidding', 'cancelled', 'expired'],
    bidding: ['providerSelected', 'cancelled', 'expired'],
    providerSelected: ['teamForming', 'scheduled', 'cancelled'],
    teamForming: ['scheduled', 'cancelled'],
    scheduled: ['inProgress', 'cancelled'],
    inProgress: ['submittedForVerification', 'disputed'],
    submittedForVerification: ['approved', 'disputed'],
    approved: ['paymentReleased'],
    paymentReleased: ['completed'],
    // Terminal states
    completed: [],
    cancelled: [],
    disputed: ['completed', 'cancelled'], // admin only
    expired: [],
};
// ── onTaskStatusChanged: validate + audit transitions ─────────────────────────
exports.onTaskStatusChanged = (0, firestore_1.onDocumentUpdated)('tasks/{taskId}', async (event) => {
    const before = event.data?.before.data();
    const after = event.data?.after.data();
    if (!before || !after || before.status === after.status)
        return;
    const taskId = event.params.taskId;
    const fromStatus = before.status;
    const toStatus = after.status;
    // Validate transition
    const allowed = VALID_TRANSITIONS[fromStatus] ?? [];
    if (!allowed.includes(toStatus)) {
        console.error(`INVALID TRANSITION: ${fromStatus} → ${toStatus} on task ${taskId}`);
        // Revert — write back previous status
        await db.collection('tasks').doc(taskId).update({
            status: fromStatus,
            updatedAt: firestore_2.FieldValue.serverTimestamp(),
        });
        return;
    }
    // Append to status history audit trail
    await db
        .collection('tasks')
        .doc(taskId)
        .collection('statusHistory')
        .add({
        fromStatus,
        toStatus,
        changedAt: firestore_2.FieldValue.serverTimestamp(),
        // changedBy is set by the callable function that triggered the write
    });
    console.log(`Task ${taskId}: ${fromStatus} → ${toStatus}`);
    // Notify creator/provider based on status change
    let notifyUserId = after.creatorId;
    let title = 'Task Update';
    let body = `Your task "${after.title}" is now ${toStatus}.`;
    let shouldNotify = false;
    if (toStatus === 'expired') {
        title = 'Task Expired';
        body = `Your task "${after.title}" has expired.`;
        shouldNotify = true;
    }
    else if (toStatus === 'inProgress' && after.selectedProviderId) {
        notifyUserId = after.selectedProviderId;
        title = 'Task In Progress';
        body = `The task "${after.title}" is now in progress.`;
        shouldNotify = true;
    }
    else if (toStatus === 'submittedForVerification') {
        title = 'Task Pending Review';
        body = `The provider has submitted "${after.title}" for verification.`;
        shouldNotify = true;
    }
    else if (toStatus === 'approved' && after.selectedProviderId) {
        notifyUserId = after.selectedProviderId;
        title = 'Task Approved';
        body = `Your work on "${after.title}" has been approved!`;
        shouldNotify = true;
    }
    else if (toStatus === 'completed') {
        // We could notify both, but let's notify creator for now
        title = 'Task Completed';
        body = `The task "${after.title}" has been successfully completed.`;
        shouldNotify = true;
    }
    if (shouldNotify) {
        await (0, sendPush_1.sendPushAndSaveNotification)(notifyUserId, title, body, { type: 'task_status', taskId, newStatus: toStatus }, `/board/task/${taskId}`);
    }
});
});
// ── acceptOffer: Callable — task creator selects a provider ──────────────────
// (Implemented in escrow.ts as a callable; status transition handled here)
// ── expireStaleTasksCron: daily job to mark stale tasks as expired ─────────────
exports.expireStaleTasksCron = (0, scheduler_1.onSchedule)('every 24 hours', async () => {
    const now = firestore_2.Timestamp.now();
    const staleStatuses = ['published', 'fundingOpen', 'bidding'];
    for (const status of staleStatuses) {
        const snap = await db.collection('tasks')
            .where('status', '==', status)
            .where('desiredCompletionDate', '<', now)
            .limit(100)
            .get();
        const batch = db.batch();
        for (const doc of snap.docs) {
            batch.update(doc.ref, {
                status: 'expired',
                updatedAt: firestore_2.FieldValue.serverTimestamp(),
            });
        }
        if (!snap.empty)
            await batch.commit();
        console.log(`Expired ${snap.size} tasks with status '${status}'`);
    }
});
//# sourceMappingURL=tasks.js.map