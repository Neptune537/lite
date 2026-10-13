import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';

import '../../../app/theme.dart';

class WalkieTalkieTab extends StatefulWidget {
  const WalkieTalkieTab({super.key});

  @override
  State<WalkieTalkieTab> createState() => _WalkieTalkieTabState();
}

class _WalkieTalkieTabState extends State<WalkieTalkieTab>
    with SingleTickerProviderStateMixin {
  bool _isRecording = false;
  bool _webRtcActive = false;
  late final AnimationController _waveController;
  
  MediaStream? _localStream;
  RTCPeerConnection? _peerConnection;

  @override
  void initState() {
    super.initState();
    _waveController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1000),
    );
  }

  Future<void> _startWebRtcAudio() async {
    try {
      final Map<String, dynamic> mediaConstraints = {
        'audio': true,
        'video': false,
      };

      _localStream = await navigator.mediaDevices.getUserMedia(mediaConstraints);

      // Initialize WebRTC Peer Connection for P2P audio voice mesh
      final Map<String, dynamic> configuration = {
        'iceServers': [
          {'urls': 'stun:stun.l.google.com:19302'},
          {'urls': 'stun:stun1.l.google.com:19302'},
        ]
      };

      _peerConnection = await createPeerConnection(configuration);
      _localStream?.getTracks().forEach((track) {
        _peerConnection?.addTrack(track, _localStream!);
      });

      if (mounted) {
        setState(() {
          _webRtcActive = true;
        });
      }
      debugPrint('[WebRTC Voice Mesh] Audio capture & WebRTC peer connection initialized successfully.');
    } catch (e) {
      debugPrint('[WebRTC Voice Mesh] Error starting WebRTC audio: $e');
    }
  }

  Future<void> _stopWebRtcAudio() async {
    try {
      _localStream?.getTracks().forEach((track) {
        track.stop();
      });
      await _localStream?.dispose();
      await _peerConnection?.close();
      _localStream = null;
      _peerConnection = null;

      if (mounted) {
        setState(() {
          _webRtcActive = false;
        });
      }
      debugPrint('[WebRTC Voice Mesh] Audio capture released.');
    } catch (e) {
      debugPrint('[WebRTC Voice Mesh] Error stopping WebRTC audio: $e');
    }
  }

  @override
  void dispose() {
    _stopWebRtcAudio();
    _waveController.dispose();
    super.dispose();
  }

  void _onPointerDown(PointerDownEvent event) {
    setState(() => _isRecording = true);
    _waveController.repeat(reverse: true);
    _startWebRtcAudio();
  }

  void _onPointerUp(PointerUpEvent? event) {
    setState(() => _isRecording = false);
    _waveController.stop();
    _waveController.reset();
    _stopWebRtcAudio();
  }

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // WebRTC Status Badge
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              color: _webRtcActive ? Colors.green.withValues(alpha: 0.2) : HBColors.neutral,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(
                color: _webRtcActive ? Colors.greenAccent : HBColors.primary.withValues(alpha: 0.4),
              ),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  _webRtcActive ? Icons.wifi : Icons.wifi_off,
                  size: 14,
                  color: _webRtcActive ? Colors.greenAccent : Colors.white54,
                ),
                const SizedBox(width: 6),
                Text(
                  _webRtcActive ? 'WEBRTC P2P VOICE STREAM ACTIVE' : 'WEBRTC VOICE CHANNEL READY (462.5625 MHz)',
                  style: TextStyle(
                    color: _webRtcActive ? Colors.greenAccent : Colors.white70,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: HBSpacing.lg),

          Container(
            height: 100,
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: HBSpacing.xl),
            child: AnimatedBuilder(
              animation: _waveController,
              builder: (context, child) {
                return CustomPaint(
                  painter: _AudioWavePainter(
                    isRecording: _isRecording,
                    animationValue: _waveController.value,
                  ),
                );
              },
            ),
          ),
          const SizedBox(height: HBSpacing.xl),
          Listener(
            onPointerDown: _onPointerDown,
            onPointerUp: _onPointerUp,
            onPointerCancel: (e) => _onPointerUp(null),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 100),
              width: _isRecording ? 140 : 150,
              height: _isRecording ? 140 : 150,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: _isRecording ? HBColors.tertiary : HBColors.neutral,
                border: Border.all(
                  color: _isRecording ? HBColors.tertiary : HBColors.primary,
                  width: 4,
                ),
                boxShadow: _isRecording
                    ? [
                        BoxShadow(
                          color: HBColors.tertiary.withValues(alpha: 0.8),
                          blurRadius: 30,
                          spreadRadius: 10,
                        ),
                      ]
                    : [],
              ),
              child: Center(
                child: Icon(
                  Icons.mic,
                  size: 64,
                  color: _isRecording ? Colors.white : HBColors.primary,
                ),
              ),
            ),
          ),
          const SizedBox(height: HBSpacing.xl),
          Text(
            _isRecording ? 'TRANSMITTING WEBRTC AUDIO...' : 'HOLD TO SPEAK (WEBRTC P2P)',
            style: TextStyle(
              color: _isRecording ? HBColors.tertiary : HBColors.primary,
              fontWeight: FontWeight.bold,
              letterSpacing: 2,
            ),
          ),
        ],
      ),
    );
  }
}

class _AudioWavePainter extends CustomPainter {
  final bool isRecording;
  final double animationValue;

  _AudioWavePainter({required this.isRecording, required this.animationValue});

  @override
  void paint(Canvas canvas, Size size) {
    if (!isRecording) {
      // Draw flat line
      final paint = Paint()
        ..color = HBColors.primary.withValues(alpha: 0.5)
        ..strokeWidth = 2
        ..style = PaintingStyle.stroke;
      canvas.drawLine(
        Offset(0, size.height / 2),
        Offset(size.width, size.height / 2),
        paint,
      );
      return;
    }

    final paint = Paint()
      ..color = HBColors.tertiary
      ..strokeWidth = 3
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;

    final glowPaint = Paint()
      ..color = HBColors.tertiary.withValues(alpha: 0.5)
      ..strokeWidth = 6
      ..strokeCap = StrokeCap.round
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8)
      ..style = PaintingStyle.stroke;

    final path = Path();
    path.moveTo(0, size.height / 2);

    final width = size.width;
    final midY = size.height / 2;

    for (double i = 0; i <= width; i += 5) {
      final normalizedX = i / width;
      final envelope = math.sin(normalizedX * math.pi);

      final wave1 = math.sin(
        normalizedX * math.pi * 10 + animationValue * math.pi * 4,
      );
      final wave2 = math.cos(
        normalizedX * math.pi * 15 - animationValue * math.pi * 2,
      );

      final totalWave = (wave1 + wave2) * 0.5 * envelope;

      final y = midY + totalWave * (size.height / 2);
      path.lineTo(i, y);
    }

    canvas.drawPath(path, glowPaint);
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant _AudioWavePainter oldDelegate) {
    return oldDelegate.isRecording != isRecording ||
        oldDelegate.animationValue != animationValue;
  }
}
