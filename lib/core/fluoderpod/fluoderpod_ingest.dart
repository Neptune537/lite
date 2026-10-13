// lib/core/fluoderpod/fluoderpod_ingest.dart

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'entity_packet_encoder.dart';
import 'fluoderpod_bridge.dart';
// for taskToPacket if needed elsewhere

/// Public API for ingesting entity data into the Fluoderpod GPU pipeline.
///
/// This class abstracts the bridge interaction and can be used from any
/// feature module without pulling in the bridge directly.
class FluoderpodIngest {
  final FluoderpodBridge _bridge;

  FluoderpodIngest(this._bridge);

  /// Ingest a list of [EntityPacket]s. The packets are encoded into a flat
  /// byte buffer and handed to the native renderer via the bridge.
  /// Returns the number of packets successfully handed over.
  int ingestEntities(List<EntityPacket> packets) {
    if (!_bridge.isInitialized) return 0;
    final encoded = EntityPacketEncoder.encodeBatch(packets);
    _bridge.ingestBatch(encoded);
    return packets.length;
  }
}

/// Riverpod provider exposing a singleton instance of [FluoderpodIngest].
final fluoderpodIngestProvider = Provider<FluoderpodIngest>((ref) {
  final bridge = ref.watch(fluoderpodBridgeProvider);
  return FluoderpodIngest(bridge);
});

/// Convenience provider that directly accepts a list of [EntityPacket]s and
/// forwards them to the ingest API. Returns the count of ingested packets.
final fluoderpodIngestEntitiesProvider = Provider.autoDispose.family<int, List<EntityPacket>>((ref, packets) {
  final ingest = ref.read(fluoderpodIngestProvider);
  return ingest.ingestEntities(packets);
});
