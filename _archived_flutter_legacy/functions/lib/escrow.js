"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.releaseEscrow = exports.acceptOffer = void 0;
const https_1 = require("firebase-functions/v2/https");
const firestore_1 = require("firebase-admin/firestore");
const params_1 = require("firebase-functions/params");
const stripe_1 = __importDefault(require("stripe"));
const db = (0, firestore_1.getFirestore)();
const stripeSecretKey = (0, params_1.defineSecret)('STRIPE_SECRET_KEY');
const db = (0, firestore_1.getFirestore)();
const params_1 = require("firebase-functions/params");
const stripe_1 = __importDefault(require("stripe"));
const db = (0, firestore_1.getFirestore)();
const STRIPE_SECRET_KEY = (0, params_1.defineSecret)('STRIPE_SECRET_KEY');
// ── acceptOffer: Callable — task creator selects a winning bid ────────────────
//
// Called by the task creator. Transitions task → providerSelected,
// sets selectedProviderId, and rejects all other pending bids.
exports.acceptOffer = (0, https_1.onCall)({ region: 'us-central1' }, async (req) => {
    const { taskId, offerId } = req.data;
    const uid = req.auth?.uid;
    if (!uid)
        throw new https_1.HttpsError('unauthenticated', 'Must be signed in');
    if (!taskId || !offerId) {
        throw new https_1.HttpsError('invalid-argument', 'taskId and offerId are required');
    }
    await db.runTransaction(async (t) => {
        const taskRef = db.collection('tasks').doc(taskId);
        const offerRef = db.collection('bids').doc(offerId);
        const [taskSnap, offerSnap] = await Promise.all([
            t.get(taskRef), t.get(offerRef),
        ]);
        if (!taskSnap.exists)
            throw new https_1.HttpsError('not-found', 'Task not found');
        if (!offerSnap.exists)
            throw new https_1.HttpsError('not-found', 'Offer not found');
        const task = taskSnap.data();
        const offer = offerSnap.data();
        if (task.creatorId !== uid) {
            throw new https_1.HttpsError('permission-denied', 'Only the task creator can accept bids');
        }
        if (!['published', 'fundingOpen', 'bidding'].includes(task.status)) {
            throw new https_1.HttpsError('failed-precondition', `Task is not accepting bids (status: ${task.status})`);
        }
        if (offer.taskId !== taskId) {
            throw new https_1.HttpsError('invalid-argument', 'Offer does not belong to this task');
        }
        if (offer.status !== 'pending') {
            throw new https_1.HttpsError('failed-precondition', `Offer is not pending (status: ${offer.status})`);
        }
        // Accept the winning offer
        t.update(offerRef, { status: 'accepted' });
        // Transition task to providerSelected
        t.update(taskRef, {
            status: 'providerSelected',
            selectedProviderId: offer.bidderId,
            updatedAt: firestore_1.FieldValue.serverTimestamp(),
        });
    });
    // Reject all other pending bids for this task (outside transaction for scale)
    const otherBids = await db.collection('bids')
        .where('taskId', '==', taskId)
        .where('status', '==', 'pending')
        .get();
    const batch = db.batch();
    for (const doc of otherBids.docs) {
        if (doc.id !== offerId) {
            batch.update(doc.ref, { status: 'rejected' });
        }
    }
    if (!otherBids.empty)
        await batch.commit();
    return { success: true };
});
// ── releaseEscrow: Callable — task creator approves work & releases payment ───
//
// Full Stripe integration wired here in Sprint 5.
// For now, transitions task through approved → paymentReleased → completed.
exports.releaseEscrow = (0, https_1.onCall)({ region: 'us-central1' }, async (req) => {
exports.releaseEscrow = (0, https_1.onCall)({ region: 'us-central1', secrets: [STRIPE_SECRET_KEY] }, async (req) => {
    const { taskId } = req.data;
    const uid = req.auth?.uid;
    if (!uid)
        throw new https_1.HttpsError('unauthenticated', 'Must be signed in');
    await db.runTransaction(async (t) => {
    const stripe = new stripe_1.default(STRIPE_SECRET_KEY.value(), {
        apiVersion: '2023-10-16', // Use type assertion to avoid TypeScript error on older/newer SDK types
    });
    // Step 1: Read all necessary data in a transaction and verify preconditions
    const transferDetails = await db.runTransaction(async (t) => {
        const taskRef = db.collection('tasks').doc(taskId);
        const taskSnap = await t.get(taskRef);
        if (!taskSnap.exists)
            throw new https_1.HttpsError('not-found', 'Task not found');
        const task = taskSnap.data();
        if (task.creatorId !== uid) {
            throw new https_1.HttpsError('permission-denied', 'Only the task creator can release payment');
        }
        if (task.status !== 'submittedForVerification') {
            throw new https_1.HttpsError('failed-precondition', `Task must be in submittedForVerification state (current: ${task.status})`);
        }
        // TODO Sprint 5: call Stripe transfer API here
        // await stripe.transfers.create({ amount: task.budgetAmount * 100, destination: providerStripeId });
        if (!task.selectedProviderId) {
            throw new https_1.HttpsError('failed-precondition', 'Task does not have a selected provider');
        }
        // Fetch provider's Stripe Account ID
        const providerRef = db.collection('users').doc(task.selectedProviderId);
        const providerSnap = await t.get(providerRef);
        if (!providerSnap.exists) {
            throw new https_1.HttpsError('not-found', 'Provider not found');
        }
        const provider = providerSnap.data();
        const providerStripeId = provider.stripeAccountId;
        if (!providerStripeId) {
            throw new https_1.HttpsError('failed-precondition', 'Provider does not have a linked Stripe account');
        }
        // Fetch platform settings to determine fee
        const settingsRef = db.collection('settings').doc('platform');
        const settingsSnap = await t.get(settingsRef);
        let platformFeePercentage = 10; // Default to 10% if not set
        if (settingsSnap.exists) {
            const settings = settingsSnap.data();
            if (typeof settings.platformFeePercentage === 'number') {
                platformFeePercentage = settings.platformFeePercentage;
            }
        }
        return {
            amount: task.budgetAmount || 0,
            currency: task.currency || 'usd',
            platformFeePercentage,
            providerStripeId,
        };
    });
    // Step 2: Calculate transfer amount
    const feeAmount = (transferDetails.amount * transferDetails.platformFeePercentage) / 100;
    const transferAmount = transferDetails.amount - feeAmount;
    const currency = transferDetails.currency.toLowerCase();
    // Stripe expects amounts in cents/smallest currency unit for zero-decimal currencies
    const isZeroDecimalCurrency = ['bif', 'clp', 'djf', 'gnf', 'jpy', 'kmf', 'krw', 'mga', 'pyg', 'rwf', 'ugx', 'vnd', 'vuv', 'xaf', 'xof', 'xpf'].includes(currency);
    const transferAmountInSmallestUnit = isZeroDecimalCurrency ? Math.round(transferAmount) : Math.round(transferAmount * 100);
    // Step 3: Perform the Stripe transfer OUTSIDE the transaction
    if (transferAmountInSmallestUnit > 0) {
        try {
            await stripe.transfers.create({
                amount: transferAmountInSmallestUnit,
                currency: currency,
                destination: transferDetails.providerStripeId,
                transfer_group: `task_${taskId}`,
            }, {
                idempotencyKey: `transfer_${taskId}`, // Ensures the transfer only happens once
            });
        }
        catch (error) {
            console.error('Stripe transfer failed:', error);
            throw new https_1.HttpsError('internal', `Stripe transfer failed: ${error.message || 'Unknown error'}`);
        }
    }
    // Step 4: Update the task status to approved in a new transaction
    await db.runTransaction(async (t) => {
        const taskRef = db.collection('tasks').doc(taskId);
        const taskSnap = await t.get(taskRef);
        // Safety check again in case state changed while Stripe transfer was executing
        if (!taskSnap.exists)
            return;
        const task = taskSnap.data();
        if (task.status !== 'submittedForVerification')
            return;
        t.update(taskRef, {
            status: 'approved',
            updatedAt: firestore_1.FieldValue.serverTimestamp(),
        });
        // Append status history
        t.set(taskRef.collection('statusHistory').doc(), {
            fromStatus: 'submittedForVerification',
            toStatus: 'approved',
            changedBy: uid,
            changedAt: firestore_1.FieldValue.serverTimestamp(),
            reason: 'creator_approved_work',
        });
    });
    return { success: true };
});
//# sourceMappingURL=escrow.js.map