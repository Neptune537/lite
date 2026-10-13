import 'package:flutter/material.dart';
import 'cesium_map_stub.dart' if (dart.library.html) 'cesium_map_web.dart' as platform_impl;

/// Universal Cesium 3D Mapping Viewport across Android and Web.
/// Directly replaces legacy Google Maps embeds and 2D tile renderers with
/// CesiumJS WGS84 ECEF 3D Globe, Photorealistic 3D Tiles, and turn-by-turn routing.
class CesiumMapView extends StatelessWidget {
  final double latitude;
  final double longitude;
  final String? title;

  const CesiumMapView({
    super.key,
    required this.latitude,
    required this.longitude,
    this.title,
  });

  @override
  Widget build(BuildContext context) {
    return platform_impl.buildCesiumMapView(
      latitude,
      longitude,
      title: title,
    );
  }
}
