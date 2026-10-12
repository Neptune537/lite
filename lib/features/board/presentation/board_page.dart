import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'dart:convert';
import 'dart:math' as math;
import 'package:go_router/go_router.dart';
import 'package:geolocator/geolocator.dart';

import '../../../app/theme.dart';
import '../../notifications/presentation/notifications_button.dart';
import 'board_search_delegate.dart';
import 'local_offerings_provider.dart';
import '../../teams/presentation/team_providers.dart';
import '../../tasks/presentation/make_offer_modal.dart';
import 'edit_offering_dialog.dart';

class BoardPage extends ConsumerStatefulWidget {
  const BoardPage({super.key});

  @override
  ConsumerState<BoardPage> createState() => _BoardPageState();
}

class _BoardPageState extends ConsumerState<BoardPage>
    with TickerProviderStateMixin {
  late final TabController _tabCtrl;

  @override
  void initState() {
    super.initState();
    _tabCtrl = TabController(length: 4, vsync: this);
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
        title: const Row(
          children: [
            Icon(Icons.handshake_rounded, color: HBColors.primary),
            SizedBox(width: 8.0),
            Text('HeavenlyBond Board'),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {
              showSearch(
                context: context,
                delegate: BoardSearchDelegate(ref: ref),
              );
            },
          ),
          const NotificationsButton(),
        ],
        bottom: TabBar(
          controller: _tabCtrl,
          isScrollable: true,
          tabAlignment: TabAlignment.start,
          tabs: const [
            Tab(text: 'Main Feed'),
            Tab(text: 'My Guild'),
            Tab(text: 'Local Radar'),
            Tab(text: 'My Interactions'),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.push('/create'),
        icon: const Icon(Icons.add_task),
        label: const Text('Post Task'),
      ),
      body: TabBarView(
        controller: _tabCtrl,
        children: const [
          _MainOfferingsTab(),
          _MyClubOfferingsTab(),
          _LocalTab(),
          _InteractingTab(),
        ],
      ),
    );
  }
}

class _MainOfferingsTab extends ConsumerStatefulWidget {
  const _MainOfferingsTab();

  @override
  ConsumerState<_MainOfferingsTab> createState() => _MainOfferingsTabState();
}

class _MainOfferingsTabState extends ConsumerState<_MainOfferingsTab> {
  final TextEditingController _zipFilterCtrl = TextEditingController();
  String _activeZipFilter = '';

  @override
  void dispose() {
    _zipFilterCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final offerings = ref.watch(localOfferingsProvider);

    final filtered = offerings.where((offering) {
      if (_activeZipFilter.isEmpty) return true;
      final dataStr = offering['data'] as String? ?? '{}';
      try {
        final decoded = jsonDecode(dataStr) as Map<String, dynamic>;
        final zip = decoded['zipCode']?.toString() ?? '';
        final loc = decoded['locationLabel']?.toString() ?? '';
        return zip.contains(_activeZipFilter) || loc.toLowerCase().contains(_activeZipFilter.toLowerCase());
      } catch (_) {
        return true;
      }
    }).toList();

    return Column(
      children: [
        // Location / ZIP Code Filter Bar
        Padding(
          padding: const EdgeInsets.fromLTRB(16.0, 12.0, 16.0, 4.0),
          child: TextField(
            controller: _zipFilterCtrl,
            style: const TextStyle(color: Colors.white),
            decoration: InputDecoration(
              hintText: 'Filter tasks by ZIP code or location...',
              hintStyle: const TextStyle(color: Colors.white38, fontSize: 14),
              prefixIcon: const Icon(Icons.pin_drop, color: HBColors.primary),
              suffixIcon: _activeZipFilter.isNotEmpty
                  ? IconButton(
                      icon: const Icon(Icons.clear, color: Colors.white54),
                      onPressed: () {
                        _zipFilterCtrl.clear();
                        setState(() => _activeZipFilter = '');
                      },
                    )
                  : null,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              filled: true,
              fillColor: HBColors.neutral,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(HBRadius.md),
                borderSide: const BorderSide(color: HBColors.primary),
              ),
            ),
            onChanged: (val) {
              setState(() => _activeZipFilter = val.trim());
            },
          ),
        ),

        Expanded(
          child: filtered.isEmpty
              ? Center(
                  child: Text(
                    _activeZipFilter.isNotEmpty
                        ? 'No tasks found matching ZIP "$_activeZipFilter"'
                        : 'No offerings found.',
                    style: const TextStyle(color: Colors.white70, fontSize: 16),
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.all(16.0),
                  itemCount: filtered.length,
                  itemBuilder: (context, index) {
                    return OfferingCard(offering: filtered[index]);
                  },
                ),
        ),
      ],
    );
  }
}

