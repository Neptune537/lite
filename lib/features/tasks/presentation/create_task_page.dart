import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:uuid/uuid.dart';
import '../../../app/theme.dart';
import '../../../core/blockchain/fluoridian_service.dart';
import '../../../core/blockchain/models/network_event.dart';
import '../../board/presentation/local_offerings_provider.dart';
import '../../teams/presentation/team_providers.dart';

/// Sprint 2: Multi-step Task Creation Wizard
/// Target Platforms: Android and Web.
class CreateTaskPage extends ConsumerStatefulWidget {
  const CreateTaskPage({super.key});

  @override
  ConsumerState<CreateTaskPage> createState() => _CreateTaskPageState();
}

class _CreateTaskPageState extends ConsumerState<CreateTaskPage> {
  int _currentStep = 0;

  // Step 1: Basic Info
  final _titleCtrl = TextEditingController();
  final _descCtrl = TextEditingController();
  String _selectedCategory = 'Home Improvement';

  // Step 2: Logistics
  final _locationCtrl = TextEditingController(text: 'Austin, TX (Local Area)');
  final _zipCtrl = TextEditingController(text: '78701');
  DateTime _desiredDate = DateTime.now().add(const Duration(days: 3));
  int _workerCount = 1;

  // Step 3: Requirements
  final List<String> _skills = ['General Labor'];
  final _skillInputCtrl = TextEditingController();
  bool _requireId = true;
  bool _requireInsurance = false;

  // Step 4: Funding & Tithe
  final _bountyCtrl = TextEditingController(text: '150.00');

  final List<String> _categories = [
    'Home Improvement',
    'Logistics & Delivery',
    'Automotive & OBD',
    'Event Support',
    'Field Cleanup',
    'Tech & Hardware',
  ];

  @override
  void dispose() {
    _titleCtrl.dispose();
    _descCtrl.dispose();
    _locationCtrl.dispose();
    _zipCtrl.dispose();
    _skillInputCtrl.dispose();
    _bountyCtrl.dispose();
    super.dispose();
  }

  void _nextStep() {
    if (_currentStep < 4) {
      setState(() => _currentStep++);
    } else {
      _publishTask();
    }
  }

  void _prevStep() {
    if (_currentStep > 0) {
      setState(() => _currentStep--);
    }
  }

