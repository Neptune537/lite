import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:uuid/uuid.dart';
import '../../../app/theme.dart';
import '../../board/presentation/local_offerings_provider.dart';
import '../../../core/services/local_database_service.dart';
import '../domain/team_war.dart';
import 'team_providers.dart';
import 'declare_war_dialog.dart';

class TeamsPage extends ConsumerStatefulWidget {
  const TeamsPage({super.key});

  @override
  ConsumerState<TeamsPage> createState() => _TeamsPageState();
}

class _TeamsPageState extends ConsumerState<TeamsPage>
    with TickerProviderStateMixin {
  late TabController _tabController;
  final dbService = LocalDatabaseService.instance;

  // Elemental teams data
  final List<Map<String, dynamic>> elementalTeams = [
    {
      'id': 'water',
      'name': 'Water Clan',
      'membersOnline': 124,
      'icon': Icons.water_drop,
      'color': Colors.blue,
    },
    {
      'id': 'fire',
      'name': 'Fire Tribe',
      'membersOnline': 89,
      'icon': Icons.local_fire_department,
      'color': Colors.orange,
    },
    {
      'id': 'earth',
      'name': 'Earth Guild',
      'membersOnline': 210,
      'icon': Icons.eco,
      'color': Colors.green,
    },
    {
      'id': 'wind',
      'name': 'Wind Order',
      'membersOnline': 156,
      'icon': Icons.air,
      'color': Colors.cyan,
    },
  ];

  List<Map<String, dynamic>> _teamOfferings = [];
  List<TeamWar> _wars = [];
  bool _isLoading = false;
  String _listingFilter = 'all'; // 'all', 'forSale', 'wantedToBuy', 'forBook'

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    _loadOfferings();
    _loadWars();
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _loadOfferings() async {
    setState(() => _isLoading = true);
    try {
      final db = await dbService.database;
      final results = await db.query('tasks', orderBy: 'createdAt DESC');
      setState(() {
        _teamOfferings = results
            .map((e) => Map<String, dynamic>.from(e))
            .toList();
      });
    } catch (e) {
      debugPrint('Error loading offerings: $e');
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Future<void> _loadWars() async {
    try {
      final db = await dbService.database;
      final results = await db.query('wars', orderBy: 'declaredAt DESC');
      setState(() {
        _wars = results.map((e) => TeamWar.fromMap(e)).toList();
      });
    } catch (e) {
      debugPrint('Error loading wars: $e');
    }
  }

  String _getTeamName(String teamId) {
    final team = elementalTeams.firstWhere(
      (t) => t['id'] == teamId,
      orElse: () => {'name': 'Unknown'},
    );
    return team['name'] as String;
  }

  IconData _getTeamIcon(String teamId) {
    final team = elementalTeams.firstWhere(
      (t) => t['id'] == teamId,
      orElse: () => {'icon': Icons.help_outline},
    );
    return team['icon'] as IconData;
  }

  void _declareWar(String enemyTeamId, String enemyTeamName) {
    final selectedTeamId = ref.read(selectedTeamProvider);
    if (selectedTeamId == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Join a team first before declaring war!'),
        ),
      );
      return;
    }
    if (selectedTeamId == enemyTeamId) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text("You can't declare war on your own team!"),
        ),
      );
      return;
    }

    final myTeamName = _getTeamName(selectedTeamId);

    showDialog(
      context: context,
      builder: (context) => DeclareWarDialog(
        myTeamName: myTeamName,
        enemyTeamName: enemyTeamName,
        enemyTeamId: enemyTeamId,
        onDeclare: (message) async {
          final db = await dbService.database;
          final war = TeamWar(
            id: const Uuid().v4(),
            challengerTeamId: selectedTeamId,
            defenderTeamId: enemyTeamId,
            status: 'active',
            declaredAt: DateTime.now().millisecondsSinceEpoch,
            message: message.isEmpty ? null : message,
          );
          await db.insert('wars', war.toMap());
          _loadWars();
          if (!context.mounted) return;
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('⚔️ War declared against $enemyTeamName!'),
              backgroundColor: HBColors.error,
            ),
          );
          // Switch to Wars tab
          _tabController.animateTo(2);
        },
      ),
    );
  }

  void _showCreateOfferingModal() {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Theme.of(context).scaffoldBackgroundColor,
      builder: (context) => Padding(
        padding: EdgeInsets.only(
          bottom: MediaQuery.of(context).viewInsets.bottom,
        ),
        child: CreateOfferingForm(
          onSaved:
              (
                title,
                description,
                category,
                bounty,
                teamId,
                listingType,
                bookingDuration,
                zipCode,
                conversationType,
              ) async {
                final db = await dbService.database;
                final now = DateTime.now().millisecondsSinceEpoch;
                final id = const Uuid().v4();
                await db.insert('tasks', {
                  'id': id,
                  'creatorId': teamId,
                  'data':
                      '{"title": "$title", "description": "$description", "category": "$category", "bounty": $bounty, "zipCode": "$zipCode", "conversationType": "$conversationType"}',
                  'listingType': listingType,
                  'bookingDuration': bookingDuration,
                  'createdAt': now,
                });
                if (!context.mounted) return;
                ref.read(localOfferingsNotifierProvider.notifier).refresh();
                Navigator.pop(context);
                _loadOfferings();
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Listing created successfully!'),
                  ),
                );
              },
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Guilds / Marketplace'),
        actions: [
          IconButton(
            icon: const Icon(Icons.hub_outlined),
            tooltip: 'Constellation Map',
            onPressed: () {
              context.push('/teams/constellation');
            },
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(text: 'My Teams'),
            Tab(text: 'Marketplace'),
            Tab(icon: Icon(Icons.military_tech, size: 18), text: 'Wars'),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _showCreateOfferingModal,
        child: const Icon(Icons.add),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          // ── Tab 1: My Teams ──────────────────────────────────────────────
          _buildTeamsTab(),

          // ── Tab 2: Marketplace ───────────────────────────────────────────
          _buildMarketplaceTab(),

          // ── Tab 3: Wars ──────────────────────────────────────────────────
          _buildWarsTab(),
        ],
      ),
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TAB 1: MY TEAMS
  // ═══════════════════════════════════════════════════════════════════════════
  Widget _buildTeamsTab() {
    return Consumer(
      builder: (context, ref, child) {
        final selectedTeamId = ref.watch(selectedTeamProvider);

        return ListView.builder(
          itemCount: elementalTeams.length,
          itemBuilder: (context, index) {
            final team = elementalTeams[index];
            final isSelected = team['id'] == selectedTeamId;
            final isEnemy =
                selectedTeamId != null && team['id'] != selectedTeamId;

            return Card(
              margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
              child: ListTile(
                leading: CircleAvatar(
                  backgroundColor: isSelected
                      ? HBColors.primary
                      : (team['color'] as Color).withValues(alpha: 0.3),
                  child: Icon(
                    team['icon'] as IconData,
                    color: isSelected ? Colors.white : team['color'] as Color,
                  ),
                ),
                title: Text(
                  team['name'] as String,
                  style: TextStyle(
                    fontWeight: isSelected
                        ? FontWeight.bold
                        : FontWeight.normal,
                  ),
                ),
                subtitle: Text('${team['membersOnline']} members online'),
                trailing: isSelected
                    ? const Icon(Icons.check_circle, color: Colors.green)
                    : Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          if (isEnemy)
                            IconButton(
                              icon: const Icon(
                                Icons.military_tech,
                                color: HBColors.error,
                              ),
                              tooltip: 'Declare War',
                              onPressed: () => _declareWar(
                                team['id'] as String,
                                team['name'] as String,
                              ),
                            ),
                          ElevatedButton(
                            onPressed: () {
                              ref
                                  .read(selectedTeamProvider.notifier)
                                  .selectTeam(team['id'] as String);
                            },
                            child: const Text('Join'),
                          ),
                        ],
                      ),
                onTap: () {
                  if (!isSelected) {
                    ref
                        .read(selectedTeamProvider.notifier)
                        .selectTeam(team['id'] as String);
                  }
                },
              ),
            );
          },
        );
      },
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TAB 2: MARKETPLACE
  // ═══════════════════════════════════════════════════════════════════════════
  Widget _buildMarketplaceTab() {
    final filtered = _listingFilter == 'all'
        ? _teamOfferings
        : _teamOfferings.where((item) {
            final type = item['listingType'] as String? ?? 'forSale';
            return type == _listingFilter;
          }).toList();

    return Column(
      children: [
        // Filter chips
        Padding(
          padding: const EdgeInsets.all(HBSpacing.sm),
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                _filterChip('All', 'all'),
                const SizedBox(width: 8),
                _filterChip('💰 For Sale', 'forSale'),
                const SizedBox(width: 8),
                _filterChip('🛒 Wanted', 'wantedToBuy'),
                const SizedBox(width: 8),
                _filterChip('📅 Book Service', 'forBook'),
              ],
            ),
          ),
        ),

        // Listings
        Expanded(
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : filtered.isEmpty
              ? const Center(child: Text('No listings yet. Create one!'))
              : ListView.builder(
                  itemCount: filtered.length,
                  itemBuilder: (context, index) {
                    final item = filtered[index];
                    return _buildMarketplaceCard(item);
                  },
                ),
        ),
      ],
    );
  }

  Widget _filterChip(String label, String value) {
    final isSelected = _listingFilter == value;
    return FilterChip(
      label: Text(label),
      selected: isSelected,
      onSelected: (_) => setState(() => _listingFilter = value),
      selectedColor: HBColors.primary.withValues(alpha: 0.3),
      checkmarkColor: HBColors.primary,
    );
  }

  Widget _buildMarketplaceCard(Map<String, dynamic> item) {
    final dataStr = item['data'] as String;
    final listingType = item['listingType'] as String? ?? 'forSale';
    final bookingDuration = item['bookingDuration'] as String?;
    String title = 'Unknown';
    String category = 'Unknown';
    String bounty = '0';

    try {
      final decoded = jsonDecode(dataStr) as Map<String, dynamic>;
      title = decoded['title']?.toString() ?? 'Unknown';
      category = decoded['category']?.toString() ?? 'Unknown';
      bounty = decoded['bounty']?.toString() ?? '0';
    } catch (e) {
      debugPrint('Parse error: $e');
    }

    // Listing type badge
    final (badgeText, badgeColor) = switch (listingType) {
      'forSale' => ('FOR SALE', Colors.green),
      'wantedToBuy' => ('WANTED', Colors.orange),
      'forBook' => ('FOR BOOKING', Colors.blue),
      _ => ('LISTING', Colors.grey),
    };

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
      child: ListTile(
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: badgeColor.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(HBRadius.sm),
                border: Border.all(color: badgeColor, width: 1),
              ),
              child: Text(
                badgeText,
                style: TextStyle(
                  color: badgeColor,
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                  letterSpacing: 0.5,
                ),
              ),
            ),
            const SizedBox(width: 8),
            Expanded(child: Text(title)),
          ],
        ),
        subtitle: Text(
          'Category: $category • \$$bounty'
          '${bookingDuration != null ? ' • $bookingDuration' : ''}',
        ),
        trailing: const Icon(Icons.circle, color: Colors.green, size: 12),
      ),
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TAB 3: WARS
  // ═══════════════════════════════════════════════════════════════════════════
  Widget _buildWarsTab() {
    if (_wars.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.military_tech,
              size: 64,
              color: Colors.grey.withValues(alpha: 0.4),
            ),
            const SizedBox(height: HBSpacing.md),
            const Text(
              'No active wars',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: HBSpacing.sm),
            const Text(
              'Join a team, then declare war\non a rival from the Teams tab!',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(HBSpacing.md),
      itemCount: _wars.length,
      itemBuilder: (context, index) {
        final war = _wars[index];
        return _buildWarCard(war);
      },
    );
  }

  Widget _buildWarCard(TeamWar war) {
    final challengerName = _getTeamName(war.challengerTeamId);
    final defenderName = _getTeamName(war.defenderTeamId);
    final challengerIcon = _getTeamIcon(war.challengerTeamId);
    final defenderIcon = _getTeamIcon(war.defenderTeamId);

    final statusColor = switch (war.status) {
      'active' => HBColors.error,
      'pending' => Colors.amber,
      'ceasefire' => Colors.blue,
      'victory' => Colors.green,
      _ => Colors.grey,
    };

    final statusEmoji = switch (war.status) {
      'active' => '⚔️',
      'pending' => '⏳',
      'ceasefire' => '🕊️',
      'victory' => '🏆',
      'defeat' => '💀',
      _ => '❓',
    };

    return Card(
      margin: const EdgeInsets.only(bottom: HBSpacing.md),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(HBRadius.lg),
        side: BorderSide(color: statusColor, width: 1.5),
      ),
      child: Padding(
        padding: const EdgeInsets.all(HBSpacing.md),
        child: Column(
          children: [
            // Status header
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(statusEmoji, style: const TextStyle(fontSize: 18)),
                const SizedBox(width: 8),
                Text(
                  war.status.toUpperCase(),
                  style: TextStyle(
                    color: statusColor,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 2,
                    fontSize: 14,
                  ),
                ),
              ],
            ),
            const SizedBox(height: HBSpacing.md),

            // Teams face-off
            Row(
              children: [
                Expanded(
                  child: Column(
                    children: [
                      Icon(challengerIcon, size: 36, color: HBColors.primary),
                      const SizedBox(height: 4),
                      Text(
                        challengerName,
                        textAlign: TextAlign.center,
                        style: const TextStyle(
                          fontWeight: FontWeight.bold,
                          fontSize: 13,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        '${war.challengerScore}',
                        style: const TextStyle(
                          fontSize: 28,
                          fontWeight: FontWeight.bold,
                          color: HBColors.primary,
                        ),
                      ),
                    ],
                  ),
                ),
                const Text(
                  'VS',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.bold,
                    color: Colors.white38,
                  ),
                ),
                Expanded(
                  child: Column(
                    children: [
                      Icon(defenderIcon, size: 36, color: HBColors.error),
                      const SizedBox(height: 4),
                      Text(
                        defenderName,
                        textAlign: TextAlign.center,
                        style: const TextStyle(
                          fontWeight: FontWeight.bold,
                          fontSize: 13,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        '${war.defenderScore}',
                        style: const TextStyle(
                          fontSize: 28,
                          fontWeight: FontWeight.bold,
                          color: HBColors.error,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),

            // War message
            if (war.message != null && war.message!.isNotEmpty) ...[
              const SizedBox(height: HBSpacing.sm),
              Text(
                '"${war.message}"',
                style: const TextStyle(
                  fontStyle: FontStyle.italic,
                  color: Colors.white54,
                  fontSize: 12,
                ),
                textAlign: TextAlign.center,
              ),
            ],
          ],
        ),
      ),
    );
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CREATE OFFERING FORM — with Buy/Sell/Rent listing type
// ═══════════════════════════════════════════════════════════════════════════════
class CreateOfferingForm extends StatefulWidget {
  final Function(
    String title,
    String description,
    String category,
    int bounty,
    String teamId,
    String listingType,
    String? bookingDuration,
    String zipCode,
    String conversationType,
  )
  onSaved;

  const CreateOfferingForm({super.key, required this.onSaved});

  @override
  State<CreateOfferingForm> createState() => _CreateOfferingFormState();
}

class _CreateOfferingFormState extends State<CreateOfferingForm> {
  final _formKey = GlobalKey<FormState>();
  String _title = '';
  String _description = '';
  String _category = 'Service';
  int _bounty = 100;
  String _zipCode = '78701';
  String _selectedTeamId = 'water';
  String _listingType = 'forSale';
  String? _bookingDuration;
  String _conversationType = 'none';

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Form(
        key: _formKey,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'Create Listing',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),

              // Listing Type
              DropdownButtonFormField<String>(
                initialValue: _listingType,
                decoration: const InputDecoration(labelText: 'Listing Type'),
                items: const [
                  DropdownMenuItem(
                    value: 'forSale',
                    child: Text('💰 For Sale'),
                  ),
                  DropdownMenuItem(
                    value: 'wantedToBuy',
                    child: Text('🛒 Wanted (Buy)'),
                  ),
                  DropdownMenuItem(
                    value: 'forBook',
                    child: Text('🔑 Book Service'),
                  ),
                ],
                onChanged: (v) => setState(() {
                  _listingType = v!;
                  if (v != 'forBook') _bookingDuration = null;
                }),
              ),
              const SizedBox(height: 16),

              TextFormField(
                decoration: const InputDecoration(labelText: 'Title'),
                validator: (v) => v == null || v.isEmpty ? 'Required' : null,
                onSaved: (v) => _title = v ?? '',
              ),
              const SizedBox(height: 16),
              TextFormField(
                decoration: const InputDecoration(labelText: 'Description'),
                onSaved: (v) => _description = v ?? '',
              ),
              const SizedBox(height: 16),
              DropdownButtonFormField<String>(
                initialValue: _category,
                decoration: const InputDecoration(labelText: 'Category'),
                items: const [
                  DropdownMenuItem(value: 'Service', child: Text('Service')),
                  DropdownMenuItem(value: 'Event', child: Text('Event')),
                  DropdownMenuItem(value: 'Item', child: Text('Item')),
                  DropdownMenuItem(
                    value: 'Equipment',
                    child: Text('Equipment'),
                  ),
                  DropdownMenuItem(value: 'Vehicle', child: Text('Vehicle')),
                ],
                onChanged: (v) => setState(() => _category = v!),
              ),
              const SizedBox(height: 16),
              TextFormField(
                decoration: InputDecoration(
                  labelText: _listingType == 'forBook'
                      ? 'Price (per period)'
                      : 'Price',
                ),
                keyboardType: TextInputType.number,
                initialValue: '100',
                onSaved: (v) => _bounty = int.tryParse(v ?? '0') ?? 0,
              ),
              const SizedBox(height: 16),
              TextFormField(
                decoration: const InputDecoration(labelText: 'ZIP Code / Location Filter'),
                keyboardType: TextInputType.number,
                initialValue: '78701',
                onSaved: (v) => _zipCode = v ?? '78701',
              ),

              // Booking duration (only when forRent)
              if (_listingType == 'forBook') ...[
                const SizedBox(height: 16),
                DropdownButtonFormField<String>(
                  initialValue: _bookingDuration ?? 'daily',
                  decoration: const InputDecoration(labelText: 'Booking Period'),
                  items: const [
                    DropdownMenuItem(value: 'hourly', child: Text('Per Hour')),
                    DropdownMenuItem(value: 'daily', child: Text('Per Day')),
                    DropdownMenuItem(value: 'weekly', child: Text('Per Week')),
                    DropdownMenuItem(
                      value: 'monthly',
                      child: Text('Per Month'),
                    ),
                  ],
                  onChanged: (v) => setState(() => _bookingDuration = v),
                ),
              ],

              const SizedBox(height: 16),
              DropdownButtonFormField<String>(
                initialValue: _conversationType,
                decoration: const InputDecoration(labelText: 'Conversation Type'),
                items: const [
                  DropdownMenuItem(value: 'none', child: Text('None (Direct Book)')),
                  DropdownMenuItem(value: 'public', child: Text('Public Chat')),
                  DropdownMenuItem(value: 'team', child: Text('Team-only Chat')),
                ],
                onChanged: (v) => setState(() => _conversationType = v!),
              ),

              const SizedBox(height: 16),
              DropdownButtonFormField<String>(
                initialValue: _selectedTeamId,
                decoration: const InputDecoration(labelText: 'Team'),
                items: const [
                  DropdownMenuItem(value: 'water', child: Text('Water Clan')),
                  DropdownMenuItem(value: 'fire', child: Text('Fire Tribe')),
                  DropdownMenuItem(value: 'earth', child: Text('Earth Guild')),
                  DropdownMenuItem(value: 'wind', child: Text('Wind Order')),
                ],
                onChanged: (v) => setState(() => _selectedTeamId = v!),
              ),
              const SizedBox(height: 24),
              ElevatedButton(
                onPressed: () {
                  if (_formKey.currentState!.validate()) {
                    _formKey.currentState!.save();
                    widget.onSaved(
                      _title,
                      _description,
                      _category,
                      _bounty,
                      _selectedTeamId,
                      _listingType,
                      _listingType == 'forBook'
                          ? (_bookingDuration ?? 'daily')
                          : null,
                      _zipCode,
                      'service',
                    );
                  }
                },
                child: const Text('Publish Listing'),
              ),
              const SizedBox(height: 16),
            ],
          ),
        ),
      ),
    );
  }
}
