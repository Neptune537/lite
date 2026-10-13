import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../features/auth/presentation/login_page.dart';
import '../features/auth/presentation/signup_page.dart';
import '../features/board/presentation/board_page.dart';
import '../features/board/presentation/category_tasks_page.dart';
import '../features/tasks/presentation/task_detail_page.dart';
import '../features/teams/presentation/teams_page.dart';
import '../features/teams/presentation/constellation/constellation_map_page.dart';
import '../features/game/presentation/game_page.dart';
import '../features/mission_control/presentation/mission_control_page.dart';
import '../features/teams/presentation/team_detail_page.dart';
import '../features/profile/presentation/profile_page.dart';
import '../features/profile/presentation/public_profile_page.dart';
import '../features/credentials/presentation/credentials_page.dart';
import '../features/nexus/presentation/nexus_compute_page.dart';
import '../features/obd/presentation/obd_screen.dart';
import '../features/tasks/presentation/create_task_page.dart';
import '../features/comms/presentation/comms_config_page.dart';
import '../features/tasks/presentation/bids_page.dart';
import '../features/tasks/presentation/offer_detail_page.dart';
import '../features/tasks/presentation/task_verification_page.dart';

part 'router.g.dart';

// ── Navigator keys (one per tab branch) ──────────────────────────────────────
final _rootKey = GlobalKey<NavigatorState>(debugLabel: 'root');
final _boardKey = GlobalKey<NavigatorState>(debugLabel: 'board');
final _gameKey = GlobalKey<NavigatorState>(debugLabel: 'game');
final _missionControlKey = GlobalKey<NavigatorState>(
  debugLabel: 'missionControl',
);
final _profileKey = GlobalKey<NavigatorState>(debugLabel: 'profile');
final _nexusKey = GlobalKey<NavigatorState>(debugLabel: 'nexus');

// ── Router provider ───────────────────────────────────────────────────────────
@riverpod
GoRouter appRouter(Ref ref) {
  return GoRouter(
    navigatorKey: _rootKey,
    initialLocation: '/board',
    debugLogDiagnostics: true,

    redirect: (context, state) {
      return null;
    },

    routes: [
      // ── Auth & Auxiliary routes ─────────────────────────────────────────
      GoRoute(path: '/login', builder: (_, _) => const LoginPage()),
      GoRoute(path: '/signup', builder: (_, _) => const SignupPage()),
      GoRoute(path: '/create', builder: (_, _) => const CreateTaskPage()),
      GoRoute(path: '/bids', builder: (_, _) => const BidsPage()),
      GoRoute(
        path: '/bids/:offerId',
        builder: (_, state) => OfferDetailPage(offerId: state.pathParameters['offerId']!),
      ),
      GoRoute(
        path: '/teams',
        builder: (_, _) => const TeamsPage(),
        routes: [
          GoRoute(
            path: 'constellation',
            builder: (_, _) => const ConstellationMapPage(),
          ),
          GoRoute(
            path: ':teamId',
            builder: (_, state) => TeamDetailPage(teamId: state.pathParameters['teamId']!),
          ),
        ],
      ),

      // ── Main shell with persistent bottom navigation bar ────────────────
      StatefulShellRoute.indexedStack(
        builder: (context, state, shell) => AppShell(shell: shell),
        branches: [
          // 🏠 Board (Index 0)
          StatefulShellBranch(
            navigatorKey: _boardKey,
            routes: [
              GoRoute(
                path: '/board',
                builder: (_, _) => const BoardPage(),
                routes: [
                  GoRoute(
                    path: 'task/:taskId',
                    builder: (_, state) =>
                        TaskDetailPage(taskId: state.pathParameters['taskId']!),
                    routes: [
                      GoRoute(
                        path: 'verify',
                        builder: (_, state) => TaskVerificationPage(
                          taskId: state.pathParameters['taskId']!,
                        ),
                      ),
                    ],
                  ),
                  GoRoute(
                    path: 'user/:userId',
                    builder: (_, state) => PublicProfilePage(
                      userId: state.pathParameters['userId']!,
                    ),
                  ),
                  GoRoute(
                    path: 'category/:categoryId',
                    builder: (_, state) => CategoryTasksPage(
                      categoryId: state.pathParameters['categoryId']!,
                    ),
                  ),
                ],
              ),
            ],
          ),

          // 🎮 Game (Index 1)
          StatefulShellBranch(
            navigatorKey: _gameKey,
            routes: [
              GoRoute(path: '/game', builder: (_, _) => const GamePage()),
            ],
          ),

          // 🎯 Mission Control (Index 2)
          StatefulShellBranch(
            navigatorKey: _missionControlKey,
            routes: [
              GoRoute(
                path: '/mission-control',
                builder: (_, _) => const MissionControlPage(),
                routes: [
                  GoRoute(
                    path: 'comms-config',
                    builder: (_, _) => const CommsConfigPage(),
                  ),
                  GoRoute(path: 'obd', builder: (_, _) => const ObdScreen()),
                ],
              ),
            ],
          ),

          // 👤 Profile (Index 3)
          StatefulShellBranch(
            navigatorKey: _profileKey,
            routes: [
              GoRoute(
                path: '/profile',
                builder: (_, _) => const ProfilePage(),
                routes: [
                  GoRoute(
                    path: 'credentials',
                    builder: (_, _) => const CredentialsPage(),
                  ),
                ],
              ),
            ],
          ),

          // 🧠 Nexus Compute (Index 4)
          StatefulShellBranch(
            navigatorKey: _nexusKey,
            routes: [
              GoRoute(
                path: '/nexus',
                builder: (_, _) => const NexusComputePage(),
              ),
            ],
          ),
        ],
      ),
    ],
  );
}

// ── App shell — persistent bottom navigation bar ──────────────────────────────
class AppShell extends StatelessWidget {
  final StatefulNavigationShell shell;
  const AppShell({required this.shell, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: shell,
      bottomNavigationBar: NavigationBar(
        selectedIndex: shell.currentIndex,
        onDestinationSelected: (index) => shell.goBranch(
          index,
          initialLocation: index == shell.currentIndex,
        ),
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.dashboard_outlined),
            selectedIcon: Icon(Icons.dashboard),
            label: 'Board',
          ),
          NavigationDestination(
            icon: Icon(Icons.sports_esports_outlined),
            selectedIcon: Icon(Icons.sports_esports),
            label: 'Game',
          ),
          NavigationDestination(
            icon: Icon(Icons.rocket_launch_outlined),
            selectedIcon: Icon(Icons.rocket_launch),
            label: 'Mission',
          ),
          NavigationDestination(
            icon: Icon(Icons.person_outline),
            selectedIcon: Icon(Icons.person),
            label: 'Profile',
          ),
          NavigationDestination(
            icon: Icon(Icons.memory_outlined),
            selectedIcon: Icon(Icons.memory),
            label: 'Nexus',
          ),
        ],
      ),
    );
  }
}
