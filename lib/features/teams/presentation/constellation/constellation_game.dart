import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flame/game.dart';
import 'package:flutter/material.dart';
import 'package:fluorescent_core/fluorescent_core.dart';

import 'package:fluorescent_flame/fluorescent_flame.dart';
import 'dart:math' as math;

import '../../domain/team.dart';

class ConstellationNode extends PositionComponent with TapCallbacks {
  final Team team;
  final bool isMotherOrg;
  final VoidCallback onTapped;

  ConstellationNode({
    required this.team,
    required this.isMotherOrg,
    required this.onTapped,
    super.position,
  }) : super(size: Vector2.all(isMotherOrg ? 64 : 32), anchor: Anchor.center);

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final paint = Paint()
      ..color = isMotherOrg ? Colors.amber : Colors.cyanAccent
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 10);

    // Core star
    canvas.drawCircle(
      Offset(size.x / 2, size.y / 2),
      size.x / 2.5,
      Paint()..color = Colors.white,
    );

    // Glow
    canvas.drawCircle(Offset(size.x / 2, size.y / 2), size.x / 2, paint);

    // Label
    final textPainter = TextPainter(
      text: TextSpan(
        text: team.name,
        style: const TextStyle(color: Colors.white, fontSize: 12),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    textPainter.paint(
      canvas,
      Offset(size.x / 2 - textPainter.width / 2, size.y + 5),
    );
  }

  @override
  void onTapUp(TapUpEvent event) {
    onTapped();
  }
}

class TreatyLink extends Component {
  final ConstellationNode node1;
  final ConstellationNode node2;

  TreatyLink(this.node1, this.node2);

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final paint = Paint()
      ..color = Colors.greenAccent.withValues(alpha: 0.5)
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;

    canvas.drawLine(
      node1.position.toOffset(),
      node2.position.toOffset(),
      paint,
    );
  }
}

class BranchLink extends Component {
  @override
  final ConstellationNode parent;
  final ConstellationNode child;

  BranchLink(this.parent, this.child);

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final paint = Paint()
      ..color = Colors.blueAccent.withValues(alpha: 0.5)
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;

    canvas.drawLine(
      parent.position.toOffset(),
      child.position.toOffset(),
      paint,
    );
  }
}

class ConstellationGame extends FlameGame with PanDetector {
  final List<Team> teams;
  final void Function(Team) onTeamTapped;

  late final FluorescentViewport _viewport;
  final Map<String, ConstellationNode> _nodes = {};

  // Camera dragging
  final Vector2 _dragVelocity = Vector2.zero();

  ConstellationGame({required this.teams, required this.onTeamTapped});

  @override
  Future<void> onLoad() async {
    // 1. Initialize 3D world (stub)
    final world3D = World3D(name: 'Constellation');
    final camera3D = Camera3D();

    _viewport = FluorescentViewport(
      world: world3D,
      camera: camera3D,
      size: size,
    );

    add(_viewport);

    // 2. Build 2D overlay nodes based on teams
    _buildNodes();
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    if (isLoaded) {
      _viewport.size = size;
      camera.viewport.size = size;
    }
  }

  void _buildNodes() {
    // Simple circular layout for now
    final center = size / 2;
    const double radius = 150;
    final double angleStep = (2 * 3.14159) / (teams.isEmpty ? 1 : teams.length);

    for (int i = 0; i < teams.length; i++) {
      final team = teams[i];
      final isMother = team.parentIds.isEmpty; // Just a heuristic

      final angle = i * angleStep;
      final position =
          center + Vector2(radius * math.cos(angle), radius * math.sin(angle));

      final node = ConstellationNode(
        team: team,
        isMotherOrg: isMother,
        position: position,
        onTapped: () => onTeamTapped(team),
      );

      _nodes[team.id] = node;
      add(node);
    }

    // Add links
    for (final team in teams) {
      final node = _nodes[team.id];
      if (node == null) continue;

      // Branch links
      for (final parentId in team.parentIds) {
        final parentNode = _nodes[parentId];
        if (parentNode != null) {
          add(BranchLink(parentNode, node));
        }
      }

      // Treaty links (avoiding duplicate renders if both list it)
      for (final treatyId in team.treatyIds) {
        // Just arbitrarily draw if this team's ID < treaty team's ID to avoid 2 lines
        if (team.id.compareTo(treatyId) < 0) {
          final treatyNode = _nodes[treatyId];
          if (treatyNode != null) {
            add(TreatyLink(node, treatyNode));
          }
        }
      }
    }
  }

  @override
  void onPanUpdate(DragUpdateInfo info) {
    // Simple 2D panning over the constellation
    camera.viewfinder.position -= info.delta.global;
  }
}
