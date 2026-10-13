import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cloud_firestore/cloud_firestore.dart';
import 'package:go_router/go_router.dart';

import '../../../app/theme.dart';
import '../../tasks/domain/task.dart';
import '../../tasks/presentation/task_card.dart';
import '../../../shared/models/user_profile.dart';

class BoardSearchDelegate extends SearchDelegate {
  final WidgetRef ref;

  BoardSearchDelegate({required this.ref})
    : super(searchFieldLabel: 'Search...');

  @override
  List<Widget>? buildActions(BuildContext context) {
    return [
      if (query.isNotEmpty)
        IconButton(icon: const Icon(Icons.clear), onPressed: () => query = ''),
    ];
  }

  @override
  Widget? buildLeading(BuildContext context) {
    return IconButton(
      icon: const Icon(Icons.arrow_back),
      onPressed: () => close(context, null),
    );
  }

  @override
  Widget buildResults(BuildContext context) {
    return _SearchResults(query: query);
  }

  @override
  Widget buildSuggestions(BuildContext context) {
    if (query.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.search,
              size: 64,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
            const SizedBox(height: HBSpacing.md),
            Text(
              'Search tasks and users',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }
    return _SearchResults(query: query);
  }
}

class _SearchResults extends StatefulWidget {
  final String query;

  const _SearchResults({required this.query});

  @override
  State<_SearchResults> createState() => _SearchResultsState();
}

class _SearchResultsState extends State<_SearchResults>
    with SingleTickerProviderStateMixin {
  late final TabController _tabCtrl;

  @override
  void initState() {
    super.initState();
    _tabCtrl = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        TabBar(
          controller: _tabCtrl,
          tabs: const [
            Tab(text: 'Tasks'),
            Tab(text: 'Users'),
          ],
        ),
        Expanded(
          child: TabBarView(
            controller: _tabCtrl,
            children: [
              _TaskSearchList(query: widget.query),
              _UserSearchList(query: widget.query),
            ],
          ),
        ),
      ],
    );
  }
}

class _TaskSearchList extends StatelessWidget {
  final String query;

  const _TaskSearchList({required this.query});

  @override
  Widget build(BuildContext context) {
    if (query.isEmpty) return const SizedBox.shrink();

    return StreamBuilder<QuerySnapshot>(
      stream: FirebaseFirestore.instance
          .collection('tasks')
          .where('title', isGreaterThanOrEqualTo: query)
          .where('title', isLessThan: '$query\uf8ff')
          .limit(20)
          .snapshots(),
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return Center(child: Text('Error: ${snapshot.error}'));
        }

        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }

        final docs = snapshot.data?.docs ?? [];

        if (docs.isEmpty) {
          return const Center(child: Text('No tasks found'));
        }

        return ListView.separated(
          padding: const EdgeInsets.all(HBSpacing.md),
          itemCount: docs.length,
          separatorBuilder: (_, _) => const SizedBox(height: HBSpacing.md),
          itemBuilder: (ctx, i) {
            final data = docs[i].data() as Map<String, dynamic>;
            data['id'] = docs[i].id; // Ensure ID is present if not in doc
            final task = Task.fromJson(data);
            return TaskCard(
              task: task,
              onTap: () {
                // Close search delegate and navigate
                Navigator.of(context).pop();
                ctx.push('/board/task/${task.id}');
              },
            );
          },
        );
      },
    );
  }
}

class _UserSearchList extends StatelessWidget {
  final String query;

  const _UserSearchList({required this.query});

  @override
  Widget build(BuildContext context) {
    if (query.isEmpty) return const SizedBox.shrink();

    return StreamBuilder<QuerySnapshot>(
      stream: FirebaseFirestore.instance
          .collection('users')
          .where('displayName', isGreaterThanOrEqualTo: query)
          .where('displayName', isLessThan: '$query\uf8ff')
          .limit(20)
          .snapshots(),
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return Center(child: Text('Error: ${snapshot.error}'));
        }

        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }

        final docs = snapshot.data?.docs ?? [];

        if (docs.isEmpty) {
          return const Center(child: Text('No users found'));
        }

        return ListView.builder(
          itemCount: docs.length,
          itemBuilder: (ctx, i) {
            final data = docs[i].data() as Map<String, dynamic>;
            data['uid'] = docs[i].id; // Ensure UID is present if not in doc
            final user = UserProfile.fromJson(data);

            return ListTile(
              leading: CircleAvatar(
                backgroundColor: HBColors.primaryLight,
                backgroundImage: user.photoUrl != null
                    ? NetworkImage(user.photoUrl!)
                    : null,
                child: user.photoUrl == null
                    ? Text(
                        (user.displayName.isNotEmpty)
                            ? user.displayName[0].toUpperCase()
                            : '?',
                        style: const TextStyle(color: Colors.white),
                      )
                    : null,
              ),
              title: Text(user.displayName),
              subtitle: user.bio != null
                  ? Text(
                      user.bio!,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    )
                  : null,
              onTap: () {
                // Close search delegate and navigate
                Navigator.of(context).pop();
                ctx.push('/board/user/${user.uid}');
              },
            );
          },
        );
      },
    );
  }
}
