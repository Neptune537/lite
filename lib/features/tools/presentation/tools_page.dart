import 'package:flutter/material.dart';
import 'package:fluoderpod/fluoderpod.dart';
import '../../../app/theme.dart';

class ToolsPage extends StatelessWidget {
  const ToolsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Comms & Tools')),
      body: const Column(
        children: [
          Expanded(
            flex: 2,
            child: CyanGlobeOverview(),
          ),
          Expanded(
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.satellite_alt, size: 48, color: HBColors.primary),
                  SizedBox(height: HBSpacing.md),
                  Text(
                    'Global Comms Link',
                    style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
                  ),
                  SizedBox(height: HBSpacing.sm),
                  Text(
                    'Walkie Talkie & GPS mesh online',
                    style: TextStyle(color: Colors.grey),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
