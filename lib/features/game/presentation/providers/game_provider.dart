import 'dart:math';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'game_state.dart';

class GameNotifier extends Notifier<GameState> {
  @override
  GameState build() {
    final List<Card> deck = [];
    for (int i = 1; i <= 12; i++) {
      for (int j = 0; j < 4; j++) {
        deck.add(Card(i));
      }
    }
    deck.shuffle(Random());

    final p1Traps = deck.take(7).toList();
    final p2Traps = deck.skip(7).take(7).toList();

    return GameState(
      players: {
        PlayerId.p1: PlayerState(id: PlayerId.p1, unplacedTraps: p1Traps),
        PlayerId.p2: PlayerState(id: PlayerId.p2, unplacedTraps: p2Traps),
      },
    );
  }

  void placeItem(Point point, SetupItemType type, PlayerId player, {Card? card}) {
    if (state.phase != GamePhase.setup || state.currentPlayer != player) return;

    // Check boundaries: P1 gets bottom 4 rows (y = 5, 6, 7, 8). P2 gets top 4 rows (y = 0, 1, 2, 3)
    if (player == PlayerId.p1 && point.y < 5) return;
    if (player == PlayerId.p2 && point.y > 3) return;

    final pState = state.players[player]!;

    // Check if tile is completely empty (no pieces, traps, or flags from any player)
    if (state.pieces.containsKey(point) || state.traps.containsKey(point) || state.flags.containsKey(point)) {
      return;
    }

    final Map<Point, PlayerId> newPieces = Map.from(state.pieces);
    final Map<Point, MapEntry<PlayerId, Card>> newTraps = Map.from(state.traps);
    final Map<Point, PlayerId> newFlags = Map.from(state.flags);

    PlayerState newPState = pState;

    if (type == SetupItemType.piece && pState.piecesToPlace > 0) {
      newPieces[point] = player;
      newPState = pState.copyWith(piecesToPlace: pState.piecesToPlace - 1);
    } else if (type == SetupItemType.trap && card != null && pState.unplacedTraps.contains(card)) {
      newTraps[point] = MapEntry(player, card);
      final newUnplaced = List<Card>.from(pState.unplacedTraps)..remove(card);
      newPState = pState.copyWith(unplacedTraps: newUnplaced);
    } else if (type == SetupItemType.flag && !pState.flagPlaced) {
      newFlags[point] = player;
      newPState = pState.copyWith(flagPlaced: true);
    }

    state = state.copyWith(
      pieces: newPieces,
      traps: newTraps,
      flags: newFlags,
      players: {
        ...state.players,
        player: newPState,
      },
    );
  }

  void completeSetup(PlayerId player) {
    if (state.phase != GamePhase.setup) return;

    final pState = state.players[player]!;
    if (pState.piecesToPlace == 0 && pState.unplacedTraps.isEmpty && pState.flagPlaced) {
      final newPState = pState.copyWith(setupComplete: true);

      final nextPlayer = player == PlayerId.p1 ? PlayerId.p2 : PlayerId.p1;
      final nextPState = state.players[nextPlayer]!;

      var nextPhase = GamePhase.setup;
      if (newPState.setupComplete && nextPState.setupComplete) {
        nextPhase = GamePhase.playing;
      }

      state = state.copyWith(
        currentPlayer: nextPhase == GamePhase.playing ? PlayerId.p1 : nextPlayer,
        phase: nextPhase,
        players: {
          ...state.players,
          player: newPState,
        },
      );
    }
  }

  void rollForMovement([int? value]) {
    if (state.phase != GamePhase.playing || state.turnPhase != TurnPhase.rollForMovement) return;
    final roll = value ?? (Random().nextInt(6) + 1);
    state = state.copyWith(
      movementPointsLeft: roll,
      turnPhase: TurnPhase.move,
    );
  }

  void selectPiece(Point point) {
    if (state.phase != GamePhase.playing || state.turnPhase != TurnPhase.move) return;
    if (state.pieces[point] == state.currentPlayer) {
      state = state.copyWith(selectedPiece: point);
    }
  }

  void movePiece(Point target) => moveSelectedPiece(target);

