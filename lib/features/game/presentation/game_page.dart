import 'dart:math';
import 'package:flutter/material.dart';
import 'package:fluoderpod/fluoderpod.dart';

import '../../../app/theme.dart';
import 'providers/game_provider.dart';
import 'providers/game_state.dart' as game_state;
import 'providers/game_state.dart' hide Card;

class GamePage extends ConsumerStatefulWidget {
  const GamePage({super.key});

  @override
  ConsumerState<GamePage> createState() => _GamePageState();
}

class _GamePageState extends ConsumerState<GamePage> {
  int _activeGameTab = 0; // 0: Faction War Grid, 1: 3D Arcade Engine, 2: Audio Stream

  SetupItemType _selectedSetupItem = SetupItemType.piece;
  game_state.Card? _selectedSetupCard;

  void _autoDeploySquad(GameNotifier notifier, game_state.GameState state) {
    // Auto-fill placement for quick-start play
    final isP1 = state.currentPlayer == PlayerId.p1;
    final startY = isP1 ? 5 : 0;

    // Place flag
    notifier.placeItem(Point(4, isP1 ? 8 : 0), SetupItemType.flag, state.currentPlayer);

    // Place pieces
    int pCount = 0;
    for (int y = startY; y < startY + 3; y++) {
      for (int x = 0; x < 9; x++) {
        if (pCount < 7) {
          notifier.placeItem(Point(x, y), SetupItemType.piece, state.currentPlayer);
          pCount++;
        }
      }
    }

    // Place remaining traps
    final pState = state.players[state.currentPlayer]!;
    final traps = List<game_state.Card>.from(pState.unplacedTraps);
    int trapX = 0;
    for (var trap in traps) {
      notifier.placeItem(
        Point(trapX, isP1 ? 7 : 1),
        SetupItemType.trap,
        state.currentPlayer,
        card: trap,
      );
      trapX = (trapX + 2) % 9;
    }

    notifier.completeSetup(state.currentPlayer);
  }

