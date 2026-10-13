import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:fluoderpod/fluoderpod.dart';

import '../../../../core/fluoderpod/fluoderpod_bridge.dart';
import '../../../../core/fluoderpod/fluoderpod_view.dart';
import '../team_providers.dart';

enum ConstellationViewportMode {
  tactical3d,
  macroFluonder,
  splitTactical,
}

class ConstellationMapPage extends ConsumerStatefulWidget {
  const ConstellationMapPage({super.key});

  @override
  ConsumerState<ConstellationMapPage> createState() => _ConstellationMapPageState();
}

class _ConstellationMapPageState extends ConsumerState<ConstellationMapPage> {
  ConstellationViewportMode _currentMode = ConstellationViewportMode.tactical3d;

  Widget _buildFluoderpod3D(List<dynamic> teams) {
    return FluoderpodView(
      onInitialized: () {
        final bridge = ref.read(fluoderpodBridgeProvider);
        final bytes = Uint8List(teams.length * 64);
        bridge.ingestBatch(bytes);
      },
      onEntityTapped: (id) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Tapped entity: $id')),
        );
      },
    );
  }

  Widget _buildFluonderGlobe(List<dynamic> teams) {
    final cyanNodes = teams.asMap().entries.map((entry) {
      final idx = entry.key;
      final team = entry.value;
      return CyanNode(
        id: team.id,
        name: team.name,
        latitude: 30.2672 + (idx * 5.0),
        longitude: -97.7431 + (idx * 8.0),
        latencyMs: 12 + (idx * 3),
        isHomebase: idx == 0,
      );
    }).toList();

    return FluonderGlobe(
      nodes: cyanNodes,
      mainStreetName: 'Fluonder Guild Constellation',
      crossStreetName: '${teams.length} Factions Active',
      onNodeSelected: (node) {
        if (node is CyanNode) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Guild Selected: ${node.name}')),
          );
        }
      },
    );
  }

  Widget _buildViewport(List<dynamic> teams) {
    switch (_currentMode) {
      case ConstellationViewportMode.tactical3d:
        return _buildFluoderpod3D(teams);
      case ConstellationViewportMode.macroFluonder:
        return _buildFluonderGlobe(teams);
      case ConstellationViewportMode.splitTactical:
        return Column(
          children: [
            Expanded(
              flex: 5,
              child: _buildFluoderpod3D(teams),
            ),
            Container(height: 2, color: const Color(0xFF00E5FF)),
            Expanded(
              flex: 5,
              child: _buildFluonderGlobe(teams),
            ),
          ],
        );
    }
  }

  @override
  Widget build(BuildContext context) {
    final teamsAsync = ref.watch(teamsProvider);

    return Scaffold(
      backgroundColor: const Color(0xFF0A0E17),
      appBar: AppBar(
        title: const Text(
          'Guild Constellation',
          style: TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.bold),
        ),
        backgroundColor: const Color(0xFF131B2E),
        elevation: 0,
        actions: [
          Padding(
            padding: const EdgeInsets.only(right: 8.0),
            child: SegmentedButton<ConstellationViewportMode>(
              style: SegmentedButton.styleFrom(
                backgroundColor: const Color(0xFF0D1424),
                selectedBackgroundColor: const Color(0xFF00E5FF),
                selectedForegroundColor: Colors.black,
                foregroundColor: Colors.white70,
                textStyle: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold),
              ),
              segments: const [
                ButtonSegment(
                  value: ConstellationViewportMode.tactical3d,
                  label: Text('3D Engine'),
                  icon: Icon(Icons.view_in_ar, size: 14),
                ),
                ButtonSegment(
                  value: ConstellationViewportMode.macroFluonder,
                  label: Text('Fluonder Globe'),
                  icon: Icon(Icons.public, size: 14),
                ),
                ButtonSegment(
                  value: ConstellationViewportMode.splitTactical,
                  label: Text('Split'),
                  icon: Icon(Icons.splitscreen, size: 14),
                ),
              ],
              selected: {_currentMode},
              onSelectionChanged: (newSelection) {
                setState(() {
                  _currentMode = newSelection.first;
                });
              },
            ),
          ),
        ],
      ),
      body: teamsAsync.when(
        data: (teams) {
          return Stack(
            children: [
              // 1. Primary Viewport
              Positioned.fill(child: _buildViewport(teams)),

              // 2. Reticulum / SOCKS5 Mesh Active Banner
              Positioned(
                top: 12,
                left: 12,
                right: 12,
                child: Card(
                  color: const Color(0xFF1A233A).withValues(alpha: 0.92),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(8),
                    side: const BorderSide(color: Color(0xFF00E5FF), width: 0.8),
                  ),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    child: Row(
                      children: [
                        const Icon(Icons.hub, color: Color(0xFF00E5FF), size: 18),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'Fluoridian Mesh Active • Guilds: ${teams.length} • Fluonder Vector Engine Online',
                            style: const TextStyle(color: Colors.white, fontSize: 11, fontWeight: FontWeight.bold),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          );
        },
        loading: () => const Center(child: CircularProgressIndicator(color: Color(0xFF00E5FF))),
        error: (err, stack) => Center(child: Text('Error: $err', style: const TextStyle(color: Colors.white))),
      ),
    );
  }
}
