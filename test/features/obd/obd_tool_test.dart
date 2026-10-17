import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hblite/features/obd/domain/obd_service.dart';
import 'package:hblite/features/obd/presentation/obd_screen.dart';

void main() {
  group('OBD2 Telemetry Tool - Domain Model Tests', () {
    test('ObdData initializes with correct default values and copyWith updates', () {
      final data = ObdData(rpm: '0', speed: '0', isConnected: false);

      expect(data.rpm, '0');
      expect(data.speed, '0');
      expect(data.isConnected, isFalse);

      final updated = data.copyWith(
        rpm: '2500',
        speed: '65',
        isConnected: true,
      );

      expect(updated.rpm, '2500');
      expect(updated.speed, '65');
      expect(updated.isConnected, isTrue);

      // Verify partial copyWith
      final partial = updated.copyWith(rpm: '3100');
      expect(partial.rpm, '3100');
      expect(partial.speed, '65');
      expect(partial.isConnected, isTrue);
    });
  });

  group('OBD2 Telemetry Tool - UI & Widget Tests', () {
    testWidgets('renders disconnected UI with MAC input and connect button', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: ObdScreen(),
          ),
        ),
      );

      expect(find.text('OBD2 Telemetry'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);
      expect(find.widgetWithText(ElevatedButton, 'Connect'), findsOneWidget);

      // Simulate typing a MAC address
      await tester.enterText(find.byType(TextField), '00:1D:A5:00:11:22');
      await tester.pump();
      expect(find.text('00:1D:A5:00:11:22'), findsOneWidget);
    });
  });
}
