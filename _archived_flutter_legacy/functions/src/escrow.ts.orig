import { onCall, HttpsError } from 'firebase-functions/v2/https';
import { getFirestore, FieldValue } from 'firebase-admin/firestore';

const db = getFirestore();

// ── acceptOffer: Callable — task creator selects a winning bid ────────────────
//
// Called by the task creator. Transitions task → providerSelected,
// sets selectedProviderId, and rejects all other pending bids.
export const acceptOffer = onCall({ region: 'us-central1' }, async (req) => {
  const { taskId, offerId } = req.data as { taskId: string; offerId: string };
  const uid = req.auth?.uid;
  if (!uid) throw new HttpsError('unauthenticated', 'Must be signed in');
  if (!taskId || !offerId) {
    throw new HttpsError('invalid-argument', 'taskId and offerId are required');
  }

  await db.runTransaction(async (t) => {
    const taskRef = db.collection('tasks').doc(taskId);
    const offerRef = db.collection('bids').doc(offerId);

    const [taskSnap, offerSnap] = await Promise.all([
      t.get(taskRef), t.get(offerRef),
    ]);

    if (!taskSnap.exists) throw new HttpsError('not-found', 'Task not found');
    if (!offerSnap.exists) throw new HttpsError('not-found', 'Offer not found');

    const task = taskSnap.data()!;
    const offer = offerSnap.data()!;

    if (task.creatorId !== uid) {
      throw new HttpsError('permission-denied', 'Only the task creator can accept bids');
    }
    if (!['published', 'fundingOpen', 'bidding'].includes(task.status)) {
      throw new HttpsError('failed-precondition', `Task is not accepting bids (status: ${task.status})`);
    }
    if (offer.taskId !== taskId) {
      throw new HttpsError('invalid-argument', 'Offer does not belong to this task');
    }
    if (offer.status !== 'pending') {
      throw new HttpsError('failed-precondition', `Offer is not pending (status: ${offer.status})`);
    }

    // Accept the winning offer
    t.update(offerRef, { status: 'accepted' });

    // Transition task to providerSelected
    t.update(taskRef, {
      status: 'providerSelected',
      selectedProviderId: offer.bidderId,
      updatedAt: FieldValue.serverTimestamp(),
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
  if (!otherBids.empty) await batch.commit();

  return { success: true };
});

// ── releaseEscrow: Callable — task creator approves work & releases payment ───
//
// Full Stripe integration wired here in Sprint 5.
// For now, transitions task through approved → paymentReleased → completed.
export const releaseEscrow = onCall({ region: 'us-central1' }, async (req) => {
  const { taskId } = req.data as { taskId: string };
  const uid = req.auth?.uid;
  if (!uid) throw new HttpsError('unauthenticated', 'Must be signed in');

  await db.runTransaction(async (t) => {
    const taskRef = db.collection('tasks').doc(taskId);
    const taskSnap = await t.get(taskRef);

    if (!taskSnap.exists) throw new HttpsError('not-found', 'Task not found');
    const task = taskSnap.data()!;

    if (task.creatorId !== uid) {
      throw new HttpsError('permission-denied', 'Only the task creator can release payment');
    }
    if (task.status !== 'submittedForVerification') {
      throw new HttpsError(
        'failed-precondition',
        `Task must be in submittedForVerification state (current: ${task.status})`,
      );
    }

    // TODO Sprint 5: call Stripe transfer API here
    // await stripe.transfers.create({ amount: task.budgetAmount * 100, destination: providerStripeId });

    t.update(taskRef, {
      status: 'approved',
      updatedAt: FieldValue.serverTimestamp(),
    });

    // Append status history
    t.set(taskRef.collection('statusHistory').doc(), {
      fromStatus: 'submittedForVerification',
      toStatus: 'approved',
      changedBy: uid,
      changedAt: FieldValue.serverTimestamp(),
      reason: 'creator_approved_work',
    });
  });

  return { success: true };
});
