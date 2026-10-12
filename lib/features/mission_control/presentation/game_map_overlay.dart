import 'dart:async';
import 'package:flutter/material.dart';
import 'package:geolocator/geolocator.dart';
import 'package:fluoderpod/fluoderpod.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../app/theme.dart';
import '../../../core/services/location_geocoding_service.dart';
import '../../comms/presentation/comms_overlay.dart';
import '../../teams/presentation/declare_war_dialog.dart';

class GameMapOverlay extends ConsumerStatefulWidget {
  const GameMapOverlay({super.key});

  @override
  ConsumerState<GameMapOverlay> createState() => _GameMapOverlayState();
}

class _GameMapOverlayState extends ConsumerState<GameMapOverlay> {
  String currentMode = 'CYBERPUNK RADAR';
  final List<String> modes = ['CYBERPUNK RADAR', 'TACTICAL GRID', 'SATELLITE 3D', 'STREET VIEW'];
  Position? _currentPosition;
  StreamSubscription<Position>? _positionStreamSub;

  String _realStreetName = 'Detecting GPS Street...';
  String _realNeighborhood = 'Local Sector';

  // ATAK-Grade Mapping Toggles
  bool _enable3dDemTerrain = true;
  bool _enableAtmosphericRayleighGlow = true;
  bool _enableSonarGossipRelay = true;
  bool _enableBioluminescencePoh = true;

  @override
  void initState() {
    super.initState();
    _startLocationTracking();
  }

  @override
  void dispose() {
    _positionStreamSub?.cancel();
    super.dispose();
  }

  void _startLocationTracking() {
    _positionStreamSub = Geolocator.getPositionStream(
      locationSettings: const LocationSettings(
        accuracy: LocationAccuracy.high,
        distanceFilter: 2,
      ),
    ).listen((position) {
      if (mounted) {
        setState(() => _currentPosition = position);
        _updateReverseGeocode(position.latitude, position.longitude);
      }
    });
  }

  Future<void> _updateReverseGeocode(double lat, double lng) async {
    final info = await LocationGeocodingService.instance.reverseGeocode(lat, lng);
    if (mounted) {
      setState(() {
        _realStreetName = info.streetName;
        _realNeighborhood = info.neighborhood;
      });
    }
  }

  void _switchMode(String mode) {
    if (mode == 'STREET VIEW') {
      final lat = _currentPosition?.latitude ?? 30.2672;
      final lng = _currentPosition?.longitude ?? -97.7431;
      _launchStreetView(lat, lng);
    } else {
      setState(() {
        currentMode = mode;
      });
    }
  }

