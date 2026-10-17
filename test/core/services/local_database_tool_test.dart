import 'package:flutter_test/flutter_test.dart';
import 'package:hblite/core/services/local_database_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Local SQLite Database Tool Tests', () {
    test('LocalDatabaseService instance returns persistent singleton', () {
      final db1 = LocalDatabaseService.instance;
      final db2 = LocalDatabaseService.instance;

      expect(identical(db1, db2), isTrue);
    });

    test('LocalDatabaseService exposes core database access contract', () {
      final service = LocalDatabaseService.instance;
      expect(service, isNotNull);
      expect(service.database, isA<Future>());
    });
  });
}
