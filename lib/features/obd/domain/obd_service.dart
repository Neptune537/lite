import 'dart:async';

import 'package:flutter/services.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'obd_service.g.dart';

class ObdData {
  final String rpm;
  final String speed;
  final bool isConnected;

  ObdData({required this.rpm, required this.speed, required this.isConnected});

  ObdData copyWith({String? rpm, String? speed, bool? isConnected}) {
    return ObdData(
      rpm: rpm ?? this.rpm,
      speed: speed ?? this.speed,
      isConnected: isConnected ?? this.isConnected,
    );
  }
}

class ObdService {
  static const MethodChannel _methodChannel = MethodChannel(
    'com.heavenlybond.hblite/obd_methods',
  );
  static const EventChannel _eventChannel = EventChannel(
    'com.heavenlybond.hblite/obd_data',
  );

  Future<void> connectToDevice(String macAddress) async {
    await _methodChannel.invokeMethod('connectToDevice', {
      'address': macAddress,
    });
  }

  Future<void> disconnect() async {
    await _methodChannel.invokeMethod('disconnect');
  }

  Stream<Map<String, dynamic>> get obdStream {
    return _eventChannel.receiveBroadcastStream().map(
      (event) => Map<String, dynamic>.from(event),
    );
  }
}

@riverpod
ObdService obdService(Ref ref) {
  return ObdService();
}

@riverpod
class ObdStateNotifier extends _$ObdStateNotifier {
  StreamSubscription? _subscription;

  @override
  ObdData build() {
    ref.onDispose(() {
      _subscription?.cancel();
    });

    final service = ref.watch(obdServiceProvider);

    _subscription = service.obdStream.listen((event) {
      if (event['type'] == 'data') {
        state = state.copyWith(
          rpm: event['rpm'] as String,
          speed: event['speed'] as String,
        );
      } else if (event['type'] == 'connection') {
        state = state.copyWith(isConnected: event['isConnected'] as bool);
      }
    });

    return ObdData(rpm: '0', speed: '0', isConnected: false);
  }

  Future<void> connect(String macAddress) async {
    await ref.read(obdServiceProvider).connectToDevice(macAddress);
  }

  Future<void> disconnect() async {
    await ref.read(obdServiceProvider).disconnect();
  }
}
