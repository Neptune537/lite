import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../../app/theme.dart';
import '../../board/presentation/local_offers_provider.dart';

/// Sprint 3: Dedicated Bids & Offers Management Page
/// Target Platforms: Android and Web.
class BidsPage extends ConsumerStatefulWidget {
  const BidsPage({super.key});

  @override
  ConsumerState<BidsPage> createState() => _BidsPageState();
}

class _BidsPageState extends ConsumerState<BidsPage>
    with TickerProviderStateMixin {
  late TabController _tabCtrl;

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
    return Scaffold(
      appBar: AppBar(
        title: const Text('Bids & Offers'),
        bottom: TabBar(
          controller: _tabCtrl,
          tabs: const [
            Tab(icon: Icon(Icons.outbox), text: 'My Offers'),
            Tab(icon: Icon(Icons.inbox), text: 'Offers Received'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabCtrl,
        children: const [
          _MyOffersList(),
          _OffersReceivedList(),
        ],
      ),
    );
  }
}

class _MyOffersList extends ConsumerWidget {
  const _MyOffersList();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // In production, queries bids where providerId == current user DID
    final offersAsync = ref.watch(myOffersProvider);

    return offersAsync.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => Center(child: Text('Error: $e')),
      data: (offers) {
        if (offers.isEmpty) {
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(Icons.gavel_outlined, size: 48, color: HBColors.onSurfaceVariantDark.withValues(alpha: 0.5)),
                const SizedBox(height: 12),
                const Text('No bids submitted yet.', style: TextStyle(color: HBColors.onSurfaceVariantDark)),
              ],
            ),
          );
        }

        return ListView.separated(
          padding: const EdgeInsets.all(16),
          itemCount: offers.length,
          separatorBuilder: (_, _) => const SizedBox(height: 12),
          itemBuilder: (context, i) {
            final offer = offers[i];
            return Card(
              child: ListTile(
                leading: const CircleAvatar(child: Icon(Icons.monetization_on)),
                title: Text('Bid: \$${offer.amount.toStringAsFixed(2)}'),
                subtitle: Text('Status: ${offer.status.name.toUpperCase()} • Task: ${offer.taskId}'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => context.push('/bids/${offer.id}'),
              ),
            );
          },
        );
      },
    );
  }
}

class _OffersReceivedList extends ConsumerWidget {
  const _OffersReceivedList();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // In production, queries bids where creatorId == current user DID
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.inbox_outlined, size: 48, color: HBColors.onSurfaceVariantDark.withValues(alpha: 0.5)),
          const SizedBox(height: 12),
          const Text('No incoming offers received yet.', style: TextStyle(color: HBColors.onSurfaceVariantDark)),
          const SizedBox(height: 8),
          const Text('Publish a task to start receiving competitive bids.', style: TextStyle(fontSize: 12, color: HBColors.onSurfaceVariantDark)),
        ],
      ),
    );
  }
}
