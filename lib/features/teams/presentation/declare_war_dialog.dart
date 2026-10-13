import 'package:flutter/material.dart';
import '../../../app/theme.dart';

class DeclareWarDialog extends StatefulWidget {
  final String myTeamName;
  final String enemyTeamName;
  final String enemyTeamId;
  final Function(String message) onDeclare;

  const DeclareWarDialog({
    super.key,
    required this.myTeamName,
    required this.enemyTeamName,
    required this.enemyTeamId,
    required this.onDeclare,
  });

  @override
  State<DeclareWarDialog> createState() => _DeclareWarDialogState();
}

class _DeclareWarDialogState extends State<DeclareWarDialog> {
  final _messageController = TextEditingController();

  @override
  void dispose() {
    _messageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: HBColors.neutralLight,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(HBRadius.lg),
        side: const BorderSide(color: HBColors.error, width: 2),
      ),
      title: const Row(
        children: [
          Icon(Icons.military_tech, color: HBColors.error, size: 28),
          SizedBox(width: HBSpacing.sm),
          Expanded(
            child: Text(
              'DECLARE WAR',
              style: TextStyle(
                color: HBColors.error,
                fontWeight: FontWeight.bold,
                letterSpacing: 2,
              ),
            ),
          ),
        ],
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          RichText(
            text: TextSpan(
              style: const TextStyle(color: Colors.white70, fontSize: 14),
              children: [
                TextSpan(
                  text: widget.myTeamName,
                  style: const TextStyle(
                    color: HBColors.primary,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const TextSpan(text: ' declares war against '),
                TextSpan(
                  text: widget.enemyTeamName,
                  style: const TextStyle(
                    color: HBColors.error,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const TextSpan(text: '!'),
              ],
            ),
          ),
          const SizedBox(height: HBSpacing.md),
          const Text(
            'This action cannot be undone. Are you sure?',
            style: TextStyle(color: Colors.white54, fontSize: 12),
          ),
          const SizedBox(height: HBSpacing.md),
          TextField(
            controller: _messageController,
            style: const TextStyle(color: Colors.white),
            maxLines: 2,
            decoration: InputDecoration(
              hintText: 'War declaration message (optional)',
              hintStyle: const TextStyle(color: Colors.white38),
              filled: true,
              fillColor: HBColors.neutral,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(HBRadius.md),
                borderSide: const BorderSide(color: HBColors.dividerDark),
              ),
            ),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text(
            'Stand Down',
            style: TextStyle(color: Colors.white54),
          ),
        ),
        ElevatedButton(
          onPressed: () {
            widget.onDeclare(_messageController.text);
            Navigator.pop(context);
          },
          style: ElevatedButton.styleFrom(
            backgroundColor: HBColors.error,
            foregroundColor: Colors.white,
          ),
          child: const Text('⚔️ DECLARE WAR'),
        ),
      ],
    );
  }
}
