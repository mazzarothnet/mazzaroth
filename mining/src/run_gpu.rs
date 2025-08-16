use crate::sha256_mining::{
    gen_sha256_by_block_hash_and_nonce, vec_to_nonce, work_hash_to_package,
};
use anyhow::Context;
use consensus::types::BlockKey;
use crypto_bigint::U256;
use std::{borrow::Cow, num::NonZeroU64};
use wgpu::util::DeviceExt;

pub fn mining_gpu_sha256(
    context: &Sha256Context,
    block_hash: [u8; 32],
    work_id: u64,
    target: BlockKey,
) -> anyhow::Result<Option<u128>> {
    let target_u32 = hex_u8_to_u32(&target.0.to_be_bytes());
    let input_data = work_hash_to_package(block_hash, work_id);
    let mut real_input: [u32; 24] = [0; 24];
    for i in 0..16 {
        real_input[i] = input_data[i];
    }
    for i in 0..8 {
        real_input[i + 16] = target_u32[i];
    }

    let block_counter = 64;
    let result = submit_work_to_gpu(
        &context,
        &real_input,
        4 * block_counter,
        block_counter as u32,
    )?;
    for i in 0..block_counter {
        let nonce = result[i * 4..i * 4 + 4]
            .try_into()
            .map_err(|e| anyhow::anyhow!("Failed to convert result to u32: {}", e))?;
        let nonce_u128 = vec_to_nonce(nonce);
        let mined_hash = gen_sha256_by_block_hash_and_nonce(block_hash, nonce_u128);
        let mined_block_key = BlockKey(U256::from_be_slice(&mined_hash));
        if mined_block_key < target {
            return Ok(Some(nonce_u128));
        }
    }
    Ok(None)
}

fn hex_u8_to_u32(bytes: &[u8; 32]) -> [u32; 8] {
    let mut result = [0u32; 8];
    for i in 0..8 {
        result[i] = u32::from_be_bytes([
            bytes[i * 4],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ]);
    }
    result
}

pub struct Sha256Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl Sha256Context {
    pub fn new() -> anyhow::Result<Self> {
        let (device, queue) = get_device()?;
        let real_sha256_wgsl = get_mining_shader();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(real_sha256_wgsl)),
        });

        let min_binding_size = NonZeroU64::new(4)
            .ok_or_else(|| anyhow::anyhow!("Failed to create min binding size"))?;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        min_binding_size: Some(min_binding_size),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        min_binding_size: Some(min_binding_size),
                        has_dynamic_offset: false,
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
            module: &module,
            entry_point: Some("mining"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            bind_group_layout,
            pipeline,
        })
    }
}

fn submit_work_to_gpu(
    context: &Sha256Context,
    input_data: &[u32],
    output_len: usize,
    block_counter: u32,
) -> anyhow::Result<Vec<u32>> {
    let input_data_buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&input_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

    // Now we create a buffer to store the output data.
    let output_data_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (output_len * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let download_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: output_data_buffer.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &context.bind_group_layout,
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

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&context.pipeline);
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

    context.queue.submit([command_buffer]);

    let buffer_slice = download_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    context
        .device
        .poll(wgpu::PollType::Wait)
        .with_context(|| "Failed to poll")?;

    let data = buffer_slice.get_mapped_range();
    let result: &[u32] = bytemuck::cast_slice(&data);

    Ok(result.to_vec())
}

fn get_mining_shader() -> String {
    let sha256_wgsl = include_str!("shader/sha256.wgsl");
    let shader = include_str!("shader/mining.wgsl");
    format!("{}\n{}", sha256_wgsl, shader)
}

fn get_device() -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .with_context(|| "Failed to create adapter")?;

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        anyhow::bail!("Adapter does not support compute shaders");
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
    .with_context(|| "Failed to create device")?;

    Ok((device, queue))
}
