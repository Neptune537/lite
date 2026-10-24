import 'dart:io';

void main() {
  final file = File('lib/features/teams/presentation/teams_page.dart');
  String content = file.readAsStringSync();

  content = content.replaceAll(
    "_zipCode,\n                      );",
    "_zipCode,\n                        _conversationType,\n                      );"
  );

  file.writeAsStringSync(content);
  print('Replaced strings in Dart.');
}
