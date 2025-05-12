use std::{num::NonZeroU64, str::FromStr};
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue};
use wgpu::{BindGroup, PipelineCache, PipelineCompilationOptions};

pub mod gpu_types;

pub struct GPUAccel {}

impl GPUAccel {
    pub fn init() -> (Device, Queue) {
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
        return (device, queue)
    }

    
}

#[cfg(test)]
pub mod test {
    use wgpu::{BindGroup, PipelineCache, PipelineCompilationOptions, util::DeviceExt};

    use crate::gpu::GPUAccel;


    pub fn test_wgpu_setup() {
        let (device, queue) = GPUAccel::init();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Test Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("test_bindgroup_layout"),
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
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let input_data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input buffer"),
            contents: bytemuck::cast_slice(&input_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output buffer"),
            size: (input_data.len() * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("doubleMe"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let num_elements = input_data.len();
        let workgroup_size = 64;
        let workgroup_count = (num_elements + workgroup_size - 1) / workgroup_size;

        compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);

        drop(compute_pass);

        let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: input_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &download_buffer,
            0,
            output_buffer.size(),
        );

        let command_buffer = encoder.finish();
        queue.submit([command_buffer]);
        
        let buffer_slice = download_buffer.slice(..);

        buffer_slice.map_async(wgpu::MapMode::Read, |_| {
            // In this case we know exactly when the mapping will be finished,
            // so we don't need to do anything in the callback.
        });

        device.poll(wgpu::PollType::Wait).unwrap();

        let data = buffer_slice.get_mapped_range();

        let result: &[f32] = bytemuck::cast_slice(&data);

        println!("{:?}", result);

    }
}
