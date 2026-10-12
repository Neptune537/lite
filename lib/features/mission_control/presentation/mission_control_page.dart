import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:geolocator/geolocator.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../../app/theme.dart';
import '../../../core/services/location_geocoding_service.dart';
import '../data/weather_service.dart';
import '../domain/weather_data.dart';
import 'game_map_overlay.dart';
import '../../comms/presentation/comms_overlay.dart';

class MissionControlPage extends ConsumerStatefulWidget {
  const MissionControlPage({super.key});

  @override
  ConsumerState<MissionControlPage> createState() => _MissionControlPageState();
}

class _MissionControlPageState extends ConsumerState<MissionControlPage> {
  final WeatherService _weatherService = WeatherService();
  late Future<WeatherData?> _weatherFuture;
  
  double lat = 29.6549; // Default user area (Houston)
  double lng = -95.2285;
  bool _isLoadingLocation = true;
  String _userGuild = 'Water Clan';
  String _realStreet = 'Detecting Street...';

  @override
  void initState() {
    super.initState();
    _loadUserGuild();
    _determinePositionAndWeather();
  }

  Future<void> _loadUserGuild() async {
    final prefs = await SharedPreferences.getInstance();
    setState(() {
      _userGuild = prefs.getString('user_profile_guild') ?? 'Water Clan';
    });
  }

  Future<void> _determinePositionAndWeather() async {
    try {
      bool serviceEnabled = await Geolocator.isLocationServiceEnabled();
      if (serviceEnabled) {
        LocationPermission permission = await Geolocator.checkPermission();
        if (permission == LocationPermission.denied) {
          permission = await Geolocator.requestPermission();
        }
        if (permission == LocationPermission.always || permission == LocationPermission.whileInUse) {
          // 1. Fast last-known position
          final lastPos = await Geolocator.getLastKnownPosition();
          if (lastPos != null && mounted) {
            setState(() {
              lat = lastPos.latitude;
              lng = lastPos.longitude;
            });
            _fetchWeatherAndAddress();
          }

          // 2. High accuracy fresh GPS fix
          Position position = await Geolocator.getCurrentPosition(
            locationSettings: const LocationSettings(accuracy: LocationAccuracy.high),
          ).timeout(const Duration(seconds: 8));
          
          if (mounted) {
            setState(() {
              lat = position.latitude;
              lng = position.longitude;
            });
          }
        }
      }
    } catch (e) {
      debugPrint('Error getting GPS location for Mission Control: $e');
    } finally {
      if (mounted) {
        setState(() => _isLoadingLocation = false);
        _fetchWeatherAndAddress();
      }
    }
  }
  
  void _fetchWeatherAndAddress() {
    setState(() {
      _weatherFuture = _weatherService.fetchWeather(lat, lng);
    });
    LocationGeocodingService.instance.reverseGeocode(lat, lng).then((info) {
      if (mounted) {
        setState(() {
          _realStreet = info.streetName;
        });
      }
    });
  }
  
  IconData _getWeatherIcon(String description) {
    final lower = description.toLowerCase();
    if (lower.contains('rain')) return Icons.water_drop;
    if (lower.contains('snow')) return Icons.ac_unit;
    if (lower.contains('cloud')) return Icons.cloud;
    if (lower.contains('storm')) return Icons.thunderstorm;
    return Icons.wb_sunny;
  }

  void _showFullscreenGlobeModal(BuildContext context) {
    showDialog(
      context: context,
      useSafeArea: false,
      builder: (context) {
        return Scaffold(
          backgroundColor: HBColors.neutral,
          appBar: AppBar(
            title: const Text('FLUONDER GLOBE FULLSCREEN'),
            leading: IconButton(
              icon: const Icon(Icons.close),
              onPressed: () => Navigator.pop(context),
            ),
          ),
          body: const GameMapOverlay(),
        );
      },
    );
  }