class _MyClubOfferingsTab extends ConsumerWidget {
  const _MyClubOfferingsTab();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final offerings = ref.watch(localOfferingsProvider);
    String? selectedTeamId;
    try {
      selectedTeamId = ref.watch(selectedTeamProvider);
    } catch (_) {}

    final clubOfferings = selectedTeamId == null
        ? offerings
        : offerings.where((o) => o['creatorId'] == selectedTeamId).toList();

    if (clubOfferings.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.group_off, size: 64, color: Colors.grey),
            const SizedBox(height: 16),
            Text(
              selectedTeamId == null
                ? 'Join a team in Teams page'
                : 'No offerings for your club yet.',
              style: const TextStyle(color: Colors.white, fontSize: 16),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(16.0),
      itemCount: clubOfferings.length,
      itemBuilder: (context, index) {
        return OfferingCard(offering: clubOfferings[index]);
      },
    );
  }
}

class OfferingCard extends ConsumerStatefulWidget {
  final Map<String, dynamic> offering;
  const OfferingCard({super.key, required this.offering});

  @override
  ConsumerState<OfferingCard> createState() => _OfferingCardState();
}

class _OfferingCardState extends ConsumerState<OfferingCard> {
  bool _expanded = true; // Auto-expanded by default for instant interaction visibility

  void _editOffering() {
    showDialog(
      context: context,
      builder: (context) => EditOfferingDialog(offering: widget.offering),
    );
  }

