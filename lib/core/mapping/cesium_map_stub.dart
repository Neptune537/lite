import 'package:flutter/material.dart';

Widget buildCesiumMapView(double lat, double lng, {String? title}) {
  return Container(
    color: const Color(0xFF070C15),
    alignment: Alignment.center,
    padding: const EdgeInsets.all(16),
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(Icons.public, color: Color(0xFF00E5FF), size: 36),
        const SizedBox(height: 12),
        Text(
          title ?? 'CESIUM 3D TACTICAL NAV',
          style: const TextStyle(
            color: Colors.white,
            fontWeight: FontWeight.bold,
            fontSize: 14,
            letterSpacing: 1.1,
          ),
        ),
        const SizedBox(height: 6),
        Text(
          'WGS84 ECEF: Lat ${lat.toStringAsFixed(4)}, Lon ${lng.toStringAsFixed(4)}',
          style: const TextStyle(
            color: Color(0xFF00E5FF),
            fontSize: 11,
            fontFamily: 'monospace',
          ),
        ),
        const SizedBox(height: 12),
        const Text(
          'Cesium 3D Globe WebGL/WebGPU overlay active.\n(Native Android Vulkan / Web Viewport)',
          textAlign: TextAlign.center,
          style: TextStyle(color: Colors.white54, fontSize: 11),
        ),
      ],
    ),
  );
}
