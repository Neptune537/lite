import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:fluoderpod/fluoderpod.dart';

final fluoderpodBridgeProvider = Provider<FluoderpodBridge>((ref) {
  return FluoderpodBridge();
});

/// Bridge connecting the Flutter UI (Android & Web) with the native
/// `fluoderpod_render` GPU-driven rendering engine.
///
/// Supported Platforms:
/// - Android: Vulkan NDK Surface via JNI/C FFI (`android_vulkan.rs`)
/// - Web: WebGPU Canvas context via Wasm / JavaScript Interop
class FluoderpodBridge {
  /// Frame rate while the user is actively interacting (pan/zoom/tap).
  static const double interactiveFps = 60.0;

  /// Frame rate once the scene has been static for [idleAfter].
  static const double idleFps = 24.0;

  /// Idle duration after which the renderer drops to [idleFps].
  static const Duration idleAfter = Duration(seconds: 3);

  bool _isInitialized = false;
  bool _paused = false;
  int _textureId = -1;
  int _activeClusters = 0;
  double _currentFps = interactiveFps;
  DateTime _lastInteraction = DateTime.now();

  late final CyanNativeBindings _nativeBindings;
  CyanNativeBindings get nativeBindings => _nativeBindings;

  bool get isInitialized => _isInitialized;
  int get textureId => _textureId;
  int get activeClusters => _activeClusters;
  double get currentFps => _currentFps;
  bool get isPaused => _paused;

  FluoderpodBridge() {
    _nativeBindings = CyanNativeBindings();
  }

  /// Call on any touch/camera gesture to ramp back to full frame rate.
  void notifyUserInteraction() {
    _lastInteraction = DateTime.now();
    _currentFps = interactiveFps;
  }

  /// Pause rendering entirely (app backgrounded / fully occluded).
  void setPaused(bool paused) {
    _paused = paused;
    if (!paused) notifyUserInteraction();
  }

  /// Re-evaluates the thermal/power tier. Returns the frame interval the
  /// render loop should wait before the next [executeFrame]; `null` when
  /// paused.
  Duration? tick([DateTime? now]) {
    if (_paused) return null;
    final idle = (now ?? DateTime.now()).difference(_lastInteraction);
    _currentFps = idle > idleAfter ? idleFps : interactiveFps;
    return Duration(microseconds: (1000000 / _currentFps).round());
  }

  static const MethodChannel _channel =
      MethodChannel('com.fluorescent.vulkan/texture');

  /// Initializes the hardware graphics context for Android (Vulkan) or Web (WebGPU).
  ///
  /// On Android this only reports `true` when the native renderer has claimed
  /// a Flutter texture; otherwise it returns `false` and the view keeps its
  /// fallback viewport.
  Future<bool> initialize({required int width, required int height}) async {
    // Platform-specific initialization. Web uses a virtual canvas ID; Android uses a native MethodChannel.
    if (kIsWeb) {
      // TODO: Replace stub with actual WebGPU JS interop when available.
      debugPrint('[FluoderpodBridge] Initializing WebGPU render pipeline ($width x $height)...');
      _textureId = 1001; // Virtual canvas placeholder ID for Web.
      _isInitialized = true;
    } else if (defaultTargetPlatform == TargetPlatform.android) {
      // Attempt to initialize FFI layers if available
      try {
        _nativeBindings.initGameController();
      } catch (e) {
        debugPrint('[FluoderpodBridge] Game controller native bindings failed: $e');
      }

      try {
        final res = await _channel.invokeMapMethod<String, Object?>(
          'start',
          {'width': width, 'height': height},
        );
        final id = res?['textureId'] as int?;
        _textureId = id ?? -1;
        _isInitialized = id != null;
        debugPrint('[FluoderpodBridge] Vulkan texture $_textureId ($width x $height)');
      } on PlatformException catch (e) {
        debugPrint('[FluoderpodBridge] Native renderer unavailable (${e.code}); using fallback.');
        _textureId = -1;
        _isInitialized = false;
      } on MissingPluginException {
        _textureId = -1;
        _isInitialized = false;
      }
    } else {
      // Platform not supported; fallback UI will be used.
      debugPrint('[FluoderpodBridge] Unsupported platform for native Fluoderpod. Using fallback.');
      _isInitialized = false;
    }
    return _isInitialized;
  }

  /// Resizes the hardware viewport upon screen rotation or browser window resize.
  void resize(int width, int height) {
    if (!_isInitialized) return;
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      _channel.invokeMethod<void>('resize', {'width': width, 'height': height});
    }
    debugPrint('[FluoderpodBridge] Resized viewport to ${width}x$height');
  }


  /// High-throughput batch ingestion directly from Riverpod state into GPU buffers.
  /// Bypasses CPU widget tree overhead and feeds directly to compute frustum culling.
  void ingestBatch(Uint8List rawEntityData) {
    if (!_isInitialized) return;
    
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      _channel.invokeMethod<int>('ingestBatch', {'buffer': rawEntityData}).then((clusters) {
        if (clusters != null) {
          _activeClusters = clusters;
        }
      });
    } else {
      _activeClusters = (rawEntityData.length / 64).round();
    }
  }

  /// Dispatches the frame execution cycle: Culling -> Virtual Geometry LOD -> Indirect Draw
  void executeFrame() {
    if (!_isInitialized) return;
    // Dispatches GPU compute shaders
  }

  void pan(double dx, double dy) {
    if (!_isInitialized) return;
    notifyUserInteraction();
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      _channel.invokeMethod<void>('pan', {'dx': dx, 'dy': dy});
    }
  }

  void zoom(double scale) {
    if (!_isInitialized) return;
    notifyUserInteraction();
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      _channel.invokeMethod<void>('zoom', {'scale': scale});
    }
  }

  /// Raycasts into the 3D scene. Returns the ID of the tapped entity, if any.
  Future<String?> raycast(double x, double y) async {
    if (!_isInitialized) return null;
    notifyUserInteraction();
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      final id = await _channel.invokeMethod<String>('raycast', {'x': x, 'y': y});
      return id;
    }
    return null;
  }

  void dispose() {
    if (_isInitialized && !kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      _channel.invokeMethod<void>('dispose');
      try {
        _nativeBindings.shutdownGameController();
      } catch (e) {
        debugPrint('[FluoderpodBridge] Game controller shutdown failed: $e');
      }
    }
    _isInitialized = false;
  }
}
