import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';
import 'package:fluoderpod/fluoderpod.dart';

import '../../../app/theme.dart';

/// Pre-configured tactical radio channels on the Cyan Network mesh.
class TacticalRadioChannel {
  final String id;
  final String callsign;
  final double frequencyMHz;
  final String description;
  final Color accentColor;

  const TacticalRadioChannel({
    required this.id,
    required this.callsign,
    required this.frequencyMHz,
    required this.description,
    this.accentColor = HBColors.primary,
  });
}

/// Tactical Radio transceiver integrating WebRTC P2P voice mesh with the
/// Fluoderpod Cyan Network node relay and decentralized RF channel tuning.
class CyanRadioWidget extends ConsumerStatefulWidget {
  final bool compact;

  const CyanRadioWidget({
    super.key,
    this.compact = false,
  });

  static const List<TacticalRadioChannel> defaultChannels = [
    TacticalRadioChannel(
      id: 'ch-01',
      callsign: 'TAC-SQUAD',
      frequencyMHz: 462.5625,
      description: 'Squad Comms & Tactical Units',
    ),
    TacticalRadioChannel(
      id: 'ch-02',
      callsign: 'GUILD-HQ',
      frequencyMHz: 462.5875,
      description: 'Faction / Clan Command Post',
      accentColor: HBColors.secondary,
    ),
    TacticalRadioChannel(
      id: 'ch-03',
      callsign: 'NEXUS-MESH',
      frequencyMHz: 467.5625,
      description: 'Fluoridian P2P Gossip Relay',
      accentColor: Colors.purpleAccent,
    ),
    TacticalRadioChannel(
      id: 'ch-04',
      callsign: 'DISTRESS-SOS',
      frequencyMHz: 462.6750,
      description: 'Emergency Beacon & Priority Hail',
      accentColor: Colors.redAccent,
    ),
  ];

  @override
  ConsumerState<CyanRadioWidget> createState() => _CyanRadioWidgetState();
}