  Future<void> _launchStreetView(double lat, double lng) async {
    final url = Uri.parse('https://www.google.com/maps/@?api=1&map_action=pano&viewpoint=$lat,$lng');
    if (await canLaunchUrl(url)) {
      await launchUrl(url, mode: LaunchMode.externalApplication);
    } else {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Opening Street View at $lat, $lng...')),
        );
      }
    }
  }

  Future<void> _launchNavigation(double lat, double lng) async {
    final url = Uri.parse('https://www.google.com/maps/dir/?api=1&destination=$lat,$lng');
    if (await canLaunchUrl(url)) {
      await launchUrl(url, mode: LaunchMode.externalApplication);
    } else {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Opening Google Maps route to $lat, $lng...')),
        );
      }
    }
  }

  void _onNodeSelectedOnGlobe(CyanNode node) {
    if (node.id.startsWith('person-')) {
      _showPersonCommsSheet(node.name, 'CALLSIGN-${node.id.toUpperCase().replaceAll('PERSON-', '')}', '462.5625 MHz', node.latitude, node.longitude);
    } else if (node.id.startsWith('sat-')) {
      _showSatelliteTelemetrySheet(node.name, node.latencyMs);
    } else {
      _showFactionWarSheet(node.name, node.id, node.latitude, node.longitude);
    }
  }

  void _showFlightSheet(EntityPacket flight) {
    final label = flight.callsign ?? flight.id;
    final isSatellite = flight.z > 30000 || flight.id.contains('SAT');

    showModalBottomSheet(
      context: context,
      backgroundColor: HBColors.neutral,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(HBRadius.xl)),
        side: BorderSide(color: HBColors.primary, width: 1.5),
      ),
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(HBSpacing.lg),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    isSatellite ? Icons.satellite_alt : Icons.flight_takeoff,
                    color: isSatellite ? Colors.amberAccent : HBColors.primary,
                    size: 28,
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      label,
                      style: const TextStyle(
                        color: Colors.white,
                        fontWeight: FontWeight.bold,
                        fontSize: 16,
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              Text(
                'Altitude: ${flight.z.toInt()}m • Speed: Mach 1.2 • Vector: En Route',
                style: const TextStyle(
                  color: Colors.white70,
                  fontSize: 12,
                  fontFamily: 'monospace',
                ),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      style: OutlinedButton.styleFrom(
                        foregroundColor: HBColors.primaryLight,
                        side: const BorderSide(color: HBColors.primaryLight),
                      ),
                      icon: const Icon(Icons.streetview, size: 16),
                      label: const Text('360° Ground View'),
                      onPressed: () {
                        Navigator.pop(context);
                        _launchStreetView(flight.y, flight.x);
                      },
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: ElevatedButton.icon(
                      style: ElevatedButton.styleFrom(
                        backgroundColor: HBColors.primary,
                      ),
                      icon: const Icon(Icons.cell_tower, size: 16),
                      label: const Text('Radio Link'),
                      onPressed: () {
                        Navigator.pop(context);
                        showModalBottomSheet(
                          context: context,
                          backgroundColor: Colors.transparent,
                          isScrollControlled: true,
                          builder: (context) => Padding(
                            padding: EdgeInsets.only(
                              top: MediaQuery.of(context).size.height * 0.3,
                            ),
                            child: const CommsOverlay(),
                          ),
                        );
                      },
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }

  void _showPersonCommsSheet(String name, String callsign, String frequency, double lat, double lng) {
    showModalBottomSheet(
      context: context,
      backgroundColor: HBColors.neutral,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(HBRadius.xl)),
        side: BorderSide(color: HBColors.primary, width: 1.5),
      ),
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(HBSpacing.lg),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const CircleAvatar(
                    backgroundColor: HBColors.primary,
                    child: Icon(Icons.person, color: Colors.white),
                  ),
                  const SizedBox(width: 12),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(name, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 16)),
                      Text('Callsign: $callsign • Radio: $frequency', style: const TextStyle(color: HBColors.primaryLight, fontSize: 12, fontFamily: 'monospace')),
                      Text('Location: $_realStreetName', style: const TextStyle(color: HBColors.secondary, fontSize: 11, fontWeight: FontWeight.bold)),
                    ],
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      style: OutlinedButton.styleFrom(
                        foregroundColor: HBColors.primaryLight,
                        side: const BorderSide(color: HBColors.primaryLight),
                      ),
                      icon: const Icon(Icons.streetview, size: 16),
                      label: const Text('360° Street View'),
                      onPressed: () {
                        Navigator.pop(context);
                        _launchStreetView(lat, lng);
                      },
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: OutlinedButton.icon(
                      style: OutlinedButton.styleFrom(
                        foregroundColor: HBColors.secondary,
                        side: const BorderSide(color: HBColors.secondary),
                      ),
                      icon: const Icon(Icons.navigation, size: 16),
                      label: const Text('Navigate (Maps)'),
                      onPressed: () {
                        Navigator.pop(context);
                        _launchNavigation(lat, lng);
                      },
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: HBColors.primary,
                    padding: const EdgeInsets.symmetric(vertical: 12),
                  ),
                  icon: const Icon(Icons.cell_tower),
                  label: const Text('📡 OPEN COMMS WITH OPERATOR'),
                  onPressed: () {
                    Navigator.pop(context);
                    showModalBottomSheet(
                      context: context,
                      backgroundColor: Colors.transparent,
                      isScrollControlled: true,
                      builder: (context) => Padding(
                        padding: EdgeInsets.only(
                          top: MediaQuery.of(context).size.height * 0.3,
                        ),
                        child: const CommsOverlay(),
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  void _showFactionWarSheet(String factionName, String factionId, double lat, double lng) {
    showModalBottomSheet(
      context: context,
      backgroundColor: HBColors.neutral,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(HBRadius.xl)),
        side: BorderSide(color: HBColors.error, width: 1.5),
      ),
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(HBSpacing.lg),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.shield, color: HBColors.error, size: 28),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      'RIVAL FACTION: ${factionName.toUpperCase()}',
                      style: const TextStyle(color: HBColors.error, fontWeight: FontWeight.bold, fontSize: 15, letterSpacing: 1.1),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text('Location: $_realStreetName, $_realNeighborhood', style: const TextStyle(color: HBColors.primaryLight, fontSize: 12, fontWeight: FontWeight.bold)),
              Text('Outpost GPS: ${lat.toStringAsFixed(4)}°, ${lng.toStringAsFixed(4)}°', style: const TextStyle(color: Colors.white70, fontSize: 11, fontFamily: 'monospace')),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      style: OutlinedButton.styleFrom(
                        foregroundColor: HBColors.primaryLight,
                        side: const BorderSide(color: HBColors.primaryLight),
                      ),
                      icon: const Icon(Icons.streetview, size: 16),
                      label: const Text('360° Street View'),
                      onPressed: () {
                        Navigator.pop(context);
                        _launchStreetView(lat, lng);
                      },
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: OutlinedButton.icon(
                      style: OutlinedButton.styleFrom(
                        foregroundColor: HBColors.secondary,
                        side: const BorderSide(color: HBColors.secondary),
                      ),
                      icon: const Icon(Icons.navigation, size: 16),
                      label: const Text('Route (Maps)'),
                      onPressed: () {
                        Navigator.pop(context);
                        _launchNavigation(lat, lng);
                      },
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: HBColors.error,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(vertical: 12),
                  ),
                  icon: const Icon(Icons.military_tech),
                  label: const Text('⚔️ INITIATE WAR FROM GLOBE'),
                  onPressed: () {
                    Navigator.pop(context);
                    showDialog(
                      context: context,
                      builder: (context) => DeclareWarDialog(
                        myTeamName: 'Water Clan',
                        enemyTeamName: factionName,
                        enemyTeamId: factionId,
                        onDeclare: (message) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text('War declared against $factionName! Message: $message'),
                              backgroundColor: Colors.redAccent,
                            ),
                          );
                        },
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  void _showSatelliteTelemetrySheet(String satName, int latency) {
    showModalBottomSheet(
      context: context,
      backgroundColor: HBColors.neutral,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(HBRadius.xl)),
        side: BorderSide(color: HBColors.primary, width: 1.5),
      ),
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(HBSpacing.lg),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.satellite_alt, color: HBColors.primary, size: 28),
                  const SizedBox(width: 10),
                  Text(satName, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 15)),
                ],
              ),
              const SizedBox(height: 12),
              Text('Orbital Latency: ${latency}ms • Downlink: 10.5 Gbps Laser Crosslink', style: const TextStyle(color: Colors.white70, fontSize: 12, fontFamily: 'monospace')),
              const SizedBox(height: 8),
              const Text('Payload: Multispectral Optical + SAR Radar Transponder.', style: TextStyle(color: Colors.white54, fontSize: 12)),
            ],
          ),
        );
      },
    );
  }

  void _showAtakConfigDialog() {
    showDialog(
      context: context,
      builder: (context) {
        return StatefulBuilder(
          builder: (context, setStateDialog) {
            return AlertDialog(
              backgroundColor: HBColors.neutral,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(HBRadius.lg),
                side: const BorderSide(color: HBColors.primary, width: 1.5),
              ),
              title: const Row(
                children: [
                  Icon(Icons.layers, color: HBColors.primary, size: 24),
                  SizedBox(width: 8),
                  Text('ATAK-GRADE 3D MAPPING CONFIG', style: TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold, letterSpacing: 1.1)),
                ],
              ),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  SwitchListTile(
                    activeThumbColor: HBColors.primary,
                    title: const Text('3D DEM Terrain Elevation', style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
                    subtitle: const Text('Digital Elevation Model heightmaps', style: TextStyle(color: Colors.white54, fontSize: 10)),
                    value: _enable3dDemTerrain,
                    onChanged: (val) {
                      setStateDialog(() => _enable3dDemTerrain = val);
                      setState(() => _enable3dDemTerrain = val);
                    },
                  ),
                  SwitchListTile(
                    activeThumbColor: HBColors.primary,
                    title: const Text('Atmospheric Rayleigh Scattering', style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
                    subtitle: const Text('Rayleigh/Mie planetary horizon glow', style: TextStyle(color: Colors.white54, fontSize: 10)),
                    value: _enableAtmosphericRayleighGlow,
                    onChanged: (val) {
                      setStateDialog(() => _enableAtmosphericRayleighGlow = val);
                      setState(() => _enableAtmosphericRayleighGlow = val);
                    },
                  ),
                  SwitchListTile(
                    activeThumbColor: HBColors.primary,
                    title: const Text('Sonar Relay Gossip Sync', style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
                    subtitle: const Text('Live WebSocket mesh packet sync', style: TextStyle(color: Colors.white54, fontSize: 10)),
                    value: _enableSonarGossipRelay,
                    onChanged: (val) {
                      setStateDialog(() => _enableSonarGossipRelay = val);
                      setState(() => _enableSonarGossipRelay = val);
                    },
                  ),
                  SwitchListTile(
                    activeThumbColor: HBColors.primary,
                    title: const Text('Bioluminescence PoH Telemetry', style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
                    subtitle: const Text('Proof of Health CPU/VRAM hardware stream', style: TextStyle(color: Colors.white54, fontSize: 10)),
                    value: _enableBioluminescencePoh,
                    onChanged: (val) {
                      setStateDialog(() => _enableBioluminescencePoh = val);
                      setState(() => _enableBioluminescencePoh = val);
                    },
                  ),
                ],
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('CLOSE'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final userLat = _currentPosition?.latitude ?? 30.2672;
    final userLng = _currentPosition?.longitude ?? -97.7431;

    // Rich Fluonder Globe Node Roster centered over live GPS camera view
    final cyanNodes = [
      // Player Base (Centered on User GPS)
      CyanNode(id: 'node-water-hq', name: 'Water Clan HQ (Command Post)', latitude: userLat, longitude: userLng, latencyMs: 12, isHomebase: true),

      // Regional Tactical Factions & Outposts
      CyanNode(id: 'node-fire-depot', name: 'Fire Tribe Depot', latitude: userLat + 6.5, longitude: userLng - 10.0, latencyMs: 18),
      CyanNode(id: 'node-earth-citadel', name: 'Earth Guild Citadel', latitude: userLat - 8.2, longitude: userLng + 12.4, latencyMs: 22),
      CyanNode(id: 'node-wind-spire', name: 'Wind Order Sky Spire', latitude: userLat + 14.0, longitude: userLng + 18.0, latencyMs: 15),

      // Global Strategic Backbone Hubs
      const CyanNode(id: 'node-frankfurt', name: 'Frankfurt Central Core', latitude: 50.11, longitude: 8.68, latencyMs: 28),
      const CyanNode(id: 'node-tokyo', name: 'Tokyo Quantum Gateway', latitude: 35.68, longitude: 139.69, latencyMs: 44),
      const CyanNode(id: 'node-singapore', name: 'Singapore Cyber Bastion', latitude: 1.35, longitude: 103.82, latencyMs: 38),
      const CyanNode(id: 'node-saopaulo', name: 'São Paulo Southern Edge', latitude: -23.55, longitude: -46.63, latencyMs: 65),

      // Orbital Satellites
      CyanNode(id: 'sat-nexus-1', name: 'NEXUS-RECON-1 (Orbital)', latitude: userLat + 28.0, longitude: userLng - 35.0, latencyMs: 45),
      CyanNode(id: 'sat-relay-alpha', name: 'FLUORIDIAN-RELAY (P2P Mesh)', latitude: userLat - 32.0, longitude: userLng + 45.0, latencyMs: 52),
    ];

    // Live Flights / Aerial Drones / Plane Trackers
    final flightPackets = [
      EntityPacket(id: 'FLIGHT-AZ-402', x: userLng + 12.0, y: userLat - 8.0, z: 12000, type: FluoderpodEntityType.player, callsign: 'FLIGHT-AZ-402 (Aerial Recon)'),
      EntityPacket(id: 'FLIGHT-CYAN-88', x: userLng - 22.0, y: userLat + 14.0, z: 24500, type: FluoderpodEntityType.player, callsign: 'FLIGHT-CYAN-88 (Suborbital Transport)'),
      EntityPacket(id: 'SAT-STARLINK-88', x: userLng + 45.0, y: userLat + 30.0, z: 550000, type: FluoderpodEntityType.player, callsign: 'STARLINK-FLR-88 (Broadband)'),
    ];

    return Stack(
      children: [
        // High-Performance Interactive Fluonder Globe via Fluoderpod
        Positioned.fill(
          child: CyanDashboard(
            globeWidget: CyanGlobeOverview(
              nodes: cyanNodes,
              flights: flightPackets,
              mainStreetName: _realStreetName,
              crossStreetName: _realNeighborhood,
              onNodeSelected: _onNodeSelectedOnGlobe,
              onFlightSelected: _showFlightSheet,
            ),
            immersiveAppWidget: const CyanImmersiveView(),
          ),
        ),

        // Live GPS & ATAK Config Header Overlay
        Positioned(
          top: 10,
          left: 10,
          right: 10,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: Colors.black.withValues(alpha: 0.8),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: HBColors.primary.withValues(alpha: 0.6)),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    const Icon(Icons.pin_drop, color: HBColors.secondary, size: 14),
                    const SizedBox(width: 6),
                    Text(
                      'STREET: $_realStreetName',
                      style: const TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace', fontWeight: FontWeight.bold),
                    ),
                  ],
                ),
                GestureDetector(
                  onTap: _showAtakConfigDialog,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: HBColors.primary.withValues(alpha: 0.2),
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(color: HBColors.primary),
                    ),
                    child: const Row(
                      children: [
                        Icon(Icons.tune, color: HBColors.primary, size: 12),
                        SizedBox(width: 4),
                        Text('ATAK CONFIG', style: TextStyle(color: HBColors.primary, fontSize: 9, fontWeight: FontWeight.bold)),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),

        // Mode Switcher Overlay Bar
        Positioned(
          bottom: 16,
          left: 16,
          right: 16,
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: modes.map((m) {
              final isSelected = currentMode == m;
              return Expanded(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4),
                  child: ElevatedButton(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: isSelected ? HBColors.primary : HBColors.neutral,
                      foregroundColor: isSelected ? Colors.black : Colors.white,
                      side: const BorderSide(color: HBColors.primary),
                      padding: const EdgeInsets.symmetric(vertical: 8),
                    ),
                    onPressed: () => _switchMode(m),
                    child: Text(
                      m,
                      style: TextStyle(
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        color: isSelected ? Colors.black : Colors.white,
                      ),
                      textAlign: TextAlign.center,
                    ),
                  ),
                ),
              );
            }).toList(),
          ),
        ),
      ],
    );
  }
}
