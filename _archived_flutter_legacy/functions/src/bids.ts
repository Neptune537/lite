import { onDocumentCreated, onDocumentUpdated } from 'firebase-functions/v2/firestore';
import { getFirestore, FieldValue } from 'firebase-admin/firestore';
import { sendPushAndSaveNotification } from './sendPush';

const db = getFirestore();

// ── Helper: idempotency dedup ─────────────────────────────────────────────────
async function alreadyProcessed(eventId: string, taskId: string): Promise<boolean> {
  const dedupRef = db.collection('processedEvents').doc(eventId);
  let seen = false;
  await db.runTransaction(async (t) => {
    const snap = await t.get(dedupRef);
    if (snap.exists) { seen = true; return; }
    const ttl = new Date(Date.now() + 24 * 60 * 60 * 1000); // 24h TTL
    t.set(dedupRef, { processedAt: FieldValue.serverTimestamp(), taskId, ttl });
  });
  return seen;
}

// ── onNewBid: notify task creator when a new bid arrives ─────────────────────
export const onNewBid = onDocumentCreated('bids/{bidId}', async (event) => {
  const bid = event.data?.data();
  if (!bid) return;

  const taskSnap = await db.collection('tasks').doc(bid.taskId).get();
  const task = taskSnap.data();
  if (!task) return;

  await sendPushAndSaveNotification(
    task.creatorId,
    'New Bid Received',
    `Someone placed a bid of $${bid.amount} on "${task.title}"`,
    { type: 'new_bid', taskId: bid.taskId, bidId: event.params.bidId },
    `/board/task/${bid.taskId}`
  );
});

// ── onBidStatusChanged: notify bidder when their bid status changes ───────────
export const onBidStatusChanged = onDocumentUpdated('bids/{bidId}', async (event) => {
  if (await alreadyProcessed(event.id, event.params.bidId)) return;

  const before = event.data?.before.data();
  const after = event.data?.after.data();
  if (!before || !after || before.status === after.status) return;

  const messages: Record<string, string> = {
    accepted: '🎉 Your bid was accepted! Get to work.',
    rejected: 'Your bid was not selected for this task.',
    withdrawn: 'Your bid has been withdrawn.',
    countered: '↩ The task creator sent a counteroffer.',
    expired: 'Your bid has expired.',
  };

  const body = messages[after.status] ?? 'Your bid status changed.';
  await sendPushAndSaveNotification(
    after.bidderId,
    'Bid Update',
    body,
    { type: 'bid_status', bidId: event.params.bidId, taskId: after.taskId, newStatus: after.status },
    `/bids/${event.params.bidId}`
  );
});
