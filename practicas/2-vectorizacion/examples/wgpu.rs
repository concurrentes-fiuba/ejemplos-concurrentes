use async_std::task;
use image::{EncodableLayout, ImageBuffer, RgbaImage};
use std::time::Instant;
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

    let shader = device.create_shader_module(wgpu::include_wgsl!("wgpu.wgsl"));

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Image example"),
        layout: None,
        module: &shader,
        entry_point: None,
        compilation_options: Default::default(),
        cache: Default::default(),
    });

    let input_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/totk.jpg");
    let input_image = image::open(input_path).unwrap().to_rgba8();

    let (width, height) = input_image.dimensions();

    let size = (width * height * 4) as BufferAddress;

    let result = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("result"),
        size,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
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
        pass.dispatch_workgroups(width / 32, height, 1);
    }

    let start = Instant::now();
    queue.write_buffer(&result, 0, input_image.as_raw());
    queue.submit([]);
    device.poll(wgpu::PollType::Wait).unwrap();
    println!("{:?}", start.elapsed());

    let start = Instant::now();
    queue.submit([encoder.finish()]);
    device.poll(wgpu::PollType::Wait).unwrap();

    println!("{:?}", start.elapsed());


    let start = Instant::now();
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&result, 0, &read_buffer, 0, size);
    queue.submit([encoder.finish()]);
    read_buffer.map_async(wgpu::MapMode::Read, .., move |result| {
    });
    device.poll(wgpu::PollType::Wait).unwrap();
    println!("{:?}", start.elapsed());

    let result_view = read_buffer.get_mapped_range(..);
    let mut output_image:RgbaImage = ImageBuffer::new(width, height);
    output_image.copy_from_slice(result_view.as_bytes());

    output_image.save(concat!(env!("CARGO_MANIFEST_DIR"), "/target/output.jpg")).unwrap();

}