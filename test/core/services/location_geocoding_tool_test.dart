import 'package:flutter_test/flutter_test.dart';
import 'package:hblite/core/services/location_geocoding_service.dart';

void main() {
  group('Location & Geocoding Tool Tests', () {
    test('AddressInfo.fallback constructs valid GPS sector descriptor', () {
      final info = AddressInfo.fallback(29.6549, -95.2285);

      expect(info.streetName, contains('GPS Sector'));
      expect(info.streetName, contains('29.655°'));
      expect(info.streetName, contains('-95.228°'));
      expect(info.neighborhood, 'Local Area');
      expect(info.city, 'Houston');
      expect(info.zipCode, '78701');
    });

    test('AddressInfo standard constructor preserves all fields', () {
      const info = AddressInfo(
        streetName: '100 Main St',
        neighborhood: 'Downtown',
        city: 'Austin',
        zipCode: '78701',
      );

      expect(info.streetName, '100 Main St');
      expect(info.neighborhood, 'Downtown');
      expect(info.city, 'Austin');
      expect(info.zipCode, '78701');
    });

    test('LocationGeocodingService maintains singleton instance', () {
      final instance1 = LocationGeocodingService.instance;
      final instance2 = LocationGeocodingService.instance;

      expect(identical(instance1, instance2), isTrue);
    });
  });
}