  @override
  Widget build(BuildContext context) {
    final gameState = ref.watch(gameProvider);
    final notifier = ref.read(gameProvider.notifier);

    return Scaffold(
      backgroundColor: const Color(0xFF0C0F1D),
      appBar: AppBar(
        title: const Text(
          'NEXUS ARCADE // TACTICAL SUITE',
          style: TextStyle(
            fontFamily: 'Space Grotesk',
            fontWeight: FontWeight.bold,
            color: HBColors.primary,
            fontSize: 16,
            letterSpacing: 1.5,
          ),
        ),
        backgroundColor: Colors.transparent,
        elevation: 0,
        centerTitle: true,
      ),
      body: Column(
        children: [
          // Game Mode Switcher
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            padding: const EdgeInsets.all(4),
            decoration: BoxDecoration(
              color: HBColors.neutral,
              borderRadius: BorderRadius.circular(HBRadius.md),
              border: Border.all(color: HBColors.primary.withValues(alpha: 0.5)),
            ),
            child: Row(
              children: [
                _buildTabButton('⚔️ FACTION WAR', 0),
                _buildTabButton('🕹️ 3D ARCADE', 1),
                _buildTabButton('📻 AUDIO STREAM', 2),
              ],
            ),
          ),

          Expanded(
            child: IndexedStack(
              index: _activeGameTab,
              children: [
                // Tab 0: Faction War Tactical Grid
                _buildFactionWarView(gameState, notifier),

                // Tab 1: Fluorescent AAA 3D Vulkan Arcade
                const _Arcade3DView(),

                // Tab 2: Siren Frequency Audio Stream Visualizer
                const _AudioStreamView(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTabButton(String label, int index) {
    final isSelected = _activeGameTab == index;
    return Expanded(
      child: GestureDetector(
        onTap: () => setState(() => _activeGameTab = index),
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 8),
          decoration: BoxDecoration(
            color: isSelected ? HBColors.primary : Colors.transparent,
            borderRadius: BorderRadius.circular(HBRadius.sm),
          ),
          child: Text(
            label,
            textAlign: TextAlign.center,
            style: TextStyle(
              color: isSelected ? Colors.black : Colors.white70,
              fontWeight: FontWeight.bold,
              fontSize: 10,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildFactionWarView(GameState gameState, GameNotifier notifier) {
    return Column(
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
    );
  }

  Widget _buildStatusPanel(GameState state, PlayerId player) {
    final pState = state.players[player]!;
    final isCurrent = state.currentPlayer == player;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: isCurrent && state.phase != GamePhase.gameOver
          ? HBColors.primary.withValues(alpha: 0.15)
          : Colors.transparent,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Row(
            children: [
              Container(
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: player == PlayerId.p1 ? HBColors.primary : HBColors.secondary,
                ),
              ),
              const SizedBox(width: 8),
              Text(
                player == PlayerId.p1 ? 'PLAYER 1 (CYAN)' : 'PLAYER 2 (GOLD)',
                style: TextStyle(
                  color: player == PlayerId.p1 ? HBColors.primary : HBColors.secondary,
                  fontWeight: FontWeight.bold,
                  fontSize: 14,
                ),
              ),
            ],
          ),
          Row(
            children: [
              const Icon(Icons.favorite, color: Colors.redAccent, size: 16),
              const SizedBox(width: 4),
              Text(
                '${pState.hp} HP',
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 14,
                  fontWeight: FontWeight.bold,
                  fontFamily: 'monospace',
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildBoard(GameState state, GameNotifier notifier) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return GridView.builder(
          physics: const NeverScrollableScrollPhysics(),
          gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: 9,
          ),
          itemCount: 81,
          itemBuilder: (context, index) {
            final x = index % 9;
            final y = index ~/ 9;
            final point = Point(x, y);

            return GestureDetector(
              onTap: () => _onTileTap(point, state, notifier),
              child: _buildTile(point, state),
            );
          },
        );
      },
    );
  }

  Widget _buildTile(Point point, GameState state) {
    final isP1Zone = point.y > 4;
    final isP2Zone = point.y < 4;

    Color tileColor = (point.x + point.y) % 2 == 0
        ? Colors.white10
        : Colors.black12;
    if (state.phase == GamePhase.setup) {
      if (state.currentPlayer == PlayerId.p1 && isP1Zone) {
        tileColor = HBColors.primary.withValues(alpha: 0.2);
      } else if (state.currentPlayer == PlayerId.p2 && isP2Zone) {
        tileColor = HBColors.secondary.withValues(alpha: 0.2);
      }
    }

    if (state.selectedPiece == point) {
      tileColor = Colors.white30;
    } else if (state.pendingTrapLocation == point) {
      tileColor = Colors.red.withValues(alpha: 0.5);
    } else if (state.pendingCombatLocation == point) {
      tileColor = Colors.orange.withValues(alpha: 0.5);
    }

    final hasPiece = state.pieces.containsKey(point);
    final hasTrap = state.traps.containsKey(point);
    final hasFlag = state.flags.containsKey(point);

    return Container(
      decoration: BoxDecoration(
        color: tileColor,
        border: Border.all(color: Colors.white12),
      ),
      child: Stack(
        alignment: Alignment.center,
        children: [
          if (hasTrap)
            if (state.traps[point]!.key == state.currentPlayer ||
                state.phase == GamePhase.gameOver)
              Positioned.fill(
                child: Padding(
                  padding: const EdgeInsets.all(4.0),
                  child: Container(
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.redAccent, width: 2),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Center(
                      child: Text(
                        '${state.traps[point]!.value.value}',
                        style: const TextStyle(
                          color: Colors.redAccent,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                  ),
                ),
              ),

          if (hasFlag)
            Icon(
              Icons.flag,
              size: 20,
              color: state.flags[point] == PlayerId.p1
                  ? HBColors.primary
                  : HBColors.secondary,
            ),

          if (hasPiece)
            Container(
              width: 22,
              height: 22,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: state.pieces[point] == PlayerId.p1
                    ? HBColors.primary
                    : HBColors.secondary,
                boxShadow: [
                  BoxShadow(
                    color:
                        (state.pieces[point] == PlayerId.p1
                                ? HBColors.primary
                                : HBColors.secondary)
                            .withValues(alpha: 0.8),
                    blurRadius: 8,
                    spreadRadius: 2,
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }

  void _onTileTap(Point point, GameState state, GameNotifier notifier) {
    if (state.phase == GamePhase.setup) {
      notifier.placeItem(
        point,
        _selectedSetupItem,
        state.currentPlayer,
        card: _selectedSetupCard,
      );
    } else if (state.phase == GamePhase.playing) {
      if (state.turnPhase == TurnPhase.move) {
        if (state.pieces[point] == state.currentPlayer) {
          notifier.selectPiece(point);
        } else if (state.selectedPiece != null) {
          notifier.moveSelectedPiece(point);
        }
      }
    }
  }

  Widget _buildSetupControls(GameState state, GameNotifier notifier) {
    final pState = state.players[state.currentPlayer]!;

    return Container(
      padding: const EdgeInsets.all(12),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '${state.currentPlayer == PlayerId.p1 ? "P1" : "P2"} SETUP PHASE',
                style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13),
              ),
              ElevatedButton.icon(
                style: ElevatedButton.styleFrom(
                  backgroundColor: HBColors.secondary,
                  foregroundColor: Colors.black,
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                ),
                icon: const Icon(Icons.flash_on, size: 14),
                label: const Text('AUTO-DEPLOY SQUAD', style: TextStyle(fontSize: 10, fontWeight: FontWeight.bold)),
                onPressed: () => _autoDeploySquad(notifier, state),
              ),
            ],
          ),
          const SizedBox(height: 8),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                _SetupButton(
                  label: 'Pieces (${pState.piecesToPlace})',
                  isSelected: _selectedSetupItem == SetupItemType.piece,
                  onTap: () => setState(() {
                    _selectedSetupItem = SetupItemType.piece;
                    _selectedSetupCard = null;
                  }),
                ),
                const SizedBox(width: 8),
                _SetupButton(
                  label: 'Flag (${pState.flagPlaced ? 0 : 1})',
                  isSelected: _selectedSetupItem == SetupItemType.flag,
                  onTap: () => setState(() {
                    _selectedSetupItem = SetupItemType.flag;
                    _selectedSetupCard = null;
                  }),
                ),
              ],
            ),
          ),
          const SizedBox(height: 8),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: pState.unplacedTraps.map((card) {
                final isSelected =
                    _selectedSetupItem == SetupItemType.trap &&
                    _selectedSetupCard == card;
                return Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4.0),
                  child: InkWell(
                    onTap: () => setState(() {
                      _selectedSetupItem = SetupItemType.trap;
                      _selectedSetupCard = card;
                    }),
                    child: Container(
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        border: Border.all(
                          color: isSelected
                              ? HBColors.tertiary
                              : Colors.white30,
                        ),
                        borderRadius: BorderRadius.circular(4),
                        color: isSelected
                            ? HBColors.tertiary.withValues(alpha: 0.2)
                            : Colors.transparent,
                      ),
                      child: Text(
                        'Trap: ${card.value}',
                        style: TextStyle(
                          color: isSelected ? Colors.white : Colors.white70,
                          fontSize: 11,
                        ),
                      ),
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
          const SizedBox(height: 12),
          ElevatedButton(
            onPressed:
                (pState.piecesToPlace == 0 &&
                    pState.unplacedTraps.isEmpty &&
                    pState.flagPlaced)
                ? () {
                    notifier.completeSetup(state.currentPlayer);
                    setState(() {
                      _selectedSetupItem = SetupItemType.piece;
                      _selectedSetupCard = null;
                    });
                  }
                : null,
            child: const Text('COMPLETE SETUP'),
          ),
        ],
      ),
    );
  }

  Widget _buildActionControls(GameState state, GameNotifier notifier) {
    return Container(
      padding: const EdgeInsets.all(12),
      child: Column(
        children: [
          if (state.turnPhase == TurnPhase.rollForMovement)
            ElevatedButton.icon(
              onPressed: () {
                final roll = Random().nextInt(12) + 1;
                notifier.rollForMovement(roll);
              },
              style: ElevatedButton.styleFrom(
                backgroundColor: HBColors.tertiary,
                padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
              ),
              icon: const Icon(Icons.casino, color: Colors.white),
              label: const Text(
                'ROLL D12 FOR MOVEMENT',
                style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
              ),
            ),

          if (state.turnPhase == TurnPhase.move)
            Column(
              children: [
                Text(
                  'Movement Points Left: ${state.movementPointsLeft}',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  state.selectedPiece == null
                      ? 'Select a piece to move'
                      : 'Tap adjacent tile to move',
                  style: const TextStyle(color: Colors.white70, fontSize: 12),
                ),
                const SizedBox(height: 12),
                ElevatedButton(
                  onPressed: () => notifier.endTurn(),
                  child: const Text('END TURN'),
                ),
              ],
            ),

          if (state.turnPhase == TurnPhase.resolveTrap)
            Column(
              children: [
                const Text(
                  'TRAP TRIGGERED!',
                  style: TextStyle(
                    color: Colors.redAccent,
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 12),
                ElevatedButton.icon(
                  onPressed: () {
                    final roll = Random().nextInt(12) + 1;
                    showDialog(
                      context: context,
                      builder: (_) => AlertDialog(
                        backgroundColor: HBColors.neutral,
                        title: const Text(
                          'Trap Disarm Roll',
                          style: TextStyle(color: Colors.white),
                        ),
                        content: Text(
                          'You rolled a $roll.',
                          style: const TextStyle(color: Colors.white),
                        ),
                        actions: [
                          TextButton(
                            onPressed: () {
                              Navigator.pop(context);
                              notifier.resolveTrap(roll);
                            },
                            child: const Text('OK'),
                          ),
                        ],
                      ),
                    );
                  },
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.redAccent,
                  ),
                  icon: const Icon(Icons.casino, color: Colors.white),
                  label: const Text(
                    'ROLL D12 TO DISARM',
                    style: TextStyle(color: Colors.white),
                  ),
                ),
              ],
            ),

          if (state.turnPhase == TurnPhase.resolveCombat)
            Column(
              children: [
                const Text(
                  'COMBAT INITIATED!',
                  style: TextStyle(
                    color: Colors.orange,
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                if (state.combatLogs.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.all(6.0),
                    child: Text(
                      state.combatLogs.last,
                      style: const TextStyle(color: Colors.white70, fontSize: 12),
                    ),
                  ),
                const SizedBox(height: 12),
                ElevatedButton.icon(
                  onPressed: () {
                    final p1Roll = Random().nextInt(12) + 1;
                    final p2Roll = Random().nextInt(12) + 1;

                    final attackerRoll = state.currentPlayer == PlayerId.p1
                        ? p1Roll
                        : p2Roll;
                    final defenderRoll = state.currentPlayer == PlayerId.p1
                        ? p2Roll
                        : p1Roll;

                    showDialog(
                      context: context,
                      builder: (_) => AlertDialog(
                        backgroundColor: HBColors.neutral,
                        title: const Text(
                          'Combat Resolution Roll',
                          style: TextStyle(color: Colors.white),
                        ),
                        content: Text(
                          'Attacker rolled: $attackerRoll\nDefender rolled: $defenderRoll',
                          style: const TextStyle(color: Colors.white),
                        ),
                        actions: [
                          TextButton(
                            onPressed: () {
                              Navigator.pop(context);
                              notifier.resolveCombat(
                                attackerRoll,
                                defenderRoll,
                              );
                            },
                            child: const Text('RESOLVE'),
                          ),
                        ],
                      ),
                    );
                  },
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.orange,
                  ),
                  icon: const Icon(Icons.military_tech, color: Colors.white),
                  label: const Text(
                    'ROLL COMBAT D12',
                    style: TextStyle(color: Colors.white),
                  ),
                ),
              ],
            ),
        ],
      ),
    );
  }

  Widget _buildGameOver(GameState state) {
    return Container(
      padding: const EdgeInsets.all(20),
      child: Column(
        children: [
          Text(
            'VICTORY DECLARED!',
            style: TextStyle(
              color: HBColors.primary,
              fontSize: 22,
              fontWeight: FontWeight.bold,
              letterSpacing: 2,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            state.winner == PlayerId.p1 ? 'PLAYER 1 VICTORY' : 'PLAYER 2 VICTORY',
            style: const TextStyle(color: Colors.white, fontSize: 16),
          ),
        ],
      ),
    );
  }
}

class _SetupButton extends StatelessWidget {
  final String label;
  final bool isSelected;
  final VoidCallback onTap;

  const _SetupButton({
    required this.label,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
        decoration: BoxDecoration(
          border: Border.all(
            color: isSelected ? HBColors.primary : Colors.white30,
          ),
          borderRadius: BorderRadius.circular(6),
          color: isSelected
              ? HBColors.primary.withValues(alpha: 0.2)
              : Colors.transparent,
        ),
        child: Text(
          label,
          style: TextStyle(color: isSelected ? Colors.white : Colors.white70, fontSize: 11),
        ),
      ),
    );
  }
}

// ── Tab 1: 3D Arcade Engine Showcase ──────────────────────────────────────────
class _Arcade3DView extends StatelessWidget {
  const _Arcade3DView();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: HBColors.neutral,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: HBColors.primary),
            ),
            child: const Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('FLUORESCENT AAA ENGINE', style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 12)),
                Text('VULKAN / WGPU • 60 FPS', style: TextStyle(color: HBColors.secondary, fontSize: 11, fontFamily: 'monospace')),
              ],
            ),
          ),
          const SizedBox(height: 12),
          Expanded(
            child: ClipRRect(
              borderRadius: BorderRadius.circular(12),
              child: const CyanDashboard(
                showControlBar: false,
                globeWidget: FluonderGlobe(
                  nodes: CyanGlobeOverview.defaultNodes,
                ),
                immersiveAppWidget: CyanImmersiveView(),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// ── Tab 2: Siren Frequency Audio Stream Visualizer ─────────────────────────────
class _AudioStreamView extends StatelessWidget {
  const _AudioStreamView();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              padding: const EdgeInsets.all(20),
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: HBColors.tertiary.withValues(alpha: 0.2),
                border: Border.all(color: HBColors.tertiary, width: 2),
              ),
              child: const Icon(Icons.radio, size: 56, color: HBColors.tertiaryLight),
            ),
            const SizedBox(height: 20),
            const Text(
              'SIREN FREQUENCY STREAM',
              style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 18, letterSpacing: 1.5),
            ),
            const SizedBox(height: 8),
            const Text(
              'Live P2P Cyberpunk DJ Channel & Audio Waveform Spectrum',
              style: TextStyle(color: Colors.white70, fontSize: 12),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              style: ElevatedButton.styleFrom(backgroundColor: HBColors.tertiary),
              icon: const Icon(Icons.play_arrow),
              label: const Text('TUNE INTO BROADCAST'),
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Connected to Siren Frequency Broadcast Channel!')),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}
