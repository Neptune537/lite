import 'package:flutter/material.dart';

import '../../../app/theme.dart';

class CommsConfigPage extends StatefulWidget {
  const CommsConfigPage({super.key});

  @override
  State<CommsConfigPage> createState() => _CommsConfigPageState();
}

class _CommsConfigPageState extends State<CommsConfigPage> {
  bool _globalCommsEnabled = true;
  bool _teamCommsEnabled = true;
  bool _directCommsEnabled = true;
  bool _locationSharingEnabled = true;
  double _radarRange = 50.0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text(
          'COMMS CONFIGURATION',
          style: TextStyle(letterSpacing: 2),
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.all(HBSpacing.lg),
        children: [
          _buildSectionHeader('CHANNELS'),
          _buildSwitchTile(
            title: 'Global Broadcasts',
            subtitle: 'Receive server-wide alerts and Nexus updates.',
            value: _globalCommsEnabled,
            onChanged: (val) => setState(() => _globalCommsEnabled = val),
          ),
          _buildSwitchTile(
            title: 'Team Tactical',
            subtitle: 'Encrypted channel for current squad members.',
            value: _teamCommsEnabled,
            onChanged: (val) => setState(() => _teamCommsEnabled = val),
          ),
          _buildSwitchTile(
            title: 'Direct Messages',
            subtitle: 'Allow P2P encrypted messages from contacts.',
            value: _directCommsEnabled,
            onChanged: (val) => setState(() => _directCommsEnabled = val),
          ),

          const SizedBox(height: HBSpacing.xl),
          _buildSectionHeader('RADAR & TRACKING'),
          _buildSwitchTile(
            title: 'Share Location',
            subtitle: 'Broadcast GPS coordinates to friends and team members.',
            value: _locationSharingEnabled,
            onChanged: (val) => setState(() => _locationSharingEnabled = val),
          ),

          const SizedBox(height: HBSpacing.md),
          const Text(
            'Radar Sweep Range (km)',
            style: TextStyle(
              color: HBColors.primary,
              fontWeight: FontWeight.bold,
            ),
          ),
          Slider(
            value: _radarRange,
            min: 5,
            max: 200,
            divisions: 39,
            activeColor: HBColors.secondary,
            inactiveColor: HBColors.primary.withValues(alpha: 0.3),
            label: '${_radarRange.round()} km',
            onChanged: (val) => setState(() => _radarRange = val),
          ),

          const SizedBox(height: HBSpacing.xl),
          _buildSectionHeader('SYSTEM'),
          ListTile(
            leading: const Icon(Icons.delete_forever, color: Colors.redAccent),
            title: const Text(
              'Clear Local Comms Cache',
              style: TextStyle(color: Colors.redAccent),
            ),
            subtitle: const Text(
              'Purge all SQLite message data from this device.',
              style: TextStyle(color: Colors.white70),
            ),
            onTap: () {
              // TODO: Implement SQLite clear
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Comms cache cleared.')),
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: HBSpacing.md),
      child: Text(
        title,
        style: const TextStyle(
          color: HBColors.secondary,
          fontSize: 18,
          fontWeight: FontWeight.bold,
          letterSpacing: 1.5,
        ),
      ),
    );
  }

  Widget _buildSwitchTile({
    required String title,
    required String subtitle,
    required bool value,
    required ValueChanged<bool> onChanged,
  }) {
    return SwitchListTile(
      title: Text(
        title,
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.bold,
        ),
      ),
      subtitle: Text(subtitle, style: const TextStyle(color: Colors.white70)),
      value: value,
      onChanged: onChanged,
      activeThumbColor: HBColors.secondary,
      activeTrackColor: HBColors.secondary.withValues(alpha: 0.3),
      inactiveThumbColor: HBColors.primary,
      inactiveTrackColor: HBColors.neutral,
      contentPadding: EdgeInsets.zero,
    );
  }
}