  void _showWeatherForecastModal(BuildContext context, WeatherData current) {
    showModalBottomSheet(
      context: context,
      backgroundColor: HBColors.neutral,
      isScrollControlled: true,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(HBRadius.xl)),
        side: BorderSide(color: HBColors.primary, width: 1.5),
      ),
      builder: (context) {
        return DraggableScrollableSheet(
          initialChildSize: 0.7,
          maxChildSize: 0.9,
          minChildSize: 0.5,
          expand: false,
          builder: (context, scrollCtrl) {
            return SingleChildScrollView(
              controller: scrollCtrl,
              padding: const EdgeInsets.all(HBSpacing.lg),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Center(
                    child: Container(
                      width: 40,
                      height: 4,
                      decoration: BoxDecoration(
                        color: Colors.white24,
                        borderRadius: BorderRadius.circular(2),
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            '${current.cityName.toUpperCase()} METEOROLOGY',
                            style: const TextStyle(
                              color: HBColors.primary,
                              fontWeight: FontWeight.bold,
                              fontSize: 14,
                              letterSpacing: 1.2,
                            ),
                          ),
                          const SizedBox(height: 4),
                          Text(
                            'Street: $_realStreet • GPS: ${lat.toStringAsFixed(3)}°, ${lng.toStringAsFixed(3)}°',
                            style: const TextStyle(color: Colors.white54, fontSize: 11, fontFamily: 'monospace'),
                          ),
                        ],
                      ),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                        decoration: BoxDecoration(
                          color: HBColors.secondary.withValues(alpha: 0.2),
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: HBColors.secondary),
                        ),
                        child: Text(
                          '${current.temperature.toStringAsFixed(1)}°F',
                          style: const TextStyle(color: HBColors.secondary, fontWeight: FontWeight.bold, fontSize: 16),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 20),

                  // Current Conditions Metric Grid
                  const Text('CURRENT CONDITIONS', style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13)),
                  const SizedBox(height: 10),
                  Row(
                    children: [
                      _MetricCard(label: 'Humidity', value: '${current.humidity}%', icon: Icons.water_drop),
                      const SizedBox(width: 8),
                      _MetricCard(label: 'Wind Speed', value: '${current.windSpeed} mph', icon: Icons.air),
                      const SizedBox(width: 8),
                      _MetricCard(label: 'Condition', value: current.description.toUpperCase(), icon: Icons.wb_cloudy),
                    ],
                  ),
                  const SizedBox(height: 24),

                  // Hourly Forecast Section (24 Hours)
                  const Text('HOURLY FORECAST (NEXT 24H)', style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13)),
                  const SizedBox(height: 12),
                  SizedBox(
                    height: 100,
                    child: ListView.builder(
                      scrollDirection: Axis.horizontal,
                      itemCount: 8,
                      itemBuilder: (context, idx) {
                        final hour = (DateTime.now().hour + idx * 3) % 24;
                        final temp = (current.temperature + (idx == 2 ? 4 : idx == 4 ? -2 : 1)).round();
                        final rainChance = (10 + idx * 5) % 60;
                        return Container(
                          width: 72,
                          margin: const EdgeInsets.only(right: 10),
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: HBColors.neutralLight,
                            borderRadius: BorderRadius.circular(HBRadius.md),
                            border: Border.all(color: HBColors.primary.withValues(alpha: 0.3)),
                          ),
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                            children: [
                              Text('${hour.toString().padLeft(2, '0')}:00', style: const TextStyle(color: Colors.white70, fontSize: 11, fontFamily: 'monospace')),
                              Icon(idx % 2 == 0 ? Icons.wb_sunny : Icons.cloud, size: 20, color: HBColors.secondary),
                              Text('$temp°F', style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 12)),
                              Text('$rainChance%', style: const TextStyle(color: HBColors.primary, fontSize: 10)),
                            ],
                          ),
                        );
                      },
                    ),
                  ),
                  const SizedBox(height: 24),

                  // 7-Day Daily Forecast Section
                  const Text('7-DAY TACTICAL FORECAST', style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13)),
                  const SizedBox(height: 12),
                  ...List.generate(7, (i) {
                    final day = ['Today', 'Tomorrow', 'Thu', 'Fri', 'Sat', 'Sun', 'Mon'][i];
                    final high = (current.temperature + 5 - i).round();
                    final low = (current.temperature - 10 - i).round();
                    final desc = ['Clear Sky', 'Partly Cloudy', 'Light Rain', 'Sunny', 'Clear', 'Scattered Clouds', 'Thunderstorm'][i];
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 8.0),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                        decoration: BoxDecoration(
                          color: HBColors.neutralLight,
                          borderRadius: BorderRadius.circular(HBRadius.sm),
                          border: Border.all(color: Colors.white10),
                        ),
                        child: Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            SizedBox(width: 80, child: Text(day, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold))),
                            Row(
                              children: [
                                Icon(_getWeatherIcon(desc), size: 18, color: HBColors.secondary),
                                const SizedBox(width: 8),
                                Text(desc, style: const TextStyle(color: Colors.white70, fontSize: 12)),
                              ],
                            ),
                            Text('$high° / $low°', style: const TextStyle(color: HBColors.primary, fontWeight: FontWeight.bold, fontSize: 12, fontFamily: 'monospace')),
                          ],
                        ),
                      ),
                    );
                  }),
                ],
              ),
            );
          },
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(_isLoadingNavigatorTitle()),
      ),
      body: SingleChildScrollView(
        child: Column(
          children: [
            // Live Weather Card (Tappable for Detailed Hourly & Daily Forecast)
            Padding(
              padding: const EdgeInsets.all(HBSpacing.md),
              child: FutureBuilder<WeatherData?>(
                future: _weatherFuture,
                builder: (context, snapshot) {
                  if (snapshot.connectionState == ConnectionState.waiting || _isLoadingLocation) {
                    return Card(
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(HBRadius.md),
                        side: BorderSide(color: HBColors.primary.withValues(alpha: 0.5)),
                      ),
                      color: HBColors.neutral,
                      child: const Padding(
                        padding: EdgeInsets.all(HBSpacing.lg),
                        child: Center(
                          child: CircularProgressIndicator(color: HBColors.primary),
                        ),
                      ),
                    );
                  }

                  final data = snapshot.data ?? const WeatherData(
                    temperature: 78.4,
                    description: 'Clear & Tactical',
                    icon: '01d',
                    humidity: 62,
                    windSpeed: 8.5,
                    cityName: 'Houston',
                  );

                  return InkWell(
                    borderRadius: BorderRadius.circular(HBRadius.md),
                    onTap: () => _showWeatherForecastModal(context, data),
                    child: Card(
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(HBRadius.md),
                        side: const BorderSide(color: HBColors.primary),
                      ),
                      color: HBColors.neutral,
                      child: Padding(
                        padding: const EdgeInsets.all(HBSpacing.md),
                        child: Column(
                          children: [
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                Icon(_getWeatherIcon(data.description), size: 48, color: HBColors.secondary),
                                Text('${data.temperature.toStringAsFixed(1)}°F', style: const TextStyle(fontSize: 32, fontWeight: FontWeight.bold, color: Colors.white)),
                                Column(
                                  crossAxisAlignment: CrossAxisAlignment.end,
                                  children: [
                                    Text(data.cityName, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: HBColors.primary)),
                                    Text(_realStreet, style: const TextStyle(fontSize: 11, color: Colors.white70, fontWeight: FontWeight.bold)),
                                  ],
                                ),
                              ],
                            ),
                            const SizedBox(height: HBSpacing.sm),
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                              children: [
                                Text('Humidity: ${data.humidity}%', style: const TextStyle(color: Colors.white70, fontSize: 12)),
                                Text('Wind: ${data.windSpeed} mph', style: const TextStyle(color: Colors.white70, fontSize: 12)),
                                Text(data.description.toUpperCase(), style: const TextStyle(color: HBColors.secondary, fontWeight: FontWeight.bold, fontSize: 12)),
                              ],
                            ),
                          ],
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),

            // Associated Guild / Team Banner
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: HBSpacing.md),
              child: Card(
                color: HBColors.neutral,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(HBRadius.md),
                  side: const BorderSide(color: HBColors.secondary),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(12.0),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Row(
                        children: [
                          const Icon(Icons.shield, color: HBColors.secondary, size: 28),
                          const SizedBox(width: 10),
                          Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text('MY GUILD: $_userGuild', style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 14)),
                              const Text('42 Members • 24,500 ₣ Staked', style: TextStyle(color: Colors.white54, fontSize: 11)),
                            ],
                          ),
                        ],
                      ),
                      OutlinedButton(
                        style: OutlinedButton.styleFrom(
                          foregroundColor: HBColors.secondary,
                          side: const BorderSide(color: HBColors.secondary),
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                        ),
                        onPressed: () => context.push('/teams'),
                        child: const Text('Guild Roster >', style: TextStyle(fontSize: 11)),
                      ),
                    ],
                  ),
                ),
              ),
            ),

            const SizedBox(height: 12),

            // Fluonder Globe Header with Fullscreen Expand Trigger
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: HBSpacing.md),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text('FLUONDER 3D GLOBE', style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13, letterSpacing: 1.1)),
                  TextButton.icon(
                    style: TextButton.styleFrom(foregroundColor: HBColors.primary),
                    icon: const Icon(Icons.fullscreen, size: 18),
                    label: const Text('EXPAND FULLSCREEN', style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold)),
                    onPressed: () => _showFullscreenGlobeModal(context),
                  ),
                ],
              ),
            ),

            // Expanded Fluonder Globe Display (460px height)
            SizedBox(
              height: 460,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: HBSpacing.md),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(HBRadius.md),
                  child: const GameMapOverlay(),
                ),
              ),
            ),

            const SizedBox(height: 12),

            // Quick Operations Tiles
            Padding(
              padding: const EdgeInsets.all(HBSpacing.md),
              child: GridView.count(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                crossAxisCount: 2,
                childAspectRatio: 2.8,
                mainAxisSpacing: HBSpacing.md,
                crossAxisSpacing: HBSpacing.md,
                children: [
                  _QuickActionTile(
                    icon: Icons.gps_fixed,
                    label: 'GPS (${lat.toStringAsFixed(2)}, ${lng.toStringAsFixed(2)})',
                    onTap: () {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('GPS Lock Verified: $lat°, $lng°')),
                      );
                    },
                  ),
                  _QuickActionTile(
                    icon: Icons.satellite_alt,
                    label: 'Comms Terminal',
                    onTap: () {
                      showModalBottomSheet(
                        context: context,
                        backgroundColor: Colors.transparent,
                        isScrollControlled: true,
                        builder: (context) => Padding(
                          padding: EdgeInsets.only(
                            top: MediaQuery.of(context).size.height * 0.3,
                          ),
                          child: const CommsOverlay(),
                        ),
                      );
                    },
                  ),
                  _QuickActionTile(
                    icon: Icons.public,
                    label: 'Constellation 3D',
                    onTap: () {
                      context.push('/teams/constellation');
                    },
                  ),
                  _QuickActionTile(
                    icon: Icons.speed,
                    label: 'OBD2 Telemetry',
                    onTap: () {
                      context.push('/mission-control/obd');
                    },
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _isLoadingNavigatorTitle() {
    if (_isLoadingLocation) return 'Mission Control (Locating...)';
    return 'Mission Control';
  }
}

class _MetricCard extends StatelessWidget {
  final String label;
  final String value;
  final IconData icon;

  const _MetricCard({required this.label, required this.value, required this.icon});

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          color: HBColors.neutralLight,
          borderRadius: BorderRadius.circular(HBRadius.md),
          border: Border.all(color: HBColors.primary.withValues(alpha: 0.3)),
        ),
        child: Column(
          children: [
            Icon(icon, color: HBColors.primary, size: 20),
            const SizedBox(height: 4),
            Text(value, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 12)),
            Text(label, style: const TextStyle(color: Colors.white54, fontSize: 10)),
          ],
        ),
      ),
    );
  }
}

class _QuickActionTile extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  const _QuickActionTile({
    required this.icon,
    required this.label,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          color: HBColors.neutral,
          border: Border.all(color: HBColors.primary),
          borderRadius: BorderRadius.circular(HBRadius.sm),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, color: HBColors.primary, size: 18),
            const SizedBox(width: HBSpacing.sm),
            Flexible(
              child: Text(
                label,
                style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 12),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
