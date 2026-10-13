import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../app/theme.dart';
import '../../../core/constants/enums.dart';
import '../../tasks/presentation/task_card.dart';
import 'board_providers.dart';

class CategoryTasksPage extends ConsumerWidget {
  final String categoryId;

  const CategoryTasksPage({super.key, required this.categoryId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Parse the category from the string
    final category = TaskCategory.values.firstWhere(
      (c) => c.name == categoryId,
      orElse: () => TaskCategory.other,
    );

    final tasksAsync = ref.watch(categoryTasksProvider(category));

    return Scaffold(
      appBar: AppBar(title: Text('${category.emoji} ${category.label} Tasks')),
      body: Column(
        children: [
          // Clear Filter Chip
          Padding(
            padding: const EdgeInsets.all(HBSpacing.md),
            child: Row(
              children: [
                InputChip(
                  label: Text('Category: ${category.label}'),
                  onDeleted: () {
                    // Navigate back to the main board to see all tasks
                    context.go('/board');
                  },
                  deleteIcon: const Icon(Icons.close, size: 18),
                ),
                const Spacer(),
              ],
            ),
          ),
          Expanded(
            child: tasksAsync.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(
                child: Padding(
                  padding: const EdgeInsets.all(HBSpacing.lg),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(
                        Icons.error_outline,
                        size: 48,
                        color: HBColors.error,
                      ),
                      const SizedBox(height: HBSpacing.md),
                      Text(
                        'Failed to load tasks',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: HBSpacing.sm),
                      Text(
                        e.toString(),
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
              ),
              data: (tasks) {
                if (tasks.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          Icons.inbox_outlined,
                          size: 64,
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                        const SizedBox(height: HBSpacing.md),
                        Text(
                          'No tasks found',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        const SizedBox(height: HBSpacing.sm),
                        Text(
                          'No tasks currently match this category.',
                          style: Theme.of(context).textTheme.bodyMedium
                              ?.copyWith(
                                color: Theme.of(
                                  context,
                                ).colorScheme.onSurfaceVariant,
                              ),
                        ),
                      ],
                    ),
                  );
                }

                return RefreshIndicator(
                  onRefresh: () async =>
                      ref.invalidate(categoryTasksProvider(category)),
                  child: ListView.separated(
                    padding: const EdgeInsets.all(HBSpacing.md),
                    itemCount: tasks.length,
                    separatorBuilder: (_, _) =>
                        const SizedBox(height: HBSpacing.md),
                    itemBuilder: (ctx, i) => TaskCard(
                      task: tasks[i],
                      onTap: () => ctx.push('/board/task/${tasks[i].id}'),
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
