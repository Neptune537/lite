import 'dart:math' as math;
import 'package:flutter_test/flutter_test.dart';
import 'package:fluoderpod/fluoderpod.dart';

void main() {
  group('Cyan Globe 3D Spherical Projection Tests', () {
    test('projectToVector produces unit vector on sphere surface', () {
      final v = projectToVector(0.0, 0.0, 0.0, 0.0);
      final len = math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
      expect(len, closeTo(1.0, 1e-4));
      // At (0, 0) facing forward, z should be 1.0 (towards camera)
      expect(v.z, closeTo(1.0, 1e-4));
      expect(v.x, closeTo(0.0, 1e-4));
      expect(v.y, closeTo(0.0, 1e-4));
    });

    test('project3dSphere front-facing point projects inside viewport', () {
      const center = Offset(100.0, 100.0);
      const radius = 80.0;
      final pt = project3dSphere(0.0, 0.0, center, radius, 0.0, 0.0);
      expect(pt, isNotNull);
      expect(pt!.dx, closeTo(100.0, 1.0));
      expect(pt.dy, closeTo(100.0, 1.0));
    });

    test('project3dSphere back-facing point is correctly culled (null)', () {
      const center = Offset(100.0, 100.0);
      const radius = 80.0;
      // 180 degrees longitude is on the back side of the sphere
      final pt = project3dSphere(0.0, 180.0, center, radius, 0.0, 0.0);
      expect(pt, isNull);
    });

    test('SphereVector3 normalization handles edge points', () {
      const vec = SphereVector3(3.0, 4.0, 0.0);
      final norm = vec.normalized();
      expect(norm.x, closeTo(0.6, 1e-4));
      expect(norm.y, closeTo(0.8, 1e-4));
      expect(norm.z, closeTo(0.0, 1e-4));
    });
  });
}