  void _openMakeOfferModal() {
    final taskId = widget.offering['id']?.toString() ?? 'seed_1';
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: HBColors.neutral,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(HBRadius.xl)),
        side: BorderSide(color: HBColors.primary, width: 1.5),
      ),
      builder: (context) => MakeOfferModal(taskId: taskId),
    );
  }

  @override
  Widget build(BuildContext context) {
    String? selectedTeamId;
    try {
      selectedTeamId = ref.watch(selectedTeamProvider);
    } catch (_) {}

    final dataStr = widget.offering['data'] as String? ?? '{}';
    String title = "Unknown";
    String description = "No description";
    String category = "Unknown";
    String bounty = "0";
    String zipCode = "78701";

    try {
      final decoded = jsonDecode(dataStr) as Map<String, dynamic>;
      title = decoded['title']?.toString() ?? "Unknown";
      description = decoded['description']?.toString() ?? "No description";
      category = decoded['category']?.toString() ?? "Unknown";
      bounty = decoded['bounty']?.toString() ?? (decoded['budgetAmount'] != null ? '\$${decoded['budgetAmount']}' : "100 ₣");
      zipCode = decoded['zipCode']?.toString() ?? "78701";
    } catch (e) {
      debugPrint('Parse error: $e');
    }

    final isCreator = widget.offering['creatorId'] == selectedTeamId;

    return Card(
      margin: const EdgeInsets.only(bottom: 16.0),
      color: HBColors.neutral,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(HBRadius.md),
        side: BorderSide(color: HBColors.primary.withValues(alpha: 0.6)),
      ),
      child: Column(
        children: [
          ListTile(
            onTap: () => setState(() => _expanded = !_expanded),
            title: Text(
              title,
              style: const TextStyle(fontWeight: FontWeight.bold, color: Colors.white, fontSize: 16),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            subtitle: Padding(
              padding: const EdgeInsets.only(top: 4.0),
              child: Text(
                '📍 ZIP: $zipCode  •  🏷️ $category  •  💰 $bounty',
                style: const TextStyle(color: HBColors.primaryLight, fontSize: 12, fontWeight: FontWeight.w600),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            trailing: IconButton(
              icon: Icon(_expanded ? Icons.expand_less : Icons.expand_more, color: HBColors.primary),
              onPressed: () => setState(() => _expanded = !_expanded),
            ),
          ),
          if (_expanded)
            Padding(
              padding: const EdgeInsets.fromLTRB(16.0, 0.0, 16.0, 16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Divider(color: Colors.white12),
                  const SizedBox(height: 8),
                  Text(
                    description,
                    style: const TextStyle(color: Colors.white70, height: 1.4, fontSize: 13),
                  ),
                  const SizedBox(height: 16),

                  // Explicit Interaction Buttons
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      if (isCreator) ...[
                        OutlinedButton.icon(
                          style: OutlinedButton.styleFrom(
                            foregroundColor: HBColors.secondary,
                            side: const BorderSide(color: HBColors.secondary),
                          ),
                          icon: const Icon(Icons.edit, size: 16),
                          label: const Text('Edit Listing'),
                          onPressed: _editOffering,
                        ),
                        const SizedBox(width: 8),
                      ],
                      ElevatedButton.icon(
                        style: ElevatedButton.styleFrom(
                          backgroundColor: HBColors.primary,
                          foregroundColor: Colors.white,
                          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                        ),
                        icon: const Icon(Icons.handshake, size: 18),
                        label: const Text('Interact / Make Offer'),
                        onPressed: _openMakeOfferModal,
                      ),
                    ],
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

// ── Interacting Tab (Bids & Negotiations) ────────────────────────────────────
class _InteractingTab extends ConsumerWidget {
  const _InteractingTab();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final bids = ref.watch(userBidsProvider);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: const [
              Icon(Icons.gavel, color: HBColors.primary, size: 28),
              SizedBox(width: 10),
              Text(
                'My Active Bids & Contracts',
                style: TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold),
              ),
            ],
          ),
          const SizedBox(height: 16),

          // Dynamically rendered bids from SQLite
          if (bids.isNotEmpty) ...[
            const Text(
              'SUBMITTED BIDS & NEGOTIATIONS',
              style: TextStyle(color: HBColors.primaryLight, fontSize: 12, fontWeight: FontWeight.bold, letterSpacing: 1.2),
            ),
            const SizedBox(height: 10),
            ...bids.map((b) {
              final dataStr = b['data'] as String? ?? '{}';
              double amount = 0.0;
              String msg = '';
              try {
                final decoded = jsonDecode(dataStr) as Map<String, dynamic>;
                amount = (decoded['amount'] as num?)?.toDouble() ?? 0.0;
                msg = decoded['message']?.toString() ?? '';
              } catch (_) {}

              return _ContractTile(
                title: 'Task ID: ${b['taskId'].toString().substring(0, math.min(12, (b['taskId'] as String).length))}',
                status: 'SUBMITTED BID',
                statusColor: HBColors.primary,
                bounty: '\$${amount.toStringAsFixed(2)}',
                zipCode: '78701',
                note: msg.isNotEmpty ? 'Note: $msg' : 'Bid active on TitheX network.',
              );
            }),
            const SizedBox(height: 20),
          ],

          const Text(
            'SAMPLE CONTRACTS & ESCROW',
            style: TextStyle(color: Colors.white54, fontSize: 12, fontWeight: FontWeight.bold, letterSpacing: 1.2),
          ),
          const SizedBox(height: 10),
          _ContractTile(
            title: 'Cybernetic Armor Repair',
            status: 'OFFER PENDING',
            statusColor: HBColors.secondary,
            bounty: '150 ₣',
            zipCode: '78701',
            note: 'Your bid of 140 ₣ is currently undergoing review by Team Alpha.',
          ),
          _ContractTile(
            title: 'Encrypted Data Relay',
            status: 'ACCEPTED / ESCROW LOCKED',
            statusColor: Colors.greenAccent,
            bounty: '300 ₣',
            zipCode: '78702',
            note: 'TitheX Escrow active: 270 ₣ Worker Bounty, 30 ₣ Community Tithe locked.',
          ),
        ],
      ),
    );
  }
}

class _ContractTile extends StatelessWidget {
  final String title;
  final String status;
  final Color statusColor;
  final String bounty;
  final String zipCode;
  final String note;

  const _ContractTile({
    required this.title,
    required this.status,
    required this.statusColor,
    required this.bounty,
    required this.zipCode,
    required this.note,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12.0),
      color: HBColors.neutral,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(HBRadius.md),
        side: BorderSide(color: statusColor.withValues(alpha: 0.6)),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(title, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 15)),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: statusColor.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: statusColor),
                  ),
                  child: Text(status, style: TextStyle(color: statusColor, fontSize: 10, fontWeight: FontWeight.bold)),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text('ZIP: $zipCode  •  Bounty: $bounty', style: const TextStyle(color: HBColors.primaryLight, fontSize: 12)),
            const SizedBox(height: 6),
            Text(note, style: const TextStyle(color: Colors.white70, fontSize: 12)),
          ],
        ),
      ),
    );
  }
}

// ── Local Tab (Proximity Radar & Distance Filtering) ─────────────────────────
class _LocalTab extends ConsumerStatefulWidget {
  const _LocalTab();

  @override
  ConsumerState<_LocalTab> createState() => _LocalTabState();
}

class _LocalTabState extends ConsumerState<_LocalTab> {
  double _radius = 25.0; // 25 miles default search radius
  Position? _userPosition;

  @override
  void initState() {
    super.initState();
    _getUserLocation();
  }

  Future<void> _getUserLocation() async {
    try {
      final lastPos = await Geolocator.getLastKnownPosition();
      if (lastPos != null && mounted) {
        setState(() => _userPosition = lastPos);
      }
      final pos = await Geolocator.getCurrentPosition(
        locationSettings: const LocationSettings(accuracy: LocationAccuracy.medium),
      ).timeout(const Duration(seconds: 5));
      if (mounted) {
        setState(() => _userPosition = pos);
      }
    } catch (_) {}
  }

