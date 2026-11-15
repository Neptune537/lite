import { onObjectFinalized } from 'firebase-functions/v2/storage';
import { getFirestore, FieldValue } from 'firebase-admin/firestore';

const db = getFirestore();

// ── onCredentialUploaded: Storage trigger → create Firestore metadata ─────────
//
// Fires when any file is uploaded under /credentials/{userId}/...
// Creates a Firestore record in /credentials/{docId} with status: 'uploaded'.
export const onCredentialUploaded = onObjectFinalized(async (event) => {
  const { name: storagePath, contentType, size } = event.data;
  if (!storagePath) return;

  // Only handle files under /credentials/
  const match = storagePath.match(/^credentials\/([^/]+)\/([^/]+)\/(.+)$/);
  if (!match) return;

  const [, userId, docType] = match;

  await db.collection('credentials').add({
    ownerId: userId,
    storagePath,
    docType,
    label: docType,                    // User can rename via profile page
    status: 'uploaded',
    mimeType: contentType ?? 'application/octet-stream',
    fileSizeBytes: Number(size),
    uploadedAt: FieldValue.serverTimestamp(),
  });

  console.log(`Credential uploaded: ${storagePath} for user ${userId}`);
});
