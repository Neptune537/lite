use eframe::{egui_wgpu, wgpu};
use glam::DVec3;
use nautilus_world::{Wgs84GeoCoord, WGS84_FLATTENING_F};
use wgpu::util::{DeviceExt, TextureDataOrder};
use std::sync::OnceLock;

const CAMERA_DISTANCE: f64 = 2.624;
const POLAR_RATIO: f64 = 1.0 - WGS84_FLATTENING_F;

struct GlobeRenderSpec { sun: [f32; 3], cloud_drift: f32 }

fn globe_render_spec() -> &'static GlobeRenderSpec {
    static SPEC: OnceLock<GlobeRenderSpec> = OnceLock::new();
    SPEC.get_or_init(|| {
        let value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../heavenlyBound/dist/assets/globe-render-v1.json"
        )).expect("Nautilus globe contract JSON");
        assert_eq!(value["version"], 1);
        assert_eq!(value["model"], "WGS84");
        let sun = std::array::from_fn(|index| value["sunDirection"][index].as_f64()
            .expect("globe sun component") as f32);
        let cloud_drift = value["cloudDriftTurnsPerSecond"].as_f64()
            .expect("globe cloud drift") as f32;
        GlobeRenderSpec { sun, cloud_drift }
    })
}

pub struct EarthView {
    eye: DVec3,
    right: DVec3,
    up: DVec3,
    focal: f64,
}

impl EarthView {
    pub fn new(rotation: f32, tilt: f32) -> Self {
        let (sin_lon, cos_lon) = (-(rotation as f64)).sin_cos();
        let (sin_lat, cos_lat) = (tilt as f64).sin_cos();
        Self {
            eye: CAMERA_DISTANCE * DVec3::new(cos_lat * cos_lon, sin_lat, -cos_lat * sin_lon),
            right: DVec3::new(-sin_lon, 0.0, -cos_lon),
            up: DVec3::new(-sin_lat * cos_lon, cos_lat, sin_lat * sin_lon),
            focal: (CAMERA_DISTANCE * CAMERA_DISTANCE - 1.0).sqrt(),
        }
    }

    pub fn project(&self, geo: Wgs84GeoCoord) -> Option<egui::Vec2> {
        let position = nautilus_world::globe::globe_position(geo, 1.0);
        let from_eye = position - self.eye;
        let depth = from_eye.dot(-self.eye / CAMERA_DISTANCE);
        if !depth.is_finite() || depth <= 0.0 {
            return None;
        }
        let ellipsoid = DVec3::new(1.0, 1.0 / POLAR_RATIO, 1.0);
        let origin = self.eye * ellipsoid;
        let ray = from_eye * ellipsoid;
        let a = ray.length_squared();
        let b = origin.dot(ray);
        let discriminant = b * b - a * (origin.length_squared() - 1.0);
        if discriminant >= 0.0 && (-b - discriminant.sqrt()) / a < 1.0 - 1e-8 {
            return None;
        }
        Some(egui::vec2(
            (position.dot(self.right) * self.focal / depth) as f32,
            (-position.dot(self.up) * self.focal / depth) as f32,
        ))
    }

    pub fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        radius: f32,
        target: wgpu::TextureFormat,
        time: f64,
    ) {
        let mut values = [0.0f32; 20];
        let spec = globe_render_spec();
        for (offset, axis) in [(0, self.eye), (4, self.right), (8, self.up)] {
            values[offset..offset + 3].copy_from_slice(&axis.as_vec3().to_array());
        }
        values[3] = self.focal as f32;
        values[7] = POLAR_RATIO as f32;
        // HeavenlyBound passes milliseconds to its orbital cloud shader.
        values[11] = (time * f64::from(spec.cloud_drift)).fract() as f32;
        values[12] = rect.width() / (2.0 * radius);
        values[13] = rect.height() / (2.0 * radius);
        values[14] = if target.is_srgb() { 1.0 } else { 0.0 };
        values[16..19].copy_from_slice(&spec.sun);
        painter.add(egui_wgpu::Callback::new_paint_callback(
            rect,
            EarthDraw { target, values },
        ));
    }
}

struct EarthDraw {
    target: wgpu::TextureFormat,
    values: [f32; 20],
}

struct EarthGpu {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
}

impl EarthGpu {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, target: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Nautilus Earth"),
            source: wgpu::ShaderSource::Wgsl(include_str!("earth.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Nautilus Earth"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(target.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        const WIDTH: u32 = 2048;
        const HEIGHT: u32 = 1024;
        let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4 * 4) as usize);
        for bytes in [
            include_bytes!("../../../../heavenlyBound/dist/assets/earth_atmos_2048.jpg").as_slice(),
            include_bytes!("../../../../heavenlyBound/dist/assets/earth_specular_2048.jpg")
                .as_slice(),
            include_bytes!("../../../../heavenlyBound/dist/assets/earth_lights_2048.png")
                .as_slice(),
            include_bytes!("../../../../heavenlyBound/dist/assets/earth_clouds_1024.png")
                .as_slice(),
        ] {
            let image =
                image::load_from_memory(bytes).expect("embedded HeavenlyBound Earth texture");
            pixels.extend(
                image
                    .resize_exact(WIDTH, HEIGHT, image::imageops::FilterType::Triangle)
                    .into_rgba8()
                    .into_raw(),
            );
        }
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("HeavenlyBound Earth maps"),
                size: wgpu::Extent3d {
                    width: WIDTH,
                    height: HEIGHT,
                    depth_or_array_layers: 4,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            &pixels,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Earth camera"),
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            pipeline,
            bind_group,
            uniform,
        }
    }
}

impl egui_wgpu::CallbackTrait for EarthDraw {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        _: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if resources.get::<EarthGpu>().is_none() {
            resources.insert(EarthGpu::new(device, queue, self.target));
        }
        let gpu = resources
            .get::<EarthGpu>()
            .expect("Earth renderer initialized");
        let mut bytes = [0; 80];
        for (chunk, value) in bytes.chunks_exact_mut(4).zip(self.values) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        queue.write_buffer(&gpu.uniform, 0, &bytes);
        Vec::new()
    }

    fn paint(
        &self,
        _: egui::epaint::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        if let Some(gpu) = resources.get::<EarthGpu>() {
            pass.set_pipeline(&gpu.pipeline);
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
