import 'dart:html' as html;
import 'dart:ui_web' as ui_web;
import 'package:flutter/material.dart';

bool _cesiumFactoryRegistered = false;
html.IFrameElement? _cesiumIframeElement;

String _generateCesiumHtml(double lat, double lng, String title) {
  // Precomputed tactical route waypoints starting from target location
  final turn1Lat = lat + 0.0035;
  final turn1Lng = lng + 0.0025;
  final turn2Lat = lat + 0.0070;
  final turn2Lng = lng + 0.0060;
  final destLat = lat + 0.0110;
  final destLng = lng + 0.0095;

  return '''<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>$title</title>
  <link rel="stylesheet" href="https://cesium.com/downloads/cesiumjs/releases/1.120/Build/Cesium/Widgets/widgets.css">
  <script src="https://cesium.com/downloads/cesiumjs/releases/1.120/Build/Cesium/Cesium.js"></script>
  <style>
    html, body, #cesiumContainer { width: 100%; height: 100%; margin: 0; padding: 0; overflow: hidden; background: #070C15; font-family: monospace; }
    .cesium-widget-credits { display: none !important; }
    #hudOverlay {
      position: absolute;
      top: 10px;
      left: 10px;
      z-index: 999;
      background: rgba(13, 20, 36, 0.92);
      border: 1px solid #00E5FF;
      border-radius: 6px;
      padding: 8px 12px;
      color: #00E5FF;
      font-size: 11px;
      pointer-events: auto;
      box-shadow: 0 4px 12px rgba(0,0,0,0.6);
    }
    .hud-title { font-weight: bold; margin-bottom: 4px; color: #FFF; font-size: 12px; }
    .hud-btn {
      background: #00E5FF;
      color: #070C15;
      border: none;
      border-radius: 4px;
      padding: 5px 10px;
      font-weight: bold;
      cursor: pointer;
      margin-top: 6px;
      margin-right: 6px;
    }
    .hud-btn:hover { background: #66FFFF; }
  </style>
</head>
<body>
  <div id="hudOverlay">
    <div class="hud-title">$title</div>
    <div>Origin: $lat, $lng</div>
    <div>Status: <span id="driveStatus">Orbit View</span></div>
    <button class="hud-btn" id="btnDrive">▶ FPS Drive (60 km/h)</button>
    <button class="hud-btn" id="btnOrbit">⟲ Orbit View</button>
  </div>
  <div id="cesiumContainer"></div>
  <script>
    window.addEventListener('DOMContentLoaded', async () => {
      try {
        const viewer = new Cesium.Viewer('cesiumContainer', {
          animation: false,
          timeline: false,
          geocoder: false,
          homeButton: false,
          baseLayerPicker: false,
          navigationHelpButton: false,
          sceneModePicker: false,
          fullscreenButton: false,
          skyAtmosphere: new Cesium.SkyAtmosphere(),
          imageryProvider: new Cesium.OpenStreetMapImageryProvider({
            url: 'https://a.basemaps.cartocdn.com/dark_all/'
          })
        });

        viewer.scene.globe.enableLighting = true;

        // Load Google Photorealistic 3D Tiles if available
        try {
          if (Cesium.createGooglePhotorealistic3DTileset) {
            const googleTileset = await Cesium.createGooglePhotorealistic3DTileset();
            viewer.scene.primitives.add(googleTileset);
          }
        } catch (_) {}

        // Waypoints for hybrid routing projection
        const waypoints = [
          { lon: $lng, lat: $lat, alt: 2.0 },
          { lon: $turn1Lng, lat: $turn1Lat, alt: 2.0 },
          { lon: $turn2Lng, lat: $turn2Lat, alt: 2.0 },
          { lon: $destLng, lat: $destLat, alt: 2.0 }
        ];

        const flatDegreesArray = [
          $lng, $lat,
          $turn1Lng, $turn1Lat,
          $turn2Lng, $turn2Lat,
          $destLng, $destLat
        ];

        // 1. Plot Start Node Entity
        viewer.entities.add({
          position: Cesium.Cartesian3.fromDegrees($lng, $lat, 50),
          point: { pixelSize: 10, color: Cesium.Color.fromCssColorString('#00E5FF'), outlineColor: Cesium.Color.WHITE, outlineWidth: 2 },
          label: { text: 'Start Node ($lat, $lng)', font: '11px monospace', fillColor: Cesium.Color.fromCssColorString('#00E5FF'), pixelOffset: new Cesium.Cartesian2(0, -18) }
        });

        // 2. Plot Destination Node Entity
        viewer.entities.add({
          position: Cesium.Cartesian3.fromDegrees($destLng, $destLat, 50),
          point: { pixelSize: 10, color: Cesium.Color.fromCssColorString('#00FF66'), outlineColor: Cesium.Color.WHITE, outlineWidth: 2 },
          label: { text: 'Destination ($destLat, $destLng)', font: '11px monospace', fillColor: Cesium.Color.fromCssColorString('#00FF66'), pixelOffset: new Cesium.Cartesian2(0, -18) }
        });

        // 3. Plot 3D Clamped Polyline Route (Hybrid Routing Visual)
        viewer.entities.add({
          name: 'Tactical Drive Route',
          polyline: {
            positions: Cesium.Cartesian3.fromDegreesArray(flatDegreesArray),
            width: 6,
            material: Cesium.Color.fromCssColorString('#00E5FF'),
            clampToGround: true
          }
        });

        // Initial Orbit Fly-To
        function setOrbitView() {
          viewer.camera.flyTo({
            destination: Cesium.Cartesian3.fromDegrees($lng, $lat, 3500),
            orientation: {
              pitch: Cesium.Math.toRadians(-45.0),
              heading: Cesium.Math.toRadians(35.0)
            },
            duration: 1.5
          });
          document.getElementById('driveStatus').innerText = 'Orbit View';
        }
        setOrbitView();

        // 4. Drive FPS Camera Along Route
        let driveTimer = null;
        let isDriving = false;

        function startFpsDrive() {
          if (isDriving) return;
          isDriving = true;
          document.getElementById('driveStatus').innerText = 'FPS Driving (60 km/h)';

          let index = 0;
          let subStep = 0;
          const totalSubSteps = 30;

          function driveStep() {
            if (!isDriving) return;
            if (index >= waypoints.length - 1) {
              document.getElementById('driveStatus').innerText = 'Destination Arrived';
              isDriving = false;
              return;
            }

            const p0 = waypoints[index];
            const p1 = waypoints[index + 1];

            const t = subStep / totalSubSteps;
            const curLon = p0.lon + (p1.lon - p0.lon) * t;
            const curLat = p0.lat + (p1.lat - p0.lat) * t;
            const curAlt = 1.8; // Eye-level driving altitude

            const dLon = p1.lon - p0.lon;
            const dLat = p1.lat - p0.lat;
            const heading = Math.atan2(dLon, dLat);

            viewer.camera.setView({
              destination: Cesium.Cartesian3.fromDegrees(curLon, curLat, curAlt),
              orientation: {
                heading: heading,
                pitch: Cesium.Math.toRadians(-2.0),
                roll: 0.0
              }
            });

            subStep++;
            if (subStep >= totalSubSteps) {
              subStep = 0;
              index++;
            }

            driveTimer = setTimeout(driveStep, 50);
          }

          driveStep();
        }

        function stopFpsDrive() {
          isDriving = false;
          if (driveTimer) clearTimeout(driveTimer);
          setOrbitView();
        }

        document.getElementById('btnDrive').onclick = startFpsDrive;
        document.getElementById('btnOrbit').onclick = stopFpsDrive;

      } catch (e) {
        console.error("Cesium initialization error:", e);
      }
    });
  </script>
</body>
</html>''';
}

Widget buildCesiumMapView(double lat, double lng, {String? title}) {
  const viewType = 'cesium-3d-globe-overlay';
  final effectiveTitle = title ?? 'CESIUM 3D TACTICAL NAV';
  final htmlContent = _generateCesiumHtml(lat, lng, effectiveTitle);

  if (!_cesiumFactoryRegistered) {
    ui_web.platformViewRegistry.registerViewFactory(
      viewType,
      (int viewId) {
        _cesiumIframeElement = html.IFrameElement()
          ..style.border = 'none'
          ..style.width = '100%'
          ..style.height = '100%'
          ..srcdoc = htmlContent
          ..allowFullscreen = true;
        return _cesiumIframeElement!;
      },
    );
    _cesiumFactoryRegistered = true;
  } else {
    if (_cesiumIframeElement != null) {
      _cesiumIframeElement!.srcdoc = htmlContent;
    }
  }

  return const HtmlElementView(viewType: viewType);
}
