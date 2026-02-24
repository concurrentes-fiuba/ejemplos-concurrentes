use async_std::task;
use wgpu::BufferAddress;

fn main() {
    task::block_on(run());
}

async fn run() {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();

    let shader = device.create_shader_module(wgpu::include_wgsl!("threadindexing.wgsl"));

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Thread Indexing example"),
        layout: None,
        module: &shader,
        entry_point: None,
        compilation_options: Default::default(),
        cache: Default::default(),
    });

    let data_size = 1024;
    let workgroups_x = 8; // 32
    let workgroups_y = 2;
    let workgroups_z = 2;
    let threads = workgroups_x * workgroups_y * workgroups_z * 32;
    assert_eq!(data_size, threads);
    let size = (data_size * 4 * size_of::<u32>() * 3) as BufferAddress;

    let result = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("result"),
        size,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read_buffer"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: result.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&Default::default());

    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups_x as u32, workgroups_y as u32, workgroups_z as u32);
    }

    encoder.copy_buffer_to_buffer(&result, 0, &read_buffer, 0, size);

    queue.submit([encoder.finish()]);

    read_buffer.map_async(wgpu::MapMode::Read, .., move |result| {
    });

    device.poll(wgpu::PollType::Wait).unwrap();

    let result_view = read_buffer.get_mapped_range(..);
    let result_view_u32: &[u32] = bytemuck::cast_slice(&result_view);
    for id in 0..threads {
        let buffer_pos = id * 4 * 3;
        println!("Data: {}, Grid X: {}, Y: {}, Z: {}; Workgroup X: {}, Y: {}, Z: {}; Thread X:{} Y:{} Z:{}",
                id,
                 result_view_u32[buffer_pos],
                 result_view_u32[buffer_pos+1],
                 result_view_u32[buffer_pos+2],
                 result_view_u32[buffer_pos+4],
                 result_view_u32[buffer_pos+5],
                 result_view_u32[buffer_pos+6],
                 result_view_u32[buffer_pos+8],
                 result_view_u32[buffer_pos+9],
                 result_view_u32[buffer_pos+10],
        );
    }
}