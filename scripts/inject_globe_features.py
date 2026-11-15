import sys

with open(r'C:\Users\blue-\projects\lite\crates\hblite_desktop\src\gui.rs', 'r', encoding='utf-8') as f:
    code = f.read()

# Let's insert SGP4 Satellite orbits and Snap Map heatmaps into render_nautilus_holographic_globe
# Right before `// 4. Draw Authoritative Geodesic Loci & VR Broadcast Pins`

satellite_and_heatmaps_code = """            // --- SGP4 Satellite Orbital Tracks (CSS Tianhe & Leopard Arcs - Image 4) ---
            // 1. CSS (Tianhe) SGP4 Orbit (~395 km equivalent trail, red arc)
            let css_points = 64;
            let css_inc = 41.5_f32.to_radians();
            for i in 0..css_points {
                let t1 = (i as f32 / css_points as f32) * std::f32::consts::TAU;
                let t2 = ((i + 1) as f32 / css_points as f32) * std::f32::consts::TAU;

                let effective_lon1 = t1 + rot * 0.4;
                let lat1 = (t1.sin() * css_inc.sin()).asin();
                let effective_lon2 = t2 + rot * 0.4;
                let lat2 = (t2.sin() * css_inc.sin()).asin();

                let is_front = effective_lon1.cos() > -0.2;
                if is_front {
                    let r_sat = radius + 14.0;
                    let px1 = center.x + r_sat * lat1.cos() * effective_lon1.sin();
                    let py1 = center.y - r_sat * (lat1.sin() * tilt.cos() - lat1.cos() * effective_lon1.cos() * tilt.sin());
                    let px2 = center.x + r_sat * lat2.cos() * effective_lon2.sin();
                    let py2 = center.y - r_sat * (lat2.sin() * tilt.cos() - lat2.cos() * effective_lon2.cos() * tilt.sin());

                    painter.line_segment(
                        [egui::Pos2::new(px1, py1), egui::Pos2::new(px2, py2)],
                        egui::Stroke::new(1.4, egui::Color32::from_rgba_unmultiplied(255, 60, 60, 180)),
                    );

                    // Satellite lead icon & badge at leading edge
                    if i == (css_points * 3 / 4) {
                        painter.circle_filled(egui::Pos2::new(px1, py1), 3.5, egui::Color32::from_rgb(255, 60, 60));
                        painter.text(
                            egui::Pos2::new(px1 + 8.0, py1 - 6.0),
                            egui::Align2::LEFT_BOTTOM,
                            "CSS (TIANHE) 395 km - SGP4 orbit",
                            egui::FontId::proportional(10.0),
                            egui::Color32::from_rgb(255, 200, 200),
                        );
                    }
                }
            }

            // 2. LEOPARD SGP4 Polar Orbit (Cyan arc)
            for i in 0..css_points {
                let t1 = (i as f32 / css_points as f32) * std::f32::consts::TAU;
                let t2 = ((i + 1) as f32 / css_points as f32) * std::f32::consts::TAU;

                let effective_lon1 = (t1.sin() * 0.35) + rot * 0.7;
                let lat1 = t1.cos();
                let effective_lon2 = (t2.sin() * 0.35) + rot * 0.7;
                let lat2 = t2.cos();

                if effective_lon1.cos() > -0.2 {
                    let r_sat = radius + 20.0;
                    let px1 = center.x + r_sat * lat1.cos() * effective_lon1.sin();
                    let py1 = center.y - r_sat * (lat1.sin() * tilt.cos() - lat1.cos() * effective_lon1.cos() * tilt.sin());
                    let px2 = center.x + r_sat * lat2.cos() * effective_lon2.sin();
                    let py2 = center.y - r_sat * (lat2.sin() * tilt.cos() - lat2.cos() * effective_lon2.cos() * tilt.sin());

                    painter.line_segment(
                        [egui::Pos2::new(px1, py1), egui::Pos2::new(px2, py2)],
                        egui::Stroke::new(1.2, egui::Color32::from_rgba_unmultiplied(0, 229, 255, 160)),
                    );

                    if i == (css_points / 2) {
                        painter.circle_filled(egui::Pos2::new(px1, py1), 3.0, egui::Color32::from_rgb(0, 229, 255));
                        painter.text(
                            egui::Pos2::new(px1 + 8.0, py1 - 6.0),
                            egui::Align2::LEFT_BOTTOM,
                            "LEOPARD 480 km",
                            egui::FontId::proportional(9.5),
                            egui::Color32::from_rgb(160, 240, 255),
                        );
                    }
                }
            }

            // --- Snap Map Social Activity Thermal Heatmaps (Image 3) ---
            let hotspots: [(f32, f32, f32, egui::Color32); 5] = [
                (-74.0, 40.7, 16.0, egui::Color32::from_rgba_unmultiplied(255, 60, 30, 140)),  // NYC (Red)
                (-118.2, 34.0, 14.0, egui::Color32::from_rgba_unmultiplied(255, 200, 30, 140)), // LA (Yellow)
                (-0.12, 51.5, 15.0, egui::Color32::from_rgba_unmultiplied(255, 50, 30, 140)),   // London (Red)
                (2.35, 48.8, 12.0, egui::Color32::from_rgba_unmultiplied(255, 180, 20, 120)),   // Paris (Yellow)
                (139.7, 35.6, 18.0, egui::Color32::from_rgba_unmultiplied(255, 40, 20, 160)),  // Tokyo (Intense Red)
            ];

            for (lon_deg, lat_deg, base_r, color) in hotspots {
                let lat_rad = lat_deg.to_radians();
                let lon_rad = lon_deg.to_radians();
                let effective_lon = lon_rad + rot;

                if effective_lon.cos() > -0.1 {
                    let px = center.x + radius * lat_rad.cos() * effective_lon.sin();
                    let py = center.y - radius * (lat_rad.sin() * tilt.cos() - lat_rad.cos() * effective_lon.cos() * tilt.sin());
                    let hp_pos = egui::Pos2::new(px, py);

                    // Concentric thermal bloom
                    painter.circle_filled(hp_pos, base_r * 1.6, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 35));
                    painter.circle_filled(hp_pos, base_r * 1.0, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 75));
                    painter.circle_filled(hp_pos, base_r * 0.5, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180));
                }
            }

"""

target = "            // 4. Draw Authoritative Geodesic Loci & VR Broadcast Pins"
if "CSS (TIANHE) 395 km - SGP4 orbit" not in code:
    code = code.replace(target, satellite_and_heatmaps_code + target)

with open(r'C:\Users\blue-\projects\lite\crates\hblite_desktop\src\gui.rs', 'w', encoding='utf-8') as f:
    f.write(code)

print('Updated gui.rs with SGP4 satellite tracks and Snap Map heatmaps.')
