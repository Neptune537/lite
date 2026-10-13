import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../../app/theme.dart';
import '../../../core/blockchain/fluoridian_service.dart';
import '../../../core/blockchain/models/network_event.dart';

/// Sprint 3: Offer Detail & Negotiation Page
/// Allows Task Creators to inspect bidder credentials, counter, accept, or reject.
/// Target Platforms: Android and Web.
class OfferDetailPage extends ConsumerStatefulWidget {
  final String offerId;

  const OfferDetailPage({required this.offerId, super.key});

  @override
  ConsumerState<OfferDetailPage> createState() => _OfferDetailPageState();
}

class _OfferDetailPageState extends ConsumerState<OfferDetailPage> {
  bool _isProcessing = false;

  Future<void> _handleDecision({required bool accept, double? counterAmount}) async {
    setState(() => _isProcessing = true);
    final fluoridian = ref.read(fluoridianServiceProvider);

    final eventType = accept ? NetworkEventType.bidAccepted : NetworkEventType.bidRejected;
    await fluoridian.signAndBroadcast(
      eventType: eventType,
      payload: {
        'offerId': widget.offerId,
        'action': accept ? 'ACCEPT' : (counterAmount != null ? 'COUNTER' : 'REJECT'),
        'counterAmount': counterAmount,
        'timestamp': DateTime.now().toIso8601String(),
      },
    );

    if (mounted) {
      setState(() => _isProcessing = false);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(accept ? 'Offer accepted! Escrow locked on Fluoridian.' : 'Offer rejected.'),
          backgroundColor: accept ? Colors.green : Colors.redAccent,
        ),
      );
      context.pop();
    }
  }

  void _showCounterDialog() {
    final counterCtrl = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Propose Counter Offer'),
        content: TextField(
          controller: counterCtrl,
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: const InputDecoration(
            labelText: 'Counter Bounty (USD)',
            prefixText: '\$ ',
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
          FilledButton(
            onPressed: () {
              final val = double.tryParse(counterCtrl.text);
              Navigator.pop(ctx);
              if (val != null) _handleDecision(accept: false, counterAmount: val);
            },
            child: const Text('Submit Counter'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Offer #${widget.offerId.substring(0, widget.offerId.length > 8 ? 8 : widget.offerId.length)}'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Card(
              child: Padding(
                padding: EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text('Bid Amount', style: TextStyle(color: HBColors.onSurfaceVariantDark)),
                        Text('\$140.00', style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: HBColors.primary)),
                      ],
                    ),
                    Divider(height: 24),
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      leading: CircleAvatar(child: Icon(Icons.person)),
                      title: Text('Provider: did:nexus:worker_alpha'),
                      subtitle: Text('Rating: ★ 4.9 (24 jobs completed)'),
                    ),
                    SizedBox(height: 8),
                    Text('Proposed Completion:', style: TextStyle(fontWeight: FontWeight.w600)),
                    Text('Within 48 hours of acceptance'),
                    SizedBox(height: 12),
                    Text('Verified Credentials:', style: TextStyle(fontWeight: FontWeight.w600)),
                    SizedBox(height: 6),
                    Wrap(
                      spacing: 8,
                      children: [
                        Chip(avatar: Icon(Icons.verified, size: 16), label: Text('Driver License')),
                        Chip(avatar: Icon(Icons.security, size: 16), label: Text('Liability Verified')),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 24),
            if (_isProcessing)
              const Center(child: CircularProgressIndicator())
            else
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      icon: const Icon(Icons.close, color: Colors.redAccent),
                      label: const Text('Reject'),
                      onPressed: () => _handleDecision(accept: false),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: OutlinedButton.icon(
                      icon: const Icon(Icons.swap_horiz),
                      label: const Text('Counter'),
                      onPressed: _showCounterDialog,
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: FilledButton.icon(
                      icon: const Icon(Icons.check),
                      label: const Text('Accept'),
                      onPressed: () => _handleDecision(accept: true),
                    ),
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }
}
