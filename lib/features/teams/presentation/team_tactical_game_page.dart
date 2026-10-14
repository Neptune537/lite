
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../app/theme.dart';
import '../../game/presentation/providers/game_provider.dart';
import '../../game/presentation/providers/game_state.dart' as game_state;
import '../../game/presentation/providers/game_state.dart' hide Card;

class TeamTacticalGamePage extends ConsumerStatefulWidget {
  const TeamTacticalGamePage({super.key});

  @override
  ConsumerState<TeamTacticalGamePage> createState() => _TeamTacticalGamePageState();
}

class _TeamTacticalGamePageState extends ConsumerState<TeamTacticalGamePage> {
  SetupItemType _selectedSetupItem = SetupItemType.piece;
  game_state.Card? _selectedSetupCard;

  @override
  Widget build(BuildContext context) {
    final gameState = ref.watch(gameProvider);
    final notifier = ref.read(gameProvider.notifier);

    return Scaffold(
      backgroundColor: const Color(0xFF0C0F1D),
      appBar: AppBar(
        title: const Text(
          'Faction War Simulation',
          style: TextStyle(
            fontFamily: 'Space Grotesk',
            fontWeight: FontWeight.bold,
            color: HBColors.primary,
          ),
        ),
        backgroundColor: Colors.transparent,
        elevation: 0,
        centerTitle: true,
      ),
      body: Column(
        children: [
          _buildStatusPanel(gameState, PlayerId.p2),
          const Divider(color: HBColors.primary, height: 1),

          Expanded(
            child: Center(
              child: AspectRatio(
                aspectRatio: 1,
                child: _buildBoard(gameState, notifier),
              ),
            ),
          ),

          const Divider(color: HBColors.primary, height: 1),
          if (gameState.phase == GamePhase.setup)
            _buildSetupControls(gameState, notifier),
          if (gameState.phase == GamePhase.playing)
            _buildActionControls(gameState, notifier),
          if (gameState.phase == GamePhase.gameOver) _buildGameOver(gameState),

          _buildStatusPanel(gameState, PlayerId.p1),
        ],
      ),
    );
  }

