import 'package:flutter/material.dart';
import 'cyan_radio_widget.dart';

class WalkieTalkieTab extends StatelessWidget {
  const WalkieTalkieTab({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: SingleChildScrollView(
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: CyanRadioWidget(),
      ),
    );
  }
}
