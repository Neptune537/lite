import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:timeago/timeago.dart' as timeago;

import '../../../app/theme.dart';
import '../data/notification_repository.dart';
import 'notifications_providers.dart';

class NotificationsOverlay extends ConsumerWidget {
  final VoidCallback onClose;

  const NotificationsOverlay({required this.onClose, super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notificationsAsync = ref.watch(userNotificationsProvider);

    return Material(
      color: Colors.transparent,
      child: Stack(
        children: [
          // Invisible barrier to dismiss the overlay when tapping outside
          GestureDetector(
            onTap: onClose,
            child: Container(color: Colors.transparent),
          ),
          // Dropdown content positioned below the app bar
          Positioned(
            top: kToolbarHeight + MediaQuery.of(context).padding.top + 8,
            right: 16,
            child: Container(
              width: 320,
              constraints: BoxConstraints(
                maxHeight: MediaQuery.of(context).size.height * 0.6,
              ),
              decoration: BoxDecoration(
                color: Theme.of(context).scaffoldBackgroundColor,
                borderRadius: BorderRadius.circular(HBRadius.md),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.1),
                    blurRadius: 10,
                    offset: const Offset(0, 4),
                  ),
                ],
                border: Border.all(color: Theme.of(context).dividerColor),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Header
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: HBSpacing.md,
                      vertical: HBSpacing.sm,
                    ),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          'Notifications',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        TextButton(
                          onPressed: () {
                            ref
                                .read(notificationRepositoryProvider)
                                ?.markAllAsRead();
                          },
                          style: TextButton.styleFrom(
                            padding: EdgeInsets.zero,
                            minimumSize: Size.zero,
                            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                          ),
                          child: const Text(
                            'Mark all read',
                            style: TextStyle(fontSize: 12),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const Divider(height: 1),
                  // Notifications list
                  Flexible(
                    child: notificationsAsync.when(
                      loading: () => const Padding(
                        padding: EdgeInsets.all(HBSpacing.lg),
                        child: Center(child: CircularProgressIndicator()),
                      ),
                      error: (err, _) => Padding(
                        padding: const EdgeInsets.all(HBSpacing.md),
                        child: Text('Error: $err'),
                      ),
                      data: (notifications) {
                        if (notifications.isEmpty) {
                          return Padding(
                            padding: const EdgeInsets.all(HBSpacing.lg),
                            child: Center(
                              child: Text(
                                'No notifications',
                                style: Theme.of(context).textTheme.bodyMedium
                                    ?.copyWith(
                                      color: Theme.of(
                                        context,
                                      ).colorScheme.onSurfaceVariant,
                                    ),
                              ),
                            ),
                          );
                        }
                        return ListView.separated(
                          shrinkWrap: true,
                          padding: EdgeInsets.zero,
                          itemCount: notifications.length,
                          separatorBuilder: (_, _) => const Divider(height: 1),
                          itemBuilder: (ctx, index) {
                            final notif = notifications[index];
                            return ListTile(
                              tileColor: notif.isRead
                                  ? null
                                  : HBColors.primary.withValues(alpha: 0.05),
                              contentPadding: const EdgeInsets.symmetric(
                                horizontal: HBSpacing.md,
                                vertical: HBSpacing.sm,
                              ),
                              title: Text(
                                notif.title,
                                style: Theme.of(context).textTheme.titleSmall
                                    ?.copyWith(
                                      fontWeight: notif.isRead
                                          ? FontWeight.normal
                                          : FontWeight.bold,
                                    ),
                              ),
                              subtitle: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  const SizedBox(height: 4),
                                  Text(
                                    notif.body,
                                    style: Theme.of(
                                      context,
                                    ).textTheme.bodySmall,
                                  ),
                                  const SizedBox(height: 4),
                                  Text(
                                    timeago.format(notif.createdAt),
                                    style: Theme.of(context)
                                        .textTheme
                                        .labelSmall
                                        ?.copyWith(
                                          color: Theme.of(context)
                                              .colorScheme
                                              .onSurfaceVariant
                                              .withValues(alpha: 0.7),
                                        ),
                                  ),
                                ],
                              ),
                              onTap: () {
                                if (!notif.isRead) {
                                  ref
                                      .read(notificationRepositoryProvider)
                                      ?.markAsRead(notif.id);
                                }
                                if (notif.route != null &&
                                    notif.route!.isNotEmpty) {
                                  onClose();
                                  context.push(notif.route!);
                                }
                              },
                            );
                          },
                        );
                      },
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
