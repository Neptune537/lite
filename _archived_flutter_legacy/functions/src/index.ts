import { initializeApp } from 'firebase-admin/app';
initializeApp();

// ── Export all function modules ───────────────────────────────────────────────
export * from './bids';
export * from './tasks';
export * from './escrow';
export * from './notifications';
