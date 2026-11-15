"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.onCredentialUploaded = void 0;
const storage_1 = require("firebase-functions/v2/storage");
const firestore_1 = require("firebase-admin/firestore");
const db = (0, firestore_1.getFirestore)();
// ── onCredentialUploaded: Storage trigger → create Firestore metadata ─────────
//
// Fires when any file is uploaded under /credentials/{userId}/...
// Creates a Firestore record in /credentials/{docId} with status: 'uploaded'.
exports.onCredentialUploaded = (0, storage_1.onObjectFinalized)(async (event) => {
    const { name: storagePath, contentType, size } = event.data;
    if (!storagePath)
        return;
    // Only handle files under /credentials/
    const match = storagePath.match(/^credentials\/([^/]+)\/([^/]+)\/(.+)$/);
    if (!match)
        return;
    const [, userId, docType] = match;
    await db.collection('credentials').add({
        ownerId: userId,
        storagePath,
        docType,
        label: docType, // User can rename via profile page
        status: 'uploaded',
        mimeType: contentType ?? 'application/octet-stream',
        fileSizeBytes: Number(size),
        uploadedAt: firestore_1.FieldValue.serverTimestamp(),
    });
    console.log(`Credential uploaded: ${storagePath} for user ${userId}`);
});
//# sourceMappingURL=notifications.js.map