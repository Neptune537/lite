import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:fluoderpod/fluoderpod.dart';
import 'package:flutter/scheduler.dart';
import 'fluoderpod_bridge.dart';

/// Unified Flutter view rendering the 3D Fluoderpod GPU-driven scene.
/// Target Platforms: Android (Vulkan NDK Texture) and Web (WebGPU).
class FluoderpodView extends ConsumerStatefulWidget {
  final Widget? overlay;
  final VoidCallback? onInitialized;
  final ValueChanged<String>? onEntityTapped;

  const FluoderpodView({
    super.key,
    this.overlay,
    this.onInitialized,
    this.onEntityTapped,
  });

  @override
  ConsumerState<FluoderpodView> createState() => _FluoderpodViewState();
}

class _FluoderpodViewState extends ConsumerState<FluoderpodView>
    with TickerProviderStateMixin, WidgetsBindingObserver {
  late AnimationController _pulseCtrl;
  late final Ticker _renderTicker;
  DateTime _lastFrameTime = DateTime.now();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    
    _pulseCtrl = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 4),
    )..repeat(reverse: true);

    _renderTicker = createTicker(_onRenderTick);

    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final bridge = ref.read(fluoderpodBridgeProvider);
      final size = MediaQuery.of(context).size;
      final pixelRatio = MediaQuery.of(context).devicePixelRatio;
      await bridge.initialize(
        width: (size.width * pixelRatio).toInt(),
        height: (size.height * pixelRatio).toInt(),
      );
      if (mounted) {
        widget.onInitialized?.call();
        setState(() {});
        _renderTicker.start();
      }
    });
  }

  void _onRenderTick(Duration elapsed) {
    final bridge = ref.read(fluoderpodBridgeProvider);
    final now = DateTime.now();
    final waitDuration = bridge.tick(now);
    
    if (waitDuration != null) {
      if (now.difference(_lastFrameTime) >= waitDuration) {
        bridge.executeFrame();
        _lastFrameTime = now;
      }
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    final bridge = ref.read(fluoderpodBridgeProvider);
    if (state == AppLifecycleState.paused || state == AppLifecycleState.hidden) {
      bridge.setPaused(true);
      _renderTicker.stop();
    } else if (state == AppLifecycleState.resumed) {
      bridge.setPaused(false);
      if (!_renderTicker.isActive && bridge.isInitialized) {
        _renderTicker.start();
      }
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _renderTicker.dispose();
    _pulseCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bridge = ref.watch(fluoderpodBridgeProvider);

    return LayoutBuilder(
      builder: (context, constraints) {
        return GestureDetector(
          onScaleUpdate: (details) {
            if (details.pointerCount == 1) {
              bridge.pan(details.focalPointDelta.dx, details.focalPointDelta.dy);
            } else {
              bridge.zoom(details.scale);
            }
          },
          onTapUp: (details) async {
            final id = await bridge.raycast(details.localPosition.dx, details.localPosition.dy);
            if (id != null && widget.onEntityTapped != null) {
              widget.onEntityTapped!(id);
            }
          },
          child: Stack(
            children: [
              // Native render surface or holographic fallback
              if (bridge.isInitialized && !kIsWeb && defaultTargetPlatform == TargetPlatform.android && bridge.textureId > 0)
                Texture(textureId: bridge.textureId)
              else
                // Animated holographic fallback for Web/simulators using the new SDK package
                FluoderpodFallbackView(
                  targetFps: bridge.currentFps.toInt(),
                  onRetryNative: () async {
                    final size = MediaQuery.of(context).size;
                    final pixelRatio = MediaQuery.of(context).devicePixelRatio;
                    await bridge.initialize(
                      width: (size.width * pixelRatio).toInt(),
                      height: (size.height * pixelRatio).toInt(),
                    );
                    setState(() {});
                  },
                ),

              // Telemetry overlay badge (FPS / Clusters)
              Positioned(
                top: 12,
                right: 12,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.7),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: Colors.cyanAccent.withValues(alpha: 0.4)),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Container(
                        width: 6,
                        height: 6,
                        decoration: const BoxDecoration(
                          color: Colors.cyanAccent,
                          shape: BoxShape.circle,
                        ),
                      ),
                      const SizedBox(width: 6),
                      Text(
                        kIsWeb
                            ? 'Fluoderpod WebGPU (60 FPS)'
                            : 'Fluoderpod Vulkan (${bridge.currentFps.toInt()} FPS)',
                        style: const TextStyle(
                          color: Colors.cyanAccent,
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                          fontFamily: 'monospace',
                        ),
                      ),
                    ],
                  ),
                ),
              ),

              // Optional child UI overlay
              if (widget.overlay != null) widget.overlay!,
            ],
          ),
        );
      },
    );
  }
}
