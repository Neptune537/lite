import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:fluoderpod/fluoderpod.dart';

void main() {
  group('ATAK Configuration Tool Tests', () {
    test('AtakMapConfig initializes with expected defaults', () {
      const config = AtakMapConfig();

      expect(config.enablePhotorealisticRasterTiles, isTrue);
      expect(config.enable3dDemElevation, isTrue);
      expect(config.enableRayleighMieScattering, isTrue);
      expect(config.enableOfflineTacticalTileCache, isTrue);
      expect(config.enableSonarGossipRelay, isTrue);
      expect(config.enableBioluminescencePoh, isTrue);
    });

    test('AtakMapConfig supports custom toggle configurations', () {
      const config = AtakMapConfig(
        enable3dDemElevation: false,
        enableRayleighMieScattering: false,
        enableSonarGossipRelay: false,
        enableBioluminescencePoh: false,
      );

      expect(config.enable3dDemElevation, isFalse);
      expect(config.enableRayleighMieScattering, isFalse);
      expect(config.enableSonarGossipRelay, isFalse);
      expect(config.enableBioluminescencePoh, isFalse);
    });

    test('ecefToMapCenter converts FluonderEcef to geographic map coordinates correctly', () {
      // Coordinate at prime meridian on equator (lat=0, lon=0)
      const ecef = FluonderEcef(6378137.0, 0.0, 0.0);
      final coords = AtakMapConfig.ecefToMapCenter(ecef);

      expect(coords.length, 2);
      expect(coords[0], closeTo(0.0, 0.01)); // longitude
      expect(coords[1], closeTo(0.0, 0.01)); // latitude
    });
  });

  group('Tactical Mode & Style Tools Tests', () {
    test('CelestiaPunkMapStyle factory presets initialize distinctive mode styles', () {
      const radar = CelestiaPunkMapStyle.cyberpunkRadar();
      expect(radar.coastlineColor, const Color(0xFF00FFFF));
      expect(radar.enableNightPhosphor, isTrue);

      const grid = CelestiaPunkMapStyle.tacticalGrid();
      expect(grid.coastlineColor, const Color(0xFF00FF66));
      expect(grid.gridWidth, 1.0);
      expect(grid.enableNightPhosphor, isFalse);

      const sat = CelestiaPunkMapStyle.satellite3d();
      expect(sat.coastlineColor, const Color(0xFF80D8FF));
      expect(sat.gridWidth, 0.5);
      expect(sat.enableNightPhosphor, isTrue);

      const recon = CelestiaPunkMapStyle.tacticalRecon();
      expect(recon.coastlineColor, const Color(0xFFFF9100));
      expect(recon.coastlineWidth, 1.5);
      expect(recon.enableNightPhosphor, isTrue);
    });

    test('CelestiaPunkMapStyle copyWith applies selective overrides', () {
      const style = CelestiaPunkMapStyle();
      final modified = style.copyWith(
        coastlineWidth: 3.5,
        enableNightPhosphor: false,
      );

      expect(modified.coastlineWidth, 3.5);
      expect(modified.enableNightPhosphor, isFalse);
      expect(modified.coastlineColor, style.coastlineColor);
    });
  });
}
