use crate::sha256_mining::{vec_to_nonce, work_hash_to_package, work_hash_to_package_to_u8_vec};
use std::{borrow::Cow, num::NonZeroU64};
use utils::sha256::sha256_hash;
use wgpu::util::DeviceExt;

pub fn get_test_sha256_cpu(block_hash: [u8; 32], work_id: u64) -> [u8; 32] {
    let input_data = work_hash_to_package_to_u8_vec(block_hash, work_id);
    sha256_hash(&input_data)
}

pub fn get_test_sha256_gpu(block_hash: [u8; 32], work_id: u64) -> [u8; 32] {
    let shader = get_test_shader();
    let input_data = work_hash_to_package(block_hash, work_id);
    let result = submit_work_to_gpu(&input_data, 8, shader, "sha256", 1);
    let u32_8: [u32; 8] = result.try_into().unwrap();
    bytes_to_hex_u8(&u32_8)
}

pub fn mining_gpu_sha256(block_hash: [u8; 32], work_id: u64, target: [u32; 8]) -> Vec<u128> {
    let shader = get_mining_shader();
    let input_data = work_hash_to_package(block_hash, work_id);
    let mut real_input: [u32; 24] = [0; 24];
    for i in 0..16 {
        real_input[i] = input_data[i];
    }
    for i in 0..8 {
        real_input[i + 16] = target[i];
    }

    let block_counter = 64;
    let result = submit_work_to_gpu(
        &real_input,
        4 * block_counter,
        shader,
        "mining",
        block_counter as u32,
    );
    let mut nonce_vec = Vec::new();
    for i in 0..block_counter {
        let nonce = result[i * 4..i * 4 + 4].try_into().unwrap();
        let nonce_u128 = vec_to_nonce(nonce);
        nonce_vec.push(nonce_u128);
    }
    nonce_vec
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

fn submit_work_to_gpu(
    input_data: &[u32],
    output_len: usize,
    shader: String,
    fn_name: &str,
    block_counter: u32,
) -> Vec<u32> {
    let (device, queue) = get_device();
    let real_sha256_wgsl = shader;
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
        size: (output_len * std::mem::size_of::<u32>()) as u64,
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
        entry_point: Some(fn_name),
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

        compute_pass.dispatch_workgroups(block_counter, 1, 1);
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
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    device.poll(wgpu::PollType::Wait).unwrap();

    let data = buffer_slice.get_mapped_range();
    let result: &[u32] = bytemuck::cast_slice(&data);

    result.to_vec()
}

fn get_test_shader() -> String {
    let sha256_wgsl = include_str!("shader/sha256.wgsl");
    let shader = include_str!("shader/test_sha256.wgsl");
    format!("{}\n{}", sha256_wgsl, shader)
}

fn get_mining_shader() -> String {
    let sha256_wgsl = include_str!("shader/sha256.wgsl");
    let shader = include_str!("shader/mining.wgsl");
    format!("{}\n{}", sha256_wgsl, shader)
}

fn get_device() -> (wgpu::Device, wgpu::Queue) {
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
