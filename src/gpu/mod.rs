use std::collections::HashMap;
use std::sync::Arc;

use gpu_types::{GpuIntersection, GpuRay, GpuShape};
use wgpu::BindGroup;
use wgpu::util::DeviceExt;
use wgpu::{BindGroupLayout, Buffer, ComputePipeline, Device, Queue};

use crate::shapes::Shape;
pub mod gpu_types;

const MAX_INTERSECTIONS_PER_RAY: u32 = 8;

pub struct GPUAccel<'a> {
    device: Device,
    queue: Queue,
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    ray_buffer: Option<Buffer>,
    shape_buffer: Option<Buffer>,
    intersection_buffer: Option<Buffer>,
    shape_mapping: HashMap<u32, &'a dyn Shape>,
    gpu_shapes: Vec<GpuShape>,
}

impl<'a> GPUAccel<'a> {
    pub fn new(shader_path: &str) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::default(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Intersect Shader"),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(shader_path).unwrap().into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("intersection_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("intersect"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            bind_group: None,
            ray_buffer: None,
            shape_buffer: None,
            intersection_buffer: None,
            shape_mapping: HashMap::new(),
            gpu_shapes: Vec::new(),
        }
    }

    pub fn upload_rays_and_shapes(&mut self, rays: &[GpuRay]) {
        let ray_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input Rays"),
                contents: bytemuck::cast_slice(rays),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let shape_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input Shapes"),
                contents: bytemuck::cast_slice(self.gpu_shapes.as_slice()),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let intersection_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Intersection Buffer"),
            size: (rays.len() * 8 * std::mem::size_of::<GpuIntersection>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ray_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: shape_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: intersection_buffer.as_entire_binding(),
                },
            ],
        });

        self.ray_buffer = Some(ray_buffer);
        self.shape_buffer = Some(shape_buffer);
        self.intersection_buffer = Some(intersection_buffer);
        self.bind_group = Some(bind_group);
    }

    pub fn dispatch(&self) {
        let ray_count =
            self.ray_buffer.as_ref().unwrap().size() / std::mem::size_of::<GpuRay>() as u64;
        let num_rays = ray_count as usize;
        let workgroup_size = 64;
        let workgroup_count = (num_rays + workgroup_size - 1) / workgroup_size;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn download_intersections(&self) -> Vec<GpuIntersection> {
        let intersection_buffer = self.intersection_buffer.as_ref().unwrap();

        let download_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Buffer"),
            size: intersection_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            intersection_buffer,
            0,
            &download_buffer,
            0,
            intersection_buffer.size(),
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = download_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::Wait).unwrap();

        let data = slice.get_mapped_range();
        let result: &[GpuIntersection] = bytemuck::cast_slice(&data);
        result.to_vec()
    }

    pub fn get_hits_for_ray(
        results: &Vec<GpuIntersection>,
        ray_idx: usize,
    ) -> Vec<GpuIntersection> {
        let start = ray_idx * MAX_INTERSECTIONS_PER_RAY as usize;
        let end = start + MAX_INTERSECTIONS_PER_RAY as usize;
        results[start..end]
            .iter()
            .filter(|hit| hit.shape_id != 0)
            .cloned()
            .collect()
    }

    pub fn populate_shapes(&mut self, shapes: &'a Vec<Arc<dyn Shape + Send + Sync>>) {
        for shape in shapes {
            let shape_ref = shape.as_ref();
            self.gpu_shapes.push(GpuShape::from_shape(shape_ref));
            self.shape_mapping.insert(shape_ref.id() as u32, shape_ref);
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::world::World;

    use super::GPUAccel;

    #[test]
    fn test_shape_mapping() {
        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");
        let w = World::default();

        gpu_accel.populate_shapes(&w.shapes);
        println!("{:?}", gpu_accel.gpu_shapes)
    }
}
