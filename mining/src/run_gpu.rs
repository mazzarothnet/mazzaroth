use std::{borrow::Cow, num::NonZeroU64};
use utils::sha256::sha256_hash;
use wgpu::util::DeviceExt;
use crate::sha256_mining::{work_hash_to_package, work_hash_to_package_to_u8_vec};


pub fn get_test_shader() -> String {
    let sha256_wgsl = include_str!("shader/sha256.wgsl");
    let shader = include_str!("shader/test_sha256.wgsl");
    format!("{}\n{}", sha256_wgsl, shader)
}

pub fn get_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Failed to create adapter");

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("Adapter does not support compute shaders");
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
    .expect("Failed to create device");

    (device, queue)
}

pub fn get_test_sha256_cpu(
    block_hash: [u8; 32], work_id: u64
) -> [u8; 32] {
    let input_data = work_hash_to_package_to_u8_vec(block_hash, work_id);
    sha256_hash(&input_data)
}

fn bytes_to_hex_u8(bytes: &[u32; 8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..8 {
        result[i * 4] = (bytes[i] >> 24) as u8;
        result[i * 4 + 1] = (bytes[i] >> 16) as u8;
        result[i * 4 + 2] = (bytes[i] >> 8) as u8;
        result[i * 4 + 3] = bytes[i] as u8;
    }
    result
}

pub fn get_test_sha256_gpu(
    block_hash: [u8; 32], work_id: u64
) -> [u8; 32] {
    let input_data = work_hash_to_package(block_hash, work_id);
    let (device, queue) = get_device();
    let real_sha256_wgsl = get_test_shader();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(real_sha256_wgsl)),
    });

    let input_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&input_data),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // Now we create a buffer to store the output data.
    let output_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (input_data.len() / 16 * 8 * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: output_data_buffer.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    has_dynamic_offset: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    has_dynamic_offset: false,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_data_buffer.as_entire_binding(),
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
        module: &module,
        entry_point: Some("sha256"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
    
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
    
            // 每个工作组处理一个512位块(16个u32)
            // 我们的工作组大小是64，但每个线程处理一个块
            let block_count = input_data.len() / 16;
            compute_pass.dispatch_workgroups(block_count as u32, 1, 1);
        }

    encoder.copy_buffer_to_buffer(
        &output_data_buffer,
        0,
        &download_buffer,
        0,
        output_data_buffer.size(),
    );

    let command_buffer = encoder.finish();

    queue.submit([command_buffer]);

    let buffer_slice = download_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {
    });

    device.poll(wgpu::PollType::Wait).unwrap();

    let data = buffer_slice.get_mapped_range();
    let result: &[u32] = bytemuck::cast_slice(&data);
    let u32_8: [u32; 8] = result.try_into().unwrap();
    bytes_to_hex_u8(&u32_8)
}