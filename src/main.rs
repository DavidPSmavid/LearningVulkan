use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::VulkanLibrary;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close button was pressed. Stopping.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        },
    )
    .expect("failed to create instance");
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Could not enumerate devices")
        .next()
        .expect("No devices available.");

    for family in physical_device.queue_family_properties() {
        println!(
            "Found a queue family with {:?} queue(s)",
            family.queue_count
        );
    }

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_queue_family_index, queue_family_properties)| {
            queue_family_properties
                .queue_flags
                .contains(QueueFlags::GRAPHICS)
        })
        .expect("Couldn't find a graphical queue family") as u32;
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("Failed to create device.");

    let _queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let data: i32 = 12;
    let _buffer = Buffer::from_data(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data,
    )
    .expect("Failed to create buffer.");

    let source_content: Vec<i32> = (0..64).collect();
    let source = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        source_content,
    )
    .expect("Failed to create source buffer.");
    let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
    let destination = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        destination_content,
    )
    .expect("Failed to create destination buffer.");

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    ));
    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue_family_index,
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    builder
        .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
        .unwrap();
    let command_buffer = builder.build().unwrap();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
