import 'package:flutter/material.dart';
import 'package:fluoderpod/fluoderpod.dart';
import '../../../app/theme.dart';
import '../../comms/presentation/cyan_radio_widget.dart';

class ToolsPage extends StatelessWidget {
  const ToolsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Comms & Tools'),
        actions: [
          IconButton(
            icon: const Icon(Icons.hub_outlined, color: Colors.cyanAccent),
            tooltip: 'Cyan Mesh Telemetry',
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Cyan Mesh Radio & Fluonder Orbital Comms Online'),
                  duration: Duration(seconds: 2),
                ),
              );
            },
          ),
        ],
      ),
      body: Column(
        children: [
          // Fluonder 3D Planetary Vector Globe
          const Expanded(
            flex: 3,
            child: FluonderGlobe(),
          ),

          // Comms Telemetry Status & Cyan Network Radio Hub
          Expanded(
            flex: 4,
            child: Container(
              color: const Color(0xFF070B14),
              child: SingleChildScrollView(
                padding: const EdgeInsets.symmetric(
                  horizontal: HBSpacing.md,
                  vertical: HBSpacing.sm,
                ),
                child: Column(
                  children: [
                    // Global Comms Status Bar
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: HBSpacing.md,
                        vertical: HBSpacing.sm,
                      ),
                      decoration: BoxDecoration(
                        color: HBColors.neutral,
                        borderRadius: BorderRadius.circular(HBRadius.md),
                        border: Border.all(
                          color: HBColors.primary.withValues(alpha: 0.3),
                        ),
                      ),
                      child: const Row(
                        children: [
                          Icon(Icons.satellite_alt, size: 28, color: HBColors.primary),
                          SizedBox(width: HBSpacing.md),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  'Global Comms Link',
                                  style: TextStyle(
                                    color: Colors.white,
                                    fontSize: 14,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                                SizedBox(height: 2),
                                Text(
                                  'Walkie Talkie & GPS mesh online',
                                  style: TextStyle(
                                    color: Colors.white60,
                                    fontSize: 11,
                                  ),
                                ),
                              ],
                            ),
                          ),
                          Icon(Icons.wifi, color: Colors.greenAccent, size: 18),
                        ],
                      ),
                    ),
                    const SizedBox(height: HBSpacing.sm),

                    // Cyan Network Radio Transceiver
                    const CyanRadioWidget(),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