class _CyanRadioWidgetState extends ConsumerState<CyanRadioWidget>
    with SingleTickerProviderStateMixin {
  int _selectedChannelIndex = 0;
  bool _isRecording = false;
  bool _webRtcActive = false;
  bool _squelchEnabled = true;
  bool _highPowerTx = true;
  late final AnimationController _waveController;

  MediaStream? _localStream;
  RTCPeerConnection? _peerConnection;

  TacticalRadioChannel get _currentChannel =>
      CyanRadioWidget.defaultChannels[_selectedChannelIndex];

  @override
  void initState() {
    super.initState();
    _waveController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1000),
    );
  }

  @override
  void dispose() {
    _stopWebRtcAudio();
    _waveController.dispose();
    super.dispose();
  }

  Future<void> _startWebRtcAudio() async {
    try {
      final Map<String, dynamic> mediaConstraints = {
        'audio': true,
        'video': false,
      };

      _localStream = await navigator.mediaDevices.getUserMedia(mediaConstraints);

      final Map<String, dynamic> configuration = {
        'iceServers': [
          {'urls': 'stun:stun.l.google.com:19302'},
          {'urls': 'stun:stun1.l.google.com:19302'},
        ],
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
      debugPrint('[Cyan Radio] Audio capture & WebRTC peer connection initialized on ${_currentChannel.frequencyMHz} MHz.');
    } catch (e) {
      debugPrint('[Cyan Radio] WebRTC audio start info: $e');
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
      debugPrint('[Cyan Radio] Audio capture released.');
    } catch (e) {
      debugPrint('[Cyan Radio] WebRTC audio stop info: $e');
    }
  }

  void _onPointerDown(PointerDownEvent event) {
    setState(() => _isRecording = true);
    _waveController.repeat(reverse: true);
    _startWebRtcAudio();

    // Broadcast transmit event to Cyan Network state if available
    try {
      ref.read(cyanNetworkProvider.notifier).handleGossipEvent({
        'type': 'ping',
        'channel': _currentChannel.callsign,
        'frequencyMHz': _currentChannel.frequencyMHz,
        'transmitting': true,
      });
    } catch (_) {}
  }

  void _onPointerUp(PointerUpEvent? event) {
    setState(() => _isRecording = false);
    _waveController.stop();
    _waveController.reset();
    _stopWebRtcAudio();

    try {
      ref.read(cyanNetworkProvider.notifier).handleGossipEvent({
        'type': 'ping',
        'channel': _currentChannel.callsign,
        'transmitting': false,
      });
    } catch (_) {}
  }

  void _cycleChannel(int delta) {
    setState(() {
      _selectedChannelIndex = (_selectedChannelIndex + delta) %
          CyanRadioWidget.defaultChannels.length;
      if (_selectedChannelIndex < 0) {
        _selectedChannelIndex += CyanRadioWidget.defaultChannels.length;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    CyanNetworkState network = const CyanNetworkState();
    try {
      network = ref.watch(cyanNetworkProvider);
    } catch (_) {
      // Fallback in environments without ProviderScope
    }

    final channel = _currentChannel;
    final rssiDbm = network.isConnected
        ? -((50 + (network.averageLatencyMs * 0.5)).clamp(50, 110)).round()
        : -120;
    final snrDb = network.isConnected
        ? ((36 - (network.averageLatencyMs * 0.25)).clamp(6, 36)).round()
        : 0;

    return Container(
      padding: EdgeInsets.all(widget.compact ? HBSpacing.sm : HBSpacing.md),
      decoration: BoxDecoration(
        color: const Color(0xFF0A0F1D),
        borderRadius: BorderRadius.circular(HBRadius.lg),
        border: Border.all(
          color: channel.accentColor.withValues(alpha: 0.6),
          width: 1.5,
        ),
        boxShadow: [
          BoxShadow(
            color: channel.accentColor.withValues(alpha: 0.15),
            blurRadius: 16,
            spreadRadius: 2,
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // ── Header: Cyan Network Status & Node Telemetry ───────────────
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: network.isConnected ? Colors.greenAccent : Colors.redAccent,
                      boxShadow: [
                        BoxShadow(
                          color: (network.isConnected ? Colors.greenAccent : Colors.redAccent)
                              .withValues(alpha: 0.6),
                          blurRadius: 6,
                          spreadRadius: 2,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 8),
                  const Text(
                    'CYAN NETWORK RADIO',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      letterSpacing: 1.2,
                    ),
                  ),
                  if (_webRtcActive) ...[
                    const SizedBox(width: 6),
                    const Icon(Icons.mic, size: 14, color: Colors.greenAccent),
                  ],
                ],
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: Colors.black.withValues(alpha: 0.6),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(color: Colors.cyanAccent.withValues(alpha: 0.4)),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.hub, size: 12, color: Colors.cyanAccent),
                    const SizedBox(width: 4),
                    Text(
                      '${network.selectedNodeId} • ${network.averageLatencyMs}ms',
                      style: const TextStyle(
                        color: Colors.cyanAccent,
                        fontSize: 10,
                        fontWeight: FontWeight.w600,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: HBSpacing.sm),

          // ── Tactical Frequency Display & Channel Dial ──────────────────
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: Colors.black.withValues(alpha: 0.8),
              borderRadius: BorderRadius.circular(HBRadius.md),
              border: Border.all(color: channel.accentColor.withValues(alpha: 0.4)),
            ),
            child: Row(
              children: [
                IconButton(
                  icon: const Icon(Icons.arrow_left, size: 28),
                  color: channel.accentColor,
                  onPressed: () => _cycleChannel(-1),
                  tooltip: 'Previous Channel',
                ),
                Expanded(
                  child: Column(
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Text(
                            channel.callsign,
                            style: TextStyle(
                              color: channel.accentColor,
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                              letterSpacing: 2,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: channel.accentColor.withValues(alpha: 0.2),
                              borderRadius: BorderRadius.circular(6),
                            ),
                            child: Text(
                              '${channel.frequencyMHz.toStringAsFixed(4)} MHz',
                              style: TextStyle(
                                color: channel.accentColor,
                                fontSize: 12,
                                fontWeight: FontWeight.bold,
                                fontFamily: 'monospace',
                              ),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        channel.description,
                        style: const TextStyle(
                          color: Colors.white60,
                          fontSize: 10,
                        ),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                ),
                IconButton(
                  icon: const Icon(Icons.arrow_right, size: 28),
                  color: channel.accentColor,
                  onPressed: () => _cycleChannel(1),
                  tooltip: 'Next Channel',
                ),
              ],
            ),
          ),
          const SizedBox(height: HBSpacing.sm),

          // ── Signal Metrics & Squelch Bar ───────────────────────────────
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              _buildMetricChip(
                icon: Icons.signal_cellular_alt,
                label: 'RSSI',
                value: '$rssiDbm dBm',
                color: network.isConnected ? Colors.greenAccent : Colors.redAccent,
              ),
              _buildMetricChip(
                icon: Icons.graphic_eq,
                label: 'SNR',
                value: '+$snrDb dB',
                color: network.isConnected ? Colors.cyanAccent : Colors.white38,
              ),
              InkWell(
                borderRadius: BorderRadius.circular(8),
                onTap: () => setState(() => _squelchEnabled = !_squelchEnabled),
                child: _buildMetricChip(
                  icon: _squelchEnabled ? Icons.volume_up : Icons.volume_off,
                  label: 'SQL',
                  value: _squelchEnabled ? 'ON' : 'OFF',
                  color: _squelchEnabled ? Colors.cyanAccent : Colors.orangeAccent,
                ),
              ),
              InkWell(
                borderRadius: BorderRadius.circular(8),
                onTap: () => setState(() => _highPowerTx = !_highPowerTx),
                child: _buildMetricChip(
                  icon: Icons.bolt,
                  label: 'PWR',
                  value: _highPowerTx ? '5W HI' : '0.5W LO',
                  color: _highPowerTx ? Colors.amberAccent : Colors.white60,
                ),
              ),
            ],
          ),
          const SizedBox(height: HBSpacing.sm),

          // ── Audio Waveform Visualizer ──────────────────────────────────
          Container(
            height: widget.compact ? 45 : 70,
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: HBSpacing.md),
            decoration: BoxDecoration(
              color: Colors.black.withValues(alpha: 0.5),
              borderRadius: BorderRadius.circular(HBRadius.sm),
              border: Border.all(color: Colors.white10),
            ),
            child: AnimatedBuilder(
              animation: _waveController,
              builder: (context, child) {
                return CustomPaint(
                  painter: _CyanAudioWavePainter(
                    isRecording: _isRecording,
                    animationValue: _waveController.value,
                    accentColor: channel.accentColor,
                  ),
                );
              },
            ),
          ),
          const SizedBox(height: HBSpacing.md),

          // ── Push-To-Talk (PTT) Button ──────────────────────────────────
          Listener(
            onPointerDown: _onPointerDown,
            onPointerUp: _onPointerUp,
            onPointerCancel: (e) => _onPointerUp(null),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 120),
              width: widget.compact ? 80 : 100,
              height: widget.compact ? 80 : 100,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: _isRecording
                    ? channel.accentColor
                    : const Color(0xFF131A2E),
                border: Border.all(
                  color: channel.accentColor,
                  width: _isRecording ? 5 : 3,
                ),
                boxShadow: _isRecording
                    ? [
                        BoxShadow(
                          color: channel.accentColor.withValues(alpha: 0.8),
                          blurRadius: 30,
                          spreadRadius: 8,
                        ),
                      ]
                    : [
                        BoxShadow(
                          color: channel.accentColor.withValues(alpha: 0.25),
                          blurRadius: 12,
                        ),
                      ],
              ),
              child: Center(
                child: Icon(
                  Icons.mic,
                  size: widget.compact ? 40 : 48,
                  color: _isRecording ? Colors.black : channel.accentColor,
                ),
              ),
            ),
          ),
          const SizedBox(height: HBSpacing.sm),

          // ── Live Transmission Indicator Text ───────────────────────────
          Text(
            _isRecording
                ? 'TRANSMITTING TO CYAN MESH [${channel.frequencyMHz.toStringAsFixed(4)} MHz]'
                : 'HOLD TO TRANSMIT (CYAN RF MESH)',
            style: TextStyle(
              color: _isRecording ? channel.accentColor : Colors.white70,
              fontSize: widget.compact ? 10 : 11,
              fontWeight: FontWeight.bold,
              letterSpacing: 1.5,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMetricChip({
    required IconData icon,
    required String label,
    required String value,
    required Color color,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.4),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 12, color: color),
          const SizedBox(width: 4),
          Text(
            '$label: ',
            style: const TextStyle(color: Colors.white54, fontSize: 9, fontWeight: FontWeight.bold),
          ),
          Text(
            value,
            style: TextStyle(color: color, fontSize: 10, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
          ),
        ],
      ),
    );
  }
}

class _CyanAudioWavePainter extends CustomPainter {
  final bool isRecording;
  final double animationValue;
  final Color accentColor;

  _CyanAudioWavePainter({
    required this.isRecording,
    required this.animationValue,
    required this.accentColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final midY = size.height / 2;
    final width = size.width;

    if (!isRecording) {
      // Idle baseline carrier signal
      final idlePaint = Paint()
        ..color = accentColor.withValues(alpha: 0.4)
        ..strokeWidth = 1.5
        ..style = PaintingStyle.stroke;

      final path = Path();
      path.moveTo(0, midY);
      for (double i = 0; i <= width; i += 6) {
        final normX = i / width;
        final wave = math.sin(normX * math.pi * 8 + animationValue * math.pi * 2) * 2;
        path.lineTo(i, midY + wave);
      }
      canvas.drawPath(path, idlePaint);
      return;
    }

    final glowPaint = Paint()
      ..color = accentColor.withValues(alpha: 0.6)
      ..strokeWidth = 5
      ..strokeCap = StrokeCap.round
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6)
      ..style = PaintingStyle.stroke;

    final paint = Paint()
      ..color = Colors.white
      ..strokeWidth = 2.5
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;

    final path = Path();
    path.moveTo(0, midY);

    for (double i = 0; i <= width; i += 4) {
      final normalizedX = i / width;
      final envelope = math.sin(normalizedX * math.pi);

      final harmonic1 = math.sin(
        normalizedX * math.pi * 12 + animationValue * math.pi * 4,
      );
      final harmonic2 = math.cos(
        normalizedX * math.pi * 20 - animationValue * math.pi * 3,
      );
      final harmonic3 = math.sin(
        normalizedX * math.pi * 30 + animationValue * math.pi * 6,
      );

      final totalWave = (harmonic1 * 0.5 + harmonic2 * 0.3 + harmonic3 * 0.2) * envelope;
      final y = midY + totalWave * (size.height * 0.45);
      path.lineTo(i, y);
    }

    canvas.drawPath(path, glowPaint);
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant _CyanAudioWavePainter oldDelegate) {
    return oldDelegate.isRecording != isRecording ||
        oldDelegate.animationValue != animationValue ||
        oldDelegate.accentColor != accentColor;
  }
}
