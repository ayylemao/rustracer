use bytemuck::{Pod, Zeroable};

use crate::{ray::Ray, shapes::Shape, Sphere};


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuRay {
    pub origin: [f32; 4],
    pub direction: [f32; 4]
}

impl GpuRay {
    pub fn from_ray(ray: &Ray) -> GpuRay {
        let mut origin = [0.0; 4];
        let mut direction = [0.0; 4];
        origin[0] = ray.origin.x;
        origin[1] = ray.origin.y;
        origin[2] = ray.origin.z;
        origin[3] = ray.origin.w;
        direction[0] = ray.direction.x;
        direction[1] = ray.direction.y;
        direction[2] = ray.direction.z;
        direction[3] = ray.direction.w;
        GpuRay { origin, direction}
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuShape {
    pub id: u32,
    pub kind: u32,
    pub _padding: [u32; 2], 
    inverse: [f32; 16],
    add_data: [f32; 24],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuIntersection {
    shape_id: u32,
    t: f32,
}

impl GpuShape {
    pub fn from_shape(shape: &dyn Shape) -> GpuShape {
        if let Some(sphere) = shape.as_any().downcast_ref::<Sphere>() {
            let inverse = sphere.inverse();

            let mut inverse_data: [f32; 16] = [0.0; 16];
            let add_data: [f32; 24] = [0.0; 24];

            for i in 0..4 {
                for j in 0..4 {
                    inverse_data[i * 4 + j] = inverse[(i, j)];
                }
            }
            
            return GpuShape {
                id: sphere.id as u32,
                kind: 1,
                _padding: [0; 2],
                inverse: inverse_data,
                add_data: add_data,
            }
        } else {
            panic!("Shape not yet implemented!")
        };
    }
}


#[cfg(test)]
pub mod tests {
    use std::{f32::consts::PI, sync::Arc};

    use wgpu::util::DeviceExt;
    use wgpu::PipelineCompilationOptions;

    use crate::ray::Ray;
    use crate::{matrix::Matrix, world::World, Sphere};
    use crate::gpu::GPUAccel;

    use super::{GpuIntersection, GpuRay, GpuShape};



    #[test]
    fn init_sphere() {
        let s1 = Sphere::with_transformation(Matrix::rotation_x(PI/2.0));
        let mut world = World::default();

        world.add_shape(Arc::new(s1));

        let s1 = world.shapes[2].as_ref();

        let gpu_sphere = GpuShape::from_shape(s1);
        println!("{:?}", gpu_sphere);
    }

    #[test]
    pub fn sphere_intersect() {
        let (device, queue) = GPUAccel::init();


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Intersect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
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

        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let trans = Matrix::scaling(2.0, 2.0, 2.0);
        let s1 = Sphere::with_transformation(trans);
        
        let gpuray = GpuRay::from_ray(&ray);
        let gpusphere = GpuShape::from_shape(Arc::new(s1).as_ref());

        let rays = vec![gpuray];
        let shapes = vec![gpusphere];

        let input_ray_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Input Rays"),
                contents: bytemuck::cast_slice(&rays),
                usage: wgpu::BufferUsages::STORAGE
            }
        );

        let input_shape_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Input Shapes"),
                contents: bytemuck::cast_slice(&shapes),
                usage: wgpu::BufferUsages::STORAGE
            }
        );

        let intersection_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Intersection Buffer"),
                size: (rays.len() * 2 * std::mem::size_of::<GpuIntersection>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false
            }
        );
        
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Bind group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input_ray_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: input_shape_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: intersection_buffer.as_entire_binding() 
                    }
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[]
            }
        );

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("intersect"),
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

        let num_elements = rays.len();
        let workgroup_size = 64;
        let workgroup_count = (num_elements + workgroup_size - 1) / workgroup_size;

        compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);

        drop(compute_pass);

        let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Buffer"),
            size: intersection_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(
            &intersection_buffer,
            0,
            &download_buffer,
            0,
            intersection_buffer.size(),
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

        let result: &[GpuIntersection] = bytemuck::cast_slice(&data);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].t, 3.0);
        assert_eq!(result[1].t, 7.0);


    }
}