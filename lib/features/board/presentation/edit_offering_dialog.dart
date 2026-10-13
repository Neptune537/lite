import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../app/theme.dart';
import '../../../core/services/local_database_service.dart';
import 'local_offerings_provider.dart';

class EditOfferingDialog extends ConsumerStatefulWidget {
  final Map<String, dynamic> offering;
  const EditOfferingDialog({super.key, required this.offering});

  @override
  ConsumerState<EditOfferingDialog> createState() => _EditOfferingDialogState();
}

class _EditOfferingDialogState extends ConsumerState<EditOfferingDialog> {
  final _formKey = GlobalKey<FormState>();
  late TextEditingController _titleCtrl;
  late TextEditingController _descCtrl;
  late TextEditingController _bountyCtrl;
  late TextEditingController _zipCtrl;

  @override
  void initState() {
    super.initState();
    final dataStr = widget.offering['data'] as String? ?? '{}';
    Map<String, dynamic> decoded = {};
    try {
      decoded = jsonDecode(dataStr) as Map<String, dynamic>;
    } catch (_) {}

    _titleCtrl = TextEditingController(
      text: decoded['title']?.toString() ?? '',
    );
    _descCtrl = TextEditingController(
      text: decoded['description']?.toString() ?? '',
    );
    _bountyCtrl = TextEditingController(
      text: decoded['bounty']?.toString().replaceAll('\$', '') ?? decoded['budgetAmount']?.toString() ?? '100',
    );
    _zipCtrl = TextEditingController(
      text: decoded['zipCode']?.toString() ?? '78701',
    );
  }

  @override
  void dispose() {
    _titleCtrl.dispose();
    _descCtrl.dispose();
    _bountyCtrl.dispose();
    _zipCtrl.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    if (!_formKey.currentState!.validate()) return;

    final dbService = LocalDatabaseService.instance;
    final db = await dbService.database;

    final dataStr = widget.offering['data'] as String? ?? '{}';
    Map<String, dynamic> decoded = {};
    try {
      decoded = jsonDecode(dataStr) as Map<String, dynamic>;
    } catch (_) {}

    final bountyText = _bountyCtrl.text.trim();
    decoded['title'] = _titleCtrl.text.trim();
    decoded['description'] = _descCtrl.text.trim();
    decoded['bounty'] = bountyText.startsWith('\$') ? bountyText : '\$$bountyText';
    decoded['budgetAmount'] = double.tryParse(bountyText.replaceAll('\$', '')) ?? 100.0;
    decoded['zipCode'] = _zipCtrl.text.trim();

    await db.update(
      'tasks',
      {'data': jsonEncode(decoded)},
      where: 'id = ?',
      whereArgs: [widget.offering['id']],
    );

    if (mounted) {
      ref.read(localOfferingsNotifierProvider.notifier).refresh();
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Listing updated successfully!'), backgroundColor: Colors.green),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: HBColors.neutral,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(HBRadius.lg),
        side: const BorderSide(color: HBColors.primary, width: 1.5),
      ),
      title: const Text('Edit Task Listing', style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold)),
      content: Form(
        key: _formKey,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextFormField(
                controller: _titleCtrl,
                style: const TextStyle(color: Colors.white),
                decoration: const InputDecoration(labelText: 'Task Title'),
                validator: (v) => v!.isEmpty ? 'Required' : null,
              ),
              const SizedBox(height: 16),
              TextFormField(
                controller: _descCtrl,
                style: const TextStyle(color: Colors.white),
                decoration: const InputDecoration(labelText: 'Description'),
                maxLines: 3,
                validator: (v) => v!.isEmpty ? 'Required' : null,
              ),
              const SizedBox(height: 16),
              TextFormField(
                controller: _bountyCtrl,
                style: const TextStyle(color: Colors.white),
                decoration: const InputDecoration(labelText: 'Bounty Amount (\$)', prefixText: '\$ '),
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
              ),
              const SizedBox(height: 16),
              TextFormField(
                controller: _zipCtrl,
                style: const TextStyle(color: Colors.white),
                decoration: const InputDecoration(labelText: 'ZIP Code / Location'),
                keyboardType: TextInputType.number,
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        ElevatedButton(
          style: ElevatedButton.styleFrom(backgroundColor: HBColors.primary),
          onPressed: _save,
          child: const Text('Save Changes'),
        ),
      ],
    );
  }
}
