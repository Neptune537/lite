import 'package:flutter_test/flutter_test.dart';
import 'package:hblite/core/blockchain/models/network_event.dart';

void main() {
  group('Fluoridian & Nexus Gossip Relay Tool Tests', () {
    test('NetworkEvent creates deterministic hash and serializes to/from JSON correctly', () {
      final now = DateTime.utc(2026, 10, 15, 12);
      final payload = {'taskId': 'task-100', 'bidAmount': 45.0, 'currency': 'NEXUS'};

      final event = NetworkEvent.create(
        did: 'did:nexus:agent_alpha',
        eventType: NetworkEventType.marketplaceBid,
        payload: payload,
        signature: 'ed25519_sig_test_123',
        createdAt: now,
      );

      expect(event.did, 'did:nexus:agent_alpha');
      expect(event.eventType, NetworkEventType.marketplaceBid);
      expect(event.eventType.toWireType(), 'MARKETPLACE_BID');
      expect(event.payload['taskId'], 'task-100');
      expect(event.payload['bidAmount'], 45.0);
      expect(event.signature, 'ed25519_sig_test_123');
      expect(event.id.length, 64); // SHA-256 length

      // Test JSON round-trip
      final json = event.toJson();
      expect(json['id'], event.id);
      expect(json['eventType'], 'MARKETPLACE_BID');

      final roundTrip = NetworkEvent.fromJson(json);
      expect(roundTrip.id, event.id);
      expect(roundTrip.did, event.did);
      expect(roundTrip.eventType, NetworkEventType.marketplaceBid);
      expect(roundTrip.payload['taskId'], 'task-100');
      expect(roundTrip.signature, event.signature);
    });

    test('NetworkEventType wire conversion supports all enum variants', () {
      final types = [
        NetworkEventType.taskCreated,
        NetworkEventType.marketplaceBid,
        NetworkEventType.bidAccepted,
        NetworkEventType.bidRejected,
        NetworkEventType.taskVerified,
        NetworkEventType.taskDisputed,
        NetworkEventType.warDeclared,
        NetworkEventType.pohHeartbeat,
      ];

      for (final type in types) {
        final wire = type.toWireType();
        final reconstructed = NetworkEventTypeX.fromWireType(wire);
        expect(reconstructed, type);
      }

      // Default fallback
      expect(NetworkEventTypeX.fromWireType('UNKNOWN_EVENT'), NetworkEventType.pohHeartbeat);
    });
  });
}
