import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;

class AddressInfo {
  final String streetName;
  final String neighborhood;
  final String city;
  final String zipCode;

  const AddressInfo({
    required this.streetName,
    required this.neighborhood,
    required this.city,
    required this.zipCode,
  });

  factory AddressInfo.fallback(double lat, double lng) {
    return AddressInfo(
      streetName: 'GPS Sector (${lat.toStringAsFixed(3)}°, ${lng.toStringAsFixed(3)}°)',
      neighborhood: 'Local Area',
      city: 'Houston',
      zipCode: '78701',
    );
  }
}

class LocationGeocodingService {
  static final LocationGeocodingService instance = LocationGeocodingService._internal();
  LocationGeocodingService._internal();

  final Map<String, AddressInfo> _cache = {};

  Future<AddressInfo> reverseGeocode(double lat, double lng) async {
    final cacheKey = '${lat.toStringAsFixed(3)}_${lng.toStringAsFixed(3)}';
    if (_cache.containsKey(cacheKey)) {
      return _cache[cacheKey]!;
    }

    // Primary: BigDataCloud Free Reverse Geocoding API
    try {
      final res = await http.get(
        Uri.parse('https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=$lat&longitude=$lng&localityLanguage=en'),
      ).timeout(const Duration(seconds: 5));

      if (res.statusCode == 200) {
        final json = jsonDecode(res.body) as Map<String, dynamic>;
        
        String road = '';
        final localityInfo = json['localityInfo'] as Map<String, dynamic>?;
        if (localityInfo != null) {
          final informatives = localityInfo['informative'] as List?;
          if (informatives != null) {
            for (var info in informatives) {
              final name = info['name']?.toString() ?? '';
              final description = info['description']?.toString() ?? '';
              if (description.toLowerCase().contains('street') ||
                  description.toLowerCase().contains('road') ||
                  description.toLowerCase().contains('avenue') ||
                  description.toLowerCase().contains('drive') ||
                  description.toLowerCase().contains('freeway') ||
                  description.toLowerCase().contains('highway') ||
                  name.toLowerCase().contains('st') ||
                  name.toLowerCase().contains('rd') ||
                  name.toLowerCase().contains('dr') ||
                  name.toLowerCase().contains('fwy')) {
                road = name;
                break;
              }
            }
          }
        }

        final city = json['city']?.toString() ?? json['locality']?.toString() ?? json['principalSubdivision']?.toString() ?? 'Houston';
        final suburb = json['locality']?.toString() ?? json['localityInfo']?['sublocality']?.toString() ?? city;
        final postcode = json['postcode']?.toString() ?? '78701';

        if (road.isEmpty) {
          road = suburb.isNotEmpty ? suburb : '$city Main Rd';
        }

        final result = AddressInfo(
          streetName: road,
          neighborhood: suburb,
          city: city,
          zipCode: postcode,
        );

        _cache[cacheKey] = result;
        return result;
      }
    } catch (e) {
      debugPrint('[LocationGeocodingService] BigDataCloud note: $e');
    }

    // Secondary: OpenStreetMap Nominatim Reverse Geocoding
    try {
      final res = await http.get(
        Uri.parse('https://nominatim.openstreetmap.org/reverse?format=json&lat=$lat&lon=$lng&zoom=18&addressdetails=1'),
        headers: {'User-Agent': 'HeavenlyBondApp/1.0 (Mobile Client)'},
      ).timeout(const Duration(seconds: 5));

      if (res.statusCode == 200) {
        final json = jsonDecode(res.body) as Map<String, dynamic>;
        final address = json['address'] as Map<String, dynamic>? ?? {};

        final road = address['road'] ??
            address['pedestrian'] ??
            address['highway'] ??
            address['suburb'] ??
            address['neighbourhood'] ??
            'Main Street';

        final suburb = address['suburb'] ?? address['neighbourhood'] ?? address['quarter'] ?? 'Local Sector';
        final city = address['city'] ?? address['town'] ?? address['village'] ?? address['county'] ?? 'Houston';
        final zip = address['postcode'] ?? '78701';

        final result = AddressInfo(
          streetName: road.toString(),
          neighborhood: suburb.toString(),
          city: city.toString(),
          zipCode: zip.toString(),
        );

        _cache[cacheKey] = result;
        return result;
      }
    } catch (e) {
      debugPrint('[LocationGeocodingService] Nominatim note: $e');
    }

    final fallback = AddressInfo.fallback(lat, lng);
    _cache[cacheKey] = fallback;
    return fallback;
  }
}
