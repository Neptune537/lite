"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sendPushAndSaveNotification = sendPushAndSaveNotification;
const firestore_1 = require("firebase-admin/firestore");
const messaging_1 = require("firebase-admin/messaging");
const db = (0, firestore_1.getFirestore)();
async function sendPushAndSaveNotification(userId, title, body, data, route) {
    // 1. Save to in-app notifications
    await db.collection('users').doc(userId).collection('notifications').add({
        userId,
        title,
        body,
        route: route ?? null,
        isRead: false,
        createdAt: firestore_1.FieldValue.serverTimestamp(),
        data,
    });
    // 2. Send FCM push
    const userSnap = await db.collection('users').doc(userId).get();
    const fcmToken = userSnap.data()?.fcmToken;
    if (!fcmToken)
        return;
    try {
        await (0, messaging_1.getMessaging)().send({
            token: fcmToken,
            notification: { title, body },
            data,
            android: { priority: 'high' },
            apns: { payload: { aps: { contentAvailable: true } } },
        });
    }
    catch (error) {
        console.error(`Failed to send push to ${userId}:`, error);
    }
}
//# sourceMappingURL=sendPush.js.map