import sys

filepath = 'lib/features/teams/presentation/teams_page.dart'
with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace("'forRent'", "'forBook'")
content = content.replace("FOR RENT", "FOR BOOKING")
content = content.replace("rentalDuration", "bookingDuration")
content = content.replace("_rentalDuration", "_bookingDuration")
content = content.replace("Rental duration (only when forBook)", "Booking duration (only when forBook)")
content = content.replace("Rental Period", "Booking Period")

# Handle the emojis which might be mojibake in the file right now
content = content.replace("ðŸ”‘ For Rent", "ðŸ“… Book Service")
content = content.replace("ðŸ”‘", "ðŸ“…")
content = content.replace("For Rent", "Book Service")

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)