  Future<void> _publishTask() async {
    final bounty = double.tryParse(_bountyCtrl.text.trim()) ?? 100.0;
    final taskId = const Uuid().v4();
    final teamId = ref.read(selectedTeamProvider) ?? 'water';

    final taskPayload = {
      'id': taskId,
      'creatorId': teamId,
      'title': _titleCtrl.text.trim().isEmpty ? 'Community Task' : _titleCtrl.text.trim(),
      'description': _descCtrl.text.trim(),
      'category': _selectedCategory,
      'locationLabel': _locationCtrl.text.trim(),
      'zipCode': _zipCtrl.text.trim().isEmpty ? '78701' : _zipCtrl.text.trim(),
      'desiredCompletionDate': _desiredDate.toIso8601String(),
      'workerCount': _workerCount,
      'requiredSkills': _skills,
      'requireId': _requireId,
      'requireInsurance': _requireInsurance,
      'budgetAmount': bounty,
      'bounty': '\$$bounty',
      'status': 'PUBLISHED',
      'createdAt': DateTime.now().toUtc().toIso8601String(),
    };

    // Broadcast over Fluoridian P2P relay and persist in parallel SQLite storage
    final fluoridian = ref.read(fluoridianServiceProvider);
    await fluoridian.signAndBroadcast(
      eventType: NetworkEventType.taskCreated,
      payload: taskPayload,
    );

    // Refresh local board provider so the new task appears immediately in all tabs
    ref.read(localOfferingsNotifierProvider.notifier).refresh();

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Task published successfully!'),
          backgroundColor: Colors.green,
        ),
      );
      context.go('/board');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Create Task Wizard'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
      ),
      body: Stepper(
        currentStep: _currentStep,
        onStepContinue: _nextStep,
        onStepCancel: _prevStep,
        controlsBuilder: (context, details) {
          final isLastStep = _currentStep == 4;
          return Padding(
            padding: const EdgeInsets.only(top: 24.0),
            child: Row(
              children: [
                FilledButton.icon(
                  onPressed: details.onStepContinue,
                  icon: Icon(isLastStep ? Icons.check_circle : Icons.arrow_forward),
                  label: Text(isLastStep ? 'Publish to Network' : 'Next'),
                ),
                if (_currentStep > 0) ...[
                  const SizedBox(width: 12),
                  OutlinedButton(
                    onPressed: details.onStepCancel,
                    child: const Text('Back'),
                  ),
                ],
              ],
            ),
          );
        },
        steps: [
          // Step 1: Basic Info
          Step(
            title: const Text('Basic Info'),
            isActive: _currentStep >= 0,
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                TextField(
                  controller: _titleCtrl,
                  decoration: const InputDecoration(
                    labelText: 'Task Title',
                    hintText: 'e.g., Clear brush from vacant lot',
                  ),
                ),
                const SizedBox(height: 16),
                DropdownButtonFormField<String>(
                  initialValue: _selectedCategory,
                  items: _categories.map((c) => DropdownMenuItem(value: c, child: Text(c))).toList(),
                  onChanged: (val) => setState(() => _selectedCategory = val!),
                  decoration: const InputDecoration(labelText: 'Category'),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _descCtrl,
                  maxLines: 3,
                  decoration: const InputDecoration(
                    labelText: 'Description',
                    hintText: 'Provide details about the work required...',
                  ),
                ),
              ],
            ),
          ),

          // Step 2: Logistics
          Step(
            title: const Text('Logistics'),
            isActive: _currentStep >= 1,
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                TextField(
                  controller: _locationCtrl,
                  decoration: const InputDecoration(
                    labelText: 'Location / Area',
                    prefixIcon: Icon(Icons.location_on),
                  ),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _zipCtrl,
                  keyboardType: TextInputType.number,
                  decoration: const InputDecoration(
                    labelText: 'ZIP Code / Location Filter',
                    hintText: 'e.g., 78701',
                    prefixIcon: Icon(Icons.map),
                  ),
                ),
                const SizedBox(height: 16),
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading: const Icon(Icons.calendar_today),
                  title: const Text('Desired Date'),
                  subtitle: Text('${_desiredDate.toLocal()}'.split(' ')[0]),
                  trailing: TextButton(
                    onPressed: () async {
                      final picked = await showDatePicker(
                        context: context,
                        initialDate: _desiredDate,
                        firstDate: DateTime.now(),
                        lastDate: DateTime.now().add(const Duration(days: 90)),
                      );
                      if (picked != null) setState(() => _desiredDate = picked);
                    },
                    child: const Text('Change'),
                  ),
                ),
                const SizedBox(height: 16),
                Row(
                  children: [
                    const Text('Workers Needed:'),
                    const SizedBox(width: 16),
                    IconButton(
                      icon: const Icon(Icons.remove_circle_outline),
                      onPressed: _workerCount > 1 ? () => setState(() => _workerCount--) : null,
                    ),
                    Text('$_workerCount', style: const TextStyle(fontWeight: FontWeight.bold)),
                    IconButton(
                      icon: const Icon(Icons.add_circle_outline),
                      onPressed: () => setState(() => _workerCount++),
                    ),
                  ],
                ),
              ],
            ),
          ),

          // Step 3: Requirements
          Step(
            title: const Text('Requirements'),
            isActive: _currentStep >= 2,
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _skillInputCtrl,
                        decoration: const InputDecoration(
                          labelText: 'Add Skill Requirement',
                          hintText: 'e.g., Heavy Lifting, Welding',
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    IconButton(
                      icon: const Icon(Icons.add),
                      onPressed: () {
                        if (_skillInputCtrl.text.trim().isNotEmpty) {
                          setState(() {
                            _skills.add(_skillInputCtrl.text.trim());
                            _skillInputCtrl.clear();
                          });
                        }
                      },
                    ),
                  ],
                ),
                Wrap(
                  spacing: 8,
                  children: _skills.map((s) => Chip(
                    label: Text(s),
                    onDeleted: () => setState(() => _skills.remove(s)),
                  )).toList(),
                ),
                const SizedBox(height: 16),
                CheckboxListTile(
                  value: _requireId,
                  onChanged: (val) => setState(() => _requireId = val ?? false),
                  title: const Text('Require Verified Photo ID'),
                ),
                CheckboxListTile(
                  value: _requireInsurance,
                  onChanged: (val) => setState(() => _requireInsurance = val ?? false),
                  title: const Text('Require Liability Insurance'),
                ),
              ],
            ),
          ),

          // Step 4: Funding
          Step(
            title: const Text('Funding & Tithe'),
            isActive: _currentStep >= 3,
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                TextField(
                  controller: _bountyCtrl,
                  keyboardType: const TextInputType.numberWithOptions(decimal: true),
                  decoration: const InputDecoration(
                    labelText: 'Total Bounty Amount (USD)',
                    prefixText: '\$ ',
                  ),
                ),
                const SizedBox(height: 12),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.amber.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.amber.withValues(alpha: 0.3)),
                  ),
                  child: const Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('TitheX Escrow Breakdown', style: TextStyle(fontWeight: FontWeight.bold)),
                      SizedBox(height: 4),
                      Text('• 90% Allocated directly to Worker Bounty upon completion'),
                      Text('• 10% Community Tithe locked in on-chain staking treasury'),
                    ],
                  ),
                ),
              ],
            ),
          ),

          // Step 5: Review & Publish
          Step(
            title: const Text('Review & Publish'),
            isActive: _currentStep >= 4,
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Title: ${_titleCtrl.text.isEmpty ? "Community Task" : _titleCtrl.text}', style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
                const SizedBox(height: 8),
                Text('Category: $_selectedCategory'),
                Text('Location: ${_locationCtrl.text} (ZIP: ${_zipCtrl.text})'),
                Text('Desired Date: ${_desiredDate.toLocal()}'.split(' ')[0]),
                Text('Bounty: \$${_bountyCtrl.text}'),
                const SizedBox(height: 12),
                const Text(
                  'Publishing will broadcast an Ed25519-signed Event Envelope to the Fluoridian network and initialize the smart escrow contract.',
                  style: TextStyle(fontSize: 12, color: HBColors.onSurfaceVariantDark),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
