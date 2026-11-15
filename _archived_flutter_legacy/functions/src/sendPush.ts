import { getFirestore, FieldValue } from 'firebase-admin/firestore';
import { getMessaging } from 'firebase-admin/messaging';

const db = getFirestore();

export async function sendPushAndSaveNotification(
  userId: string,
  title: string,
  body: string,
  data: Record<string, string>,
  route?: string,
): Promise<void> {
  // 1. Save to in-app notifications
  await db.collection('users').doc(userId).collection('notifications').add({
    userId,
    title,
    body,
    route: route ?? null,
    isRead: false,
    createdAt: FieldValue.serverTimestamp(),
    data,
  });

  // 2. Send FCM push
  const userSnap = await db.collection('users').doc(userId).get();
  const fcmToken = userSnap.data()?.fcmToken as string | undefined;
  if (!fcmToken) return;

  try {
    await getMessaging().send({
      token: fcmToken,
      notification: { title, body },
      data,
      android: { priority: 'high' },
      apns: { payload: { aps: { contentAvailable: true } } },
    });
  } catch (error) {
    console.error(`Failed to send push to ${userId}:`, error);
  }
}
