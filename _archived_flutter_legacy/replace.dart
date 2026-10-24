import 'dart:io';

void main() {
  final file = File('lib/features/teams/presentation/teams_page.dart');
  String content = file.readAsStringSync();

  content = content.replaceAll("'forRent'", "'forBook'");
  content = content.replaceAll("FOR RENT", "FOR BOOKING");
  content = content.replaceAll("rentalDuration", "bookingDuration");
  content = content.replaceAll("_rentalDuration", "_bookingDuration");
  content = content.replaceAll("Rental duration", "Booking duration");
  content = content.replaceAll("Rental Period", "Booking Period");
  content = content.replaceAll("ðŸ”‘ For Rent", "ðŸ“… Book Service");
  content = content.replaceAll("For Rent", "Book Service");
  content = content.replaceAll("🔑 For Rent", "📅 Book Service");

  file.writeAsStringSync(content);
  print('Replaced strings in Dart.');
}