  double _calculateDistanceMiles(double lat1, double lon1, double lat2, double lon2) {
    const p = 0.017453292519943295;
    final a = 0.5 - math.cos((lat2 - lat1) * p) / 2 +
        math.cos(lat1 * p) * math.cos(lat2 * p) *
        (1 - math.cos((lon2 - lon1) * p)) / 2;
    return 12742 * math.asin(math.sqrt(a)) * 0.621371;
  }

  @override
  Widget build(BuildContext context) {
    final userLat = _userPosition?.latitude ?? 29.6549; // Houston default
    final userLng = _userPosition?.longitude ?? -95.2285;

    final offerings = ref.watch(localOfferingsProvider);

    // Calculate distance in miles for each task
    final List<Map<String, dynamic>> nearbyOfferings = [];

    for (var offering in offerings) {
      final dataStr = offering['data'] as String? ?? '{}';
      double taskLat = userLat;
      double taskLng = userLng;
      String zip = '78701';

      try {
        final decoded = jsonDecode(dataStr) as Map<String, dynamic>;
        zip = decoded['zipCode']?.toString() ?? '78701';
        
        if (decoded['lat'] != null && decoded['lng'] != null) {
          taskLat = (decoded['lat'] as num).toDouble();
          taskLng = (decoded['lng'] as num).toDouble();
        } else {
          // Offsets based on ZIP code
          final zipHash = zip.hashCode;
          taskLat = userLat + ((zipHash % 100) - 50) * 0.002;
          taskLng = userLng + (((zipHash ~/ 100) % 100) - 50) * 0.002;
        }
      } catch (_) {}

      final distMiles = _calculateDistanceMiles(userLat, userLng, taskLat, taskLng);

      if (distMiles <= _radius) {
        final offeringCopy = Map<String, dynamic>.from(offering);
        offeringCopy['_distanceMiles'] = distMiles;
        nearbyOfferings.add(offeringCopy);
      }
    }

    nearbyOfferings.sort((a, b) => (a['_distanceMiles'] as double).compareTo(b['_distanceMiles'] as double));

    return Column(
      children: [
        // Search Radius Slider Header
        Container(
          padding: const EdgeInsets.all(16.0),
          color: HBColors.neutral,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.radar, color: HBColors.primary, size: 20),
                      const SizedBox(width: 8),
                      Text(
                        'LOCAL PROXIMITY RADAR: ${_radius.toInt()} MILES',
                        style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13, letterSpacing: 1.1),
                      ),
                    ],
                  ),
                  Text(
                    '${nearbyOfferings.length} Nearby Tasks',
                    style: const TextStyle(color: HBColors.secondary, fontWeight: FontWeight.bold, fontSize: 12),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Slider(
                value: _radius,
                min: 1.0,
                max: 100.0,
                divisions: 99,
                label: '${_radius.toInt()} mi',
                activeColor: HBColors.primary,
                inactiveColor: Colors.white24,
                onChanged: (val) {
                  setState(() => _radius = val);
                },
              ),
              Text(
                'GPS Lock: ${userLat.toStringAsFixed(3)}°, ${userLng.toStringAsFixed(3)}°',
                style: const TextStyle(color: Colors.white54, fontSize: 11, fontFamily: 'monospace'),
              ),
            ],
          ),
        ),

        const Divider(color: HBColors.primary, height: 1),

        // Nearby Task List
        Expanded(
          child: nearbyOfferings.isEmpty
              ? Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(Icons.radar_outlined, size: 64, color: Colors.grey),
                      const SizedBox(height: 16.0),
                      Text(
                        'No local tasks found within ${_radius.toInt()} miles.',
                        style: const TextStyle(color: Colors.white70, fontSize: 15),
                      ),
                      const SizedBox(height: 8),
                      const Text(
                        'Try expanding your radius slider above.',
                        style: TextStyle(color: Colors.white38, fontSize: 12),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.all(16.0),
                  itemCount: nearbyOfferings.length,
                  itemBuilder: (context, index) {
                    final offering = nearbyOfferings[index];
                    final dist = offering['_distanceMiles'] as double;
                    return Stack(
                      children: [
                        OfferingCard(offering: offering),
                        Positioned(
                          top: 12,
                          right: 48,
                          child: Container(
                            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                            decoration: BoxDecoration(
                              color: HBColors.secondary.withValues(alpha: 0.2),
                              borderRadius: BorderRadius.circular(6),
                              border: Border.all(color: HBColors.secondary),
                            ),
                            child: Text(
                              '${dist.toStringAsFixed(1)} mi away',
                              style: const TextStyle(color: HBColors.secondary, fontSize: 11, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                            ),
                          ),
                        ),
                      ],
                    );
                  },
                ),
        ),
      ],
    );
  }
}
