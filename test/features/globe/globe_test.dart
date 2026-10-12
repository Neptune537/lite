import 'package:flutter_test/flutter_test.dart';
import 'package:fluoderpod/fluoderpod.dart';

void main() {
  group('Fluonder Planetary Coordinates & Globe Tests', () {
    test('FluonderGeodetic to FluonderEcef converts equator point accurately', () {
      const geo = FluonderGeodetic(latitude: 0.0, longitude: 0.0, altitudeMeters: 0.0);
      final ecef = geo.toEcef();

      // At (0, 0), x should be approximately equatorial radius (6,378,137 m)
      expect(ecef.x, closeTo(Wgs84.a, 1.0));
      expect(ecef.y, closeTo(0.0, 1.0));
      expect(ecef.z, closeTo(0.0, 1.0));
    });

    test('FluonderEcef roundtrips back to FluonderGeodetic', () {
      const original = FluonderGeodetic(latitude: 37.7749, longitude: -122.4194, altitudeMeters: 100.0);
      final ecef = original.toEcef();
      final roundtrip = ecef.toGeodetic();

      expect(roundtrip.latitude, closeTo(original.latitude, 1e-4));
      expect(roundtrip.longitude, closeTo(original.longitude, 1e-4));
      expect(roundtrip.altitudeMeters, closeTo(original.altitudeMeters, 1.0));
    });

    test('FluonderAstronomy computes solar position', () {
      final now = DateTime.utc(2026, 6, 21, 12, 0, 0); // Summer solstice noon UTC
      final subsolar = FluonderAstronomy.calculateSubsolarPoint(now);

      // In late June, subsolar latitude is near +23.4 degrees (Tropic of Cancer)
      expect(subsolar.latitude, closeTo(23.44, 1.0));
      // At 12:00 UTC, subsolar longitude is near 0 degrees
      expect(subsolar.longitude, closeTo(0.0, 2.0));
    });

    test('CyanNode correctly exposes geodetic and ECEF coordinates', () {
      const node = CyanNode(
        id: 'test-node',
        name: 'Test Node',
        latitude: 40.7,
        longitude: -74.0,
        latencyMs: 15,
      );

      expect(node.geodetic.latitude, 40.7);
      expect(node.geodetic.longitude, -74.0);
      expect(node.ecef.magnitude, closeTo(Wgs84.a, 50000.0));
    });

    test('Natural Earth vector dataset is populated', () {
      final dataset = NaturalEarth110mData.dataset;
      expect(dataset.features.isNotEmpty, isTrue);
    });
  });
}
