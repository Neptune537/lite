import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hblite/features/tools/presentation/tools_page.dart';
import 'package:hblite/features/comms/presentation/cyan_radio_widget.dart';
import 'package:fluoderpod/fluoderpod.dart';

void main() {
  group('ToolsPage & Cyan Network Radio Tests', () {
    testWidgets('renders Comms & Tools header, FluonderGlobe, and status', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: ToolsPage(),
          ),
        ),
      );

      // Verify AppBar title
      expect(find.text('Comms & Tools'), findsOneWidget);

      // Verify the Fluonder 3D Planetary Globe tool is present
      expect(find.byType(FluonderGlobe), findsOneWidget);

      // Verify the Comms tool status indicators
      expect(find.byIcon(Icons.satellite_alt), findsOneWidget);
      expect(find.text('Global Comms Link'), findsOneWidget);
      expect(find.text('Walkie Talkie & GPS mesh online'), findsOneWidget);

      // Verify Cyan Network Radio is integrated
      expect(find.byType(CyanRadioWidget), findsOneWidget);
      expect(find.text('CYAN NETWORK RADIO'), findsOneWidget);
    });

    testWidgets('CyanRadioWidget displays active channel and allows cycling channels', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: CyanRadioWidget(),
            ),
          ),
        ),
      );

      // Initially on Channel 1: TAC-SQUAD
      expect(find.text('TAC-SQUAD'), findsOneWidget);
      expect(find.text('462.5625 MHz'), findsOneWidget);
      expect(find.text('Squad Comms & Tactical Units'), findsOneWidget);

      // Cycle to Channel 2: GUILD-HQ
      await tester.tap(find.byTooltip('Next Channel'));
      await tester.pumpAndSettle();

      expect(find.text('GUILD-HQ'), findsOneWidget);
      expect(find.text('462.5875 MHz'), findsOneWidget);
      expect(find.text('Faction / Clan Command Post'), findsOneWidget);

      // Cycle to Channel 3: NEXUS-MESH
      await tester.tap(find.byTooltip('Next Channel'));
      await tester.pumpAndSettle();

      expect(find.text('NEXUS-MESH'), findsOneWidget);
      expect(find.text('467.5625 MHz'), findsOneWidget);

      // Cycle backward to GUILD-HQ
      await tester.tap(find.byTooltip('Previous Channel'));
      await tester.pumpAndSettle();

      expect(find.text('GUILD-HQ'), findsOneWidget);
    });

    testWidgets('CyanRadioWidget toggles squelch and power modes', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: CyanRadioWidget(),
            ),
          ),
        ),
      );

      // Verify initial SQL is ON
      expect(find.text('ON'), findsOneWidget);
      expect(find.text('5W HI'), findsOneWidget);

      // Toggle SQL
      await tester.tap(find.text('ON'));
      await tester.pumpAndSettle();
      expect(find.text('OFF'), findsOneWidget);

      // Toggle Power mode
      await tester.tap(find.text('5W HI'));
      await tester.pumpAndSettle();
      expect(find.text('0.5W LO'), findsOneWidget);
    });

    testWidgets('CyanRadioWidget PTT button activates transmission on press', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: CyanRadioWidget(),
            ),
          ),
        ),
      );

      // Verify idle state
      expect(find.text('HOLD TO TRANSMIT (CYAN RF MESH)'), findsOneWidget);

      // Perform PointerDown on the microphone icon
      final micFinder = find.byIcon(Icons.mic);
      expect(micFinder, findsOneWidget);

      final gesture = await tester.startGesture(tester.getCenter(micFinder));
      await tester.pump(const Duration(milliseconds: 100));

      // Verify transmitting state
      expect(
        find.text('TRANSMITTING TO CYAN MESH [462.5625 MHz]'),
        findsOneWidget,
      );

      // Release PTT
      await gesture.up();
      await tester.pumpAndSettle();

      expect(find.text('HOLD TO TRANSMIT (CYAN RF MESH)'), findsOneWidget);
    });
  });
}
