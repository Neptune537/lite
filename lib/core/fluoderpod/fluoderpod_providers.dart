import 'package:cloud_firestore/cloud_firestore.dart';
import 'package:fluoderpod/fluoderpod.dart';

import '../../features/board/presentation/board_providers.dart';
import '../../features/tasks/domain/task.dart';
import '../../features/teams/domain/team.dart';
import '../../features/teams/presentation/team_providers.dart';
import '../../shared/models/user_profile.dart';

/// Maps a [Task] to a GPU entity packet.
EntityPacket taskToPacket(Task t) {
  return EntityPacket(
    id: t.id,
    x: t.location.longitude,
    y: t.location.latitude,
    type: FluoderpodEntityType.task,
    status: t.status.index,
    bountyMicro: (t.budgetAmount * 1000000).round(),
  );
}

/// Maps a [Team] (Guild Hub) to a GPU entity packet.
EntityPacket teamToPacket(Team t) {
  return EntityPacket(
    id: 'team_${t.id}',
    x: t.hubLocation?.longitude ?? 0.0,
    y: t.hubLocation?.latitude ?? 0.0,
    type: FluoderpodEntityType.guildHub,
    status: 1,
    bountyMicro: (t.rating * 1000000).round(), // Rating visualization
  );
}

/// Maps a [UserProfile] (Player) to a GPU entity packet.
EntityPacket userToPacket(UserProfile u) {
  return EntityPacket(
    id: 'user_${u.uid}',
    x: u.lastKnownLocation?.longitude ?? 0.0,
    y: u.lastKnownLocation?.latitude ?? 0.0,
    type: FluoderpodEntityType.player,
    status: 1,
    bountyMicro: (u.rating * 1000000).round(), // Rating visualization
  );
}

final allUsersStreamProvider = StreamProvider<List<UserProfile>>((ref) {
  return FirebaseFirestore.instance.collection('users').snapshots().map(
        (snap) => snap.docs.map((doc) => UserProfile.fromJson(doc.data())).toList(),
      );
});

/// Streams tasks, guild hubs, and players into the GPU scene.
final fluoderpodEntityStreamProvider = Provider<int>((ref) {
  final ingest = ref.read(cyanTelemetryProvider.notifier);
  final tasks = ref.watch(publicTasksProvider).value ?? const <Task>[];
  final teamsList = ref.watch(teamsProvider).value ?? const <Team>[];
  final users = ref.watch(allUsersStreamProvider).value ?? const <UserProfile>[];

  final packets = [
    ...tasks.map(taskToPacket),
    ...teamsList.where((t) => t.hubLocation != null).map(teamToPacket),
    ...users.where((u) => u.lastKnownLocation != null).map(userToPacket),
  ];
  
  ingest.ingestEntities(packets);
  return packets.length;
});