  Widget _buildStatusPanel(GameState state, PlayerId player) {
    final pState = state.players[player]!;
    final isCurrent = state.currentPlayer == player;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: isCurrent && state.phase != GamePhase.gameOver
          ? HBColors.primary.withValues(alpha: 0.1)
          : Colors.transparent,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            player == PlayerId.p1 ? 'FACTION 1' : 'FACTION 2',
            style: TextStyle(
              color: player == PlayerId.p1
                  ? HBColors.primary
                  : HBColors.secondary,
              fontWeight: FontWeight.bold,
              fontSize: 18,
            ),
          ),
          Row(
            children: [
              const Icon(Icons.favorite, color: Colors.redAccent, size: 20),
              const SizedBox(width: 4),
              Text(
                '${pState.hp} HP',
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 18,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(width: 16),
              Text(
                'Traps: ${pState.unplacedTraps.length}',
                style: const TextStyle(color: Colors.white70),
              ),
              const SizedBox(width: 16),
              Text(
                'Pieces: ${pState.piecesToPlace}',
                style: const TextStyle(color: Colors.white70),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildBoard(GameState state, GameNotifier notifier) {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: const Color(0xFF13182C),
        border: Border.all(color: HBColors.primary),
        borderRadius: BorderRadius.circular(HBRadius.lg),
      ),
      child: GridView.builder(
        physics: const NeverScrollableScrollPhysics(),
        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
          crossAxisCount: 7,
          crossAxisSpacing: 4,
          mainAxisSpacing: 4,
        ),
        itemCount: 49, // 7x7 grid
        itemBuilder: (context, index) {
          final x = index % 7;
          final y = index ~/ 7;
          final point = Point(x, y);

          return GestureDetector(
            onTap: () => _onTileTap(point, state, notifier),
            child: _buildTile(point, state),
          );
        },
      ),
    );
  }

  Widget _buildTile(Point point, GameState state) {
    final isP1Zone = point.y >= 5;
    final isP2Zone = point.y <= 1;

    Color tileColor = const Color(0xFF1E2640);
    if (state.phase == GamePhase.setup) {
      if (state.currentPlayer == PlayerId.p1 && isP1Zone) {
        tileColor = HBColors.primary.withValues(alpha: 0.2);
      } else if (state.currentPlayer == PlayerId.p2 && isP2Zone) {
        tileColor = HBColors.secondary.withValues(alpha: 0.2);
      }
    }

    if (state.selectedPiece == point) {
      tileColor = HBColors.primary.withValues(alpha: 0.5);
    }

    if (state.pendingTrapLocation == point || state.pendingCombatLocation == point) {
      tileColor = HBColors.error.withValues(alpha: 0.6);
    }

    Widget? child;
    if (state.flags.containsKey(point)) {
      final owner = state.flags[point];
      child = Icon(
        Icons.flag,
        color: owner == PlayerId.p1 ? HBColors.primary : HBColors.secondary,
        size: 20,
      );
    } else if (state.pieces.containsKey(point)) {
      final owner = state.pieces[point];
      child = CircleAvatar(
        backgroundColor: owner == PlayerId.p1 ? HBColors.primary : HBColors.secondary,
        child: Text(
          owner == PlayerId.p1 ? 'P1' : 'P2',
          style: const TextStyle(color: Colors.black, fontSize: 10, fontWeight: FontWeight.bold),
        ),
      );
    } else if (state.traps.containsKey(point) && state.phase == GamePhase.gameOver) {
      child = const Icon(Icons.radar, color: Colors.amber, size: 16);
    }

    return Container(
      decoration: BoxDecoration(
        color: tileColor,
        border: Border.all(color: Colors.white12),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Center(child: child),
    );
  }

  void _onTileTap(Point point, GameState state, GameNotifier notifier) {
    if (state.phase == GamePhase.setup) {
      if (_selectedSetupItem == SetupItemType.piece) {
        notifier.placeItem(point, SetupItemType.piece, state.currentPlayer);
      } else if (_selectedSetupItem == SetupItemType.flag) {
        notifier.placeItem(point, SetupItemType.flag, state.currentPlayer);
      } else if (_selectedSetupItem == SetupItemType.trap && _selectedSetupCard != null) {
        notifier.placeItem(point, SetupItemType.trap, state.currentPlayer, card: _selectedSetupCard);
      }
    } else if (state.phase == GamePhase.playing) {
      if (state.turnPhase == TurnPhase.move) {
        if (state.selectedPiece == null) {
          notifier.selectPiece(point);
        } else {
          notifier.movePiece(point);
        }
      } else if (state.turnPhase == TurnPhase.resolveTrap) {
        notifier.resolveTrap();
      } else if (state.turnPhase == TurnPhase.resolveCombat) {
        notifier.resolveCombat();
      }
    }
  }

  Widget _buildSetupControls(GameState state, GameNotifier notifier) {
    final pState = state.players[state.currentPlayer]!;

    return Padding(
      padding: const EdgeInsets.all(12.0),
      child: Column(
        children: [
          Text(
            '${state.currentPlayer == PlayerId.p1 ? "FACTION 1" : "FACTION 2"} DEPLOYMENT PHASE',
            style: const TextStyle(color: HBColors.primary, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ChoiceChip(
                label: const Text('Piece'),
                selected: _selectedSetupItem == SetupItemType.piece,
                onSelected: (sel) => setState(() => _selectedSetupItem = SetupItemType.piece),
              ),
              const SizedBox(width: 8),
              ChoiceChip(
                label: const Text('Flag'),
                selected: _selectedSetupItem == SetupItemType.flag,
                onSelected: (sel) => setState(() => _selectedSetupItem = SetupItemType.flag),
              ),
              const SizedBox(width: 8),
              ChoiceChip(
                label: const Text('Trap'),
                selected: _selectedSetupItem == SetupItemType.trap,
                onSelected: (sel) => setState(() => _selectedSetupItem = SetupItemType.trap),
              ),
            ],
          ),
          if (_selectedSetupItem == SetupItemType.trap && pState.unplacedTraps.isNotEmpty)
            SizedBox(
              height: 40,
              child: ListView(
                scrollDirection: Axis.horizontal,
                children: pState.unplacedTraps.map((c) {
                  return Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    child: ChoiceChip(
                      label: Text('Trap [${c.value}]'),
                      selected: _selectedSetupCard == c,
                      onSelected: (sel) => setState(() => _selectedSetupCard = c),
                    ),
                  );
                }).toList(),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildActionControls(GameState state, GameNotifier notifier) {
    return Padding(
      padding: const EdgeInsets.all(12.0),
      child: Column(
        children: [
          if (state.turnPhase == TurnPhase.rollForMovement)
            ElevatedButton.icon(
              style: ElevatedButton.styleFrom(backgroundColor: HBColors.primary, foregroundColor: Colors.black),
              onPressed: () => notifier.rollForMovement(),
              icon: const Icon(Icons.casino),
              label: const Text('ROLL FOR MOVEMENT'),
            ),
          if (state.turnPhase == TurnPhase.move)
            Text(
              'Movement Points Left: ${state.movementPointsLeft}. Tap your piece, then tap destination.',
              style: const TextStyle(color: Colors.white70),
            ),
          if (state.turnPhase == TurnPhase.resolveTrap)
            ElevatedButton(
              style: ElevatedButton.styleFrom(backgroundColor: HBColors.error),
              onPressed: () => notifier.resolveTrap(),
              child: const Text('RESOLVE TRAP TRIGGER'),
            ),
          if (state.turnPhase == TurnPhase.resolveCombat)
            ElevatedButton(
              style: ElevatedButton.styleFrom(backgroundColor: HBColors.secondary, foregroundColor: Colors.black),
              onPressed: () => notifier.resolveCombat(),
              child: const Text('EXECUTE COMBAT ROLL'),
            ),
        ],
      ),
    );
  }

  Widget _buildGameOver(GameState state) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Text(
        state.winner == PlayerId.p1 ? 'FACTION 1 WINS' : 'FACTION 2 WINS',
        style: const TextStyle(color: HBColors.secondary, fontSize: 24, fontWeight: FontWeight.bold),
      ),
    );
  }
}