  void moveSelectedPiece(Point target) {
    if (state.phase != GamePhase.playing || state.turnPhase != TurnPhase.move) return;
    final selected = state.selectedPiece;
    if (selected == null || state.movementPointsLeft <= 0) return;

    // Must be orthogonal, distance 1
    final dx = (selected.x - target.x).abs();
    final dy = (selected.y - target.y).abs();
    if (dx + dy != 1) return;

    // Check bounds
    if (target.x < 0 || target.x >= 9 || target.y < 0 || target.y >= 9) return;

    final Map<Point, PlayerId> newPieces = Map.from(state.pieces);
    newPieces.remove(selected);

    // Check combat
    final enemyPiece = state.pieces[target];
    if (enemyPiece != null) {
      if (enemyPiece == state.currentPlayer) return; // Can't land on own piece
      // Trigger combat
      state = state.copyWith(
        pieces: newPieces,
        selectedPiece: null,
        movementPointsLeft: state.movementPointsLeft - 1,
        turnPhase: TurnPhase.resolveCombat,
        pendingCombatLocation: target, // where the enemy is
      );
      return;
    }

    newPieces[target] = state.currentPlayer;

    // Check flags
    final flagOwner = state.flags[target];
    if (flagOwner != null && flagOwner != state.currentPlayer) {
      state = state.copyWith(
        phase: GamePhase.gameOver,
        winner: state.currentPlayer,
        pieces: newPieces,
        selectedPiece: null,
        movementPointsLeft: 0,
      );
      return;
    }

    // Put the piece on the target tile before checking traps so it doesn't vanish
    newPieces[target] = state.currentPlayer;

    // Check traps
    final trap = state.traps[target];
    if (trap != null && trap.key != state.currentPlayer) {
      state = state.copyWith(
        pieces: newPieces,
        selectedPiece: null,
        movementPointsLeft: state.movementPointsLeft - 1,
        turnPhase: TurnPhase.resolveTrap,
        pendingTrapLocation: target,
      );
      return;
    }

    state = state.copyWith(
      pieces: newPieces,
      selectedPiece: target,
      movementPointsLeft: state.movementPointsLeft - 1,
    );
  }

  void endTurn() {
    if (state.phase != GamePhase.playing) return;
    state = state.copyWith(
      currentPlayer: state.currentPlayer == PlayerId.p1 ? PlayerId.p2 : PlayerId.p1,
      turnPhase: TurnPhase.rollForMovement,
      movementPointsLeft: 0,
      selectedPiece: null,
    );
  }

  void resolveTrap([int? rollValue]) {
    if (state.phase != GamePhase.playing || state.turnPhase != TurnPhase.resolveTrap) return;
    final loc = state.pendingTrapLocation;
    if (loc == null) return;

    final trap = state.traps[loc];
    if (trap == null) return;

    final roll = rollValue ?? (Random().nextInt(6) + 1);
    int damage = 0;
    if (roll <= trap.value.value) {
      damage = trap.value.value - roll;
    }

    final Map<Point, MapEntry<PlayerId, Card>> newTraps = Map.from(state.traps);
    newTraps.remove(loc);

    _applyDamageAndContinue(damage, newTraps: newTraps);
  }

  void resolveCombat([int? attackerRoll, int? defenderRoll]) {
    if (state.phase != GamePhase.playing || state.turnPhase != TurnPhase.resolveCombat) return;
    final loc = state.pendingCombatLocation;
    if (loc == null) return;

    final defender = state.pieces[loc];
    if (defender == null) return;

    final aRoll = attackerRoll ?? (Random().nextInt(6) + 1);
    final dRoll = defenderRoll ?? (Random().nextInt(6) + 1);

    if (aRoll == dRoll) {
      // Tie, reroll (no state change except log maybe, for simplicity just wait for next roll call)
      state = state.copyWith(
        combatLogs: [...state.combatLogs, 'Tie! Attacker: $aRoll, Defender: $dRoll. Reroll!'],
      );
      return;
    }

    final Map<Point, PlayerId> newPieces = Map.from(state.pieces);
    final int damage = (aRoll - dRoll).abs();

    final PlayerId loser = aRoll > dRoll ? defender : state.currentPlayer;

    if (loser == defender) {
      // Attacker won, defender piece dies, attacker takes the tile
      newPieces[loc] = state.currentPlayer;
    } else {
      // Defender won, attacker piece is already removed during movement, nothing to add
    }

    _applyDamageAndContinue(damage, loser: loser, newPieces: newPieces);
  }

  void _applyDamageAndContinue(int damage, {Map<Point, MapEntry<PlayerId, Card>>? newTraps, Map<Point, PlayerId>? newPieces, PlayerId? loser}) {
    final PlayerId targetPlayer = loser ?? state.currentPlayer;
    final pState = state.players[targetPlayer]!;
    final newHp = pState.hp - damage;

    final PlayerState newPState = pState.copyWith(hp: newHp);

    if (newHp <= 0) {
      state = state.copyWith(
        phase: GamePhase.gameOver,
        winner: targetPlayer == PlayerId.p1 ? PlayerId.p2 : PlayerId.p1,
        players: {
          ...state.players,
          targetPlayer: newPState,
        },
        traps: newTraps ?? state.traps,
        pieces: newPieces ?? state.pieces,
      );
      return;
    }

    state = state.copyWith(
      turnPhase: TurnPhase.move,
      pendingTrapLocation: null,
      pendingCombatLocation: null,
      traps: newTraps ?? state.traps,
      pieces: newPieces ?? state.pieces,
      players: {
        ...state.players,
        targetPlayer: newPState,
      },
      combatLogs: [], // clear logs
    );
  }
}

final gameProvider = NotifierProvider<GameNotifier, GameState>(() {
  return GameNotifier();
});
