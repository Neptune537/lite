import 'package:flutter_test/flutter_test.dart';
import 'package:hblite/features/mission_control/domain/weather_data.dart';
import 'package:hblite/features/mission_control/data/weather_service.dart';

void main() {
  group('Weather Tool - Domain & Service Tests', () {
    test('WeatherData constructs with correct telemetry attributes', () {
      const data = WeatherData(
        temperature: 78.5,
        description: 'Clear Sky',
        icon: '01d',
        humidity: 55,
        windSpeed: 6.2,
        cityName: 'Austin',
      );

      expect(data.temperature, 78.5);
      expect(data.description, 'Clear Sky');
      expect(data.icon, '01d');
      expect(data.humidity, 55);
      expect(data.windSpeed, 6.2);
      expect(data.cityName, 'Austin');
    });

    test('WeatherService instance creates cleanly and handles network timeout gracefully', () async {
      final service = WeatherService();
      // Using invalid / simulated coordinates without internet shouldn't throw unhandled errors
      final result = await service.fetchWeather(-999.0, -999.0);
      // Service catches errors and returns null on failure
      expect(result, isNull);
    });
  });
}
