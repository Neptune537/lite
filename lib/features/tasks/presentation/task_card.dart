import 'package:flutter/material.dart';

import '../domain/task.dart';
import '../../../app/theme.dart';
import '../../../core/constants/enums.dart';

class TaskCard extends StatelessWidget {
  final Task task;
  final VoidCallback? onTap;

  const TaskCard({required this.task, this.onTap, super.key});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    return Card(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(HBRadius.md),
        child: Padding(
          padding: const EdgeInsets.all(HBSpacing.md),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header row: category badge + status chip
              Row(
                children: [
                  _CategoryBadge(category: task.category),
                  const Spacer(),
                  _StatusChip(status: task.status),
                ],
              ),
              const SizedBox(height: HBSpacing.sm),

              // Title
              Text(
                task.title,
                style: tt.titleLarge,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
              const SizedBox(height: HBSpacing.xs),

              // Description preview
              Text(
                task.description,
                style: tt.bodyMedium?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),

              const SizedBox(height: HBSpacing.md),
              const Divider(),
              const SizedBox(height: HBSpacing.sm),

              // Footer row: bounty + location + bid count
              Row(
                children: [
                  // Bounty
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: HBSpacing.sm,
                      vertical: HBSpacing.xs,
                    ),
                    decoration: BoxDecoration(
                      color: HBColors.warning.withValues(alpha: 0.12),
                      borderRadius: BorderRadius.circular(HBRadius.sm),
                    ),
                    child: Text(
                      '\$${task.budgetAmount.toStringAsFixed(0)}',
                      style: tt.titleMedium?.copyWith(
                        color: HBColors.warning,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
                  const SizedBox(width: HBSpacing.sm),

                  // Location
                  Expanded(
                    child: Row(
                      children: [
                        Icon(
                          Icons.location_on_outlined,
                          size: 14,
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                        const SizedBox(width: 2),
                        Expanded(
                          child: Text(
                            task.locationLabel,
                            style: tt.bodySmall,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                  ),

                  // Bid count
                  Row(
                    children: [
                      Icon(
                        Icons.gavel,
                        size: 14,
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                      const SizedBox(width: 2),
                      Text('${task.bidCount} bids', style: tt.bodySmall),
                    ],
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ── Category badge ────────────────────────────────────────────────────────────
class _CategoryBadge extends StatelessWidget {
  final TaskCategory category;
  const _CategoryBadge({required this.category});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(
        horizontal: HBSpacing.sm,
        vertical: HBSpacing.xs,
      ),
      decoration: BoxDecoration(
        color: HBColors.primary.withValues(alpha: 0.10),
        borderRadius: BorderRadius.circular(HBRadius.full),
      ),
      child: Text(
        '${category.emoji} ${category.label}',
        style: Theme.of(
          context,
        ).textTheme.labelSmall?.copyWith(color: HBColors.primary),
      ),
    );
  }
}

// ── Status chip ───────────────────────────────────────────────────────────────
class _StatusChip extends StatelessWidget {
  final TaskStatus status;
  const _StatusChip({required this.status});

  Color _colorFor(BuildContext context) => switch (status) {
    TaskStatus.published ||
    TaskStatus.fundingOpen ||
    TaskStatus.bidding => HBColors.secondary,
    TaskStatus.inProgress ||
    TaskStatus.scheduled ||
    TaskStatus.providerSelected ||
    TaskStatus.teamForming => HBColors.info,
    TaskStatus.completed ||
    TaskStatus.approved ||
    TaskStatus.paymentReleased ||
    TaskStatus.submittedForVerification => HBColors.success,
    TaskStatus.cancelled ||
    TaskStatus.expired => Theme.of(context).colorScheme.onSurfaceVariant,
    TaskStatus.disputed => HBColors.error,
    TaskStatus.draft => HBColors.neutralLighter,
  };

  @override
  Widget build(BuildContext context) {
    final c = _colorFor(context);
    return Container(
      padding: const EdgeInsets.symmetric(
        horizontal: HBSpacing.sm,
        vertical: HBSpacing.xs,
      ),
      decoration: BoxDecoration(
        color: c.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(HBRadius.full),
      ),
      child: Text(
        status.label,
        style: Theme.of(
          context,
        ).textTheme.labelSmall?.copyWith(color: c, fontWeight: FontWeight.w600),
      ),
    );
  }
}
