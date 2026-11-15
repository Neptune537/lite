struct View {
    eye_focal: vec4f,
    right_polar: vec4f,
    up_cloud_offset: vec4f,
    viewport: vec4f,
    sun: vec4f,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var earth_maps: texture_2d_array<f32>;
@group(0) @binding(2) var map_sampler: sampler;

struct Vertex {
    @builtin(position) position: vec4f,
    @location(0) screen: vec2f,
}

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> Vertex {
    let screen = array(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0))[index];
    return Vertex(vec4f(screen, 0.0, 1.0), screen);
}

fn output_color(color: vec3f) -> vec4f {
    let rgb = clamp(color, vec3f(0.0), vec3f(1.0));
    if view.viewport.z > 0.5 {
        let linear = select(rgb / 12.92, pow((rgb + 0.055) / 1.055, vec3f(2.4)), rgb > vec3f(0.04045));
        return vec4f(linear, 1.0);
    }
    return vec4f(rgb, 1.0);
}

@fragment
fn fragment_main(input: Vertex) -> @location(0) vec4f {
    let screen = input.screen * view.viewport.xy;
    let eye = view.eye_focal.xyz;
    let ray = -normalize(eye) + (view.right_polar.xyz * screen.x +
        view.up_cloud_offset.xyz * screen.y) / view.eye_focal.w;
    let inverse_polar = 1.0 / view.right_polar.w;
    let ellipsoid = vec3f(1.0, inverse_polar, 1.0);
    let q = eye * ellipsoid;
    let d = ray * ellipsoid;
    let a = dot(d, d);
    let b = dot(q, d);
    let discriminant = b * b - a * (dot(q, q) - 1.0);
    if discriminant < 0.0 {
        let cells = screen * 95.0;
        let seed = fract(sin(dot(floor(cells), vec2f(127.1, 311.7))) * 43758.5453);
        let point = 1.0 - smoothstep(0.02, 0.18, length(fract(cells) - 0.5));
        let star = select(0.0, point * (seed - 0.97) * 13.0, seed > 0.97);
        return output_color(vec3f(0.01, 0.014, 0.023) + vec3f(0.73, 0.86, 1.0) * star);
    }

    let position = eye + ray * ((-b - sqrt(discriminant)) / a);
    let normal = normalize(position * ellipsoid * ellipsoid);
    let longitude = atan2(-normal.z, normal.x);
    let latitude = asin(clamp(normal.y, -1.0, 1.0));
    let uv = vec2f(0.5 + longitude / 6.2831853, 0.5 - latitude / 3.14159265);
    let day_map = textureSampleLevel(earth_maps, map_sampler, uv, 0, 0.0).rgb;
    let spec_map = textureSampleLevel(earth_maps, map_sampler, uv, 1, 0.0).r;
    let city_map = textureSampleLevel(earth_maps, map_sampler, uv, 2, 0.0).rgb;
    let cloud_uv = uv + vec2f(view.up_cloud_offset.w, 0.0);
    let cloud_a = textureSampleLevel(earth_maps, map_sampler, cloud_uv, 3, 0.0).r;
    let cloud_b = textureSampleLevel(earth_maps, map_sampler,
        uv + vec2f(view.up_cloud_offset.w * 1.5 + 0.12, 0.0), 3, 0.0).r;
    let cloud = clamp(cloud_a * 1.05 + cloud_b * 0.25, 0.0, 1.0);

    // Orbital lighting matches HeavenlyBound's Earth shader. The terrain handoff is game-only.
    let sun = normalize(view.sun.xyz);
    let toward_eye = normalize(-ray);
    let sunlight = dot(normal, sun);
    let day_factor = smoothstep(-0.20, 0.18, sunlight);
    let specular = pow(max(dot(normal, normalize(sun + toward_eye)), 0.0), 64.0) * spec_map * (1.0 - cloud * 0.9);
    let surface = mix(day_map, vec3f(0.96, 0.98, 1.0), cloud * 0.78);
    let sun_color = vec3f(1.18, 1.12, 1.02);
    let ambient = vec3f(0.015, 0.025, 0.045);
    let day = surface * (ambient + sun_color * max(sunlight, 0.0)) +
        sun_color * specular * 0.95;
    let city_lights = city_map * vec3f(1.10, 0.92, 0.58) * 1.65 * (1.0 - cloud * 0.45);
    let night = surface * ambient + city_lights;
    let limb = pow(1.0 - clamp(dot(normal, toward_eye), 0.0, 1.0), 3.8);
    let rim = vec3f(0.15, 0.52, 1.0) * limb * 1.45 *
        (max(sunlight * 0.65 + 0.35, 0.0) * day_factor + 0.22);
    return output_color(mix(night, day, day_factor) + rim);
}
