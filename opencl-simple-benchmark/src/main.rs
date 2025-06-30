use ocl::{Platform, Device, Context, Queue, Program, Buffer, Kernel, Event, EventList, DeviceType};
use ocl::enums::{ProfilingInfo, DeviceInfo};
use ocl::core::{ProfilingInfoResult, DeviceInfoResult};
use ocl::error::Error as OclError;
use std::time::Instant;

const OPENCL_KERNEL: &str = r#"
    __kernel void vecadd(
        __global int *A,
        __global int *B,
        __global int *C,
        const int N)
    {
        int id = get_global_id(0);
        if (id < N) {
            C[id] = A[id] + B[id];
        }
    }
"#;

fn run_benchmark(platform: Platform, device: Device) -> Result<(), OclError> {
    let device_name = device.name()?;
    let platform_name = platform.name()?;
    println!("--- Benchmarking Device: {} (Platform: {}) ---", device_name, platform_name);

    let context = Context::builder().platform(platform).devices(device).build()?;
    let queue = Queue::new(&context, device, Some(ocl::flags::QUEUE_PROFILING_ENABLE))?;
    let program = Program::builder()
        .source(OPENCL_KERNEL)
        .devices(device)
        .build(&context)?;

    const DATA_SIZE: usize = 1024 * 1024;
    let h_a = vec![1i32; DATA_SIZE];
    let h_b = vec![2i32; DATA_SIZE];
    let mut h_c = vec![0i32; DATA_SIZE];

    let d_a: Buffer<i32> = Buffer::builder()
        .queue(queue.clone())
        .flags(ocl::flags::MEM_READ_ONLY | ocl::flags::MEM_HOST_WRITE_ONLY)
        .len(DATA_SIZE)
        .build()?;

    let d_b: Buffer<i32> = Buffer::builder()
        .queue(queue.clone())
        .flags(ocl::flags::MEM_READ_ONLY | ocl::flags::MEM_HOST_WRITE_ONLY)
        .len(DATA_SIZE)
        .build()?;

    let d_c: Buffer<i32> = Buffer::builder()
        .queue(queue.clone())
        .flags(ocl::flags::MEM_WRITE_ONLY | ocl::flags::MEM_HOST_READ_ONLY)
        .len(DATA_SIZE)
        .build()?;

    let kernel = Kernel::builder()
        .program(&program)
        .name("vecadd")
        .queue(queue.clone())
        .global_work_size(DATA_SIZE)
        .arg(&d_a)
        .arg(&d_b)
        .arg(&d_c)
        .arg(&(DATA_SIZE as i32))
        .build()?;

    let start_overall = Instant::now();

    let mut write_event_a = Event::empty();
    d_a.cmd().write(&h_a).enew(&mut write_event_a).enq()?;

    let mut write_event_b = Event::empty();
    d_b.cmd().write(&h_b).enew(&mut write_event_b).enq()?;

    let mut kernel_event = Event::empty();
    let mut write_events = EventList::new();
    write_events.push(write_event_a.clone());
    write_events.push(write_event_b.clone());

    unsafe {
        kernel.cmd().ewait(&write_events).enew(&mut kernel_event).enq()?;
    }

    let mut read_event_c = Event::empty();
    d_c.cmd().read(&mut h_c).ewait(&kernel_event).enew(&mut read_event_c).enq()?;

    queue.finish()?;

    let overall_ms = start_overall.elapsed().as_secs_f64() * 1000.0;

    let write_a_ms = get_event_duration_ms(&write_event_a)?;
    let write_b_ms = get_event_duration_ms(&write_event_b)?;
    let kernel_ms = get_event_duration_ms(&kernel_event)?;
    let read_c_ms = get_event_duration_ms(&read_event_c)?;

    println!("\n--- Benchmark Results ({} elements) ---", DATA_SIZE);
    println!("Data Size: {:.2} MB", (DATA_SIZE * std::mem::size_of::<i32>()) as f64 / (1024.0 * 1024.0));
    println!("Write A (Host -> Device): {:.6} ms", write_a_ms);
    println!("Write B (Host -> Device): {:.6} ms", write_b_ms);
    println!("Kernel Execution Time:    {:.6} ms", kernel_ms);
    println!("Read C (Device -> Host):  {:.6} ms", read_c_ms);
    println!("Total Overall Time (measured by host clock): {:.6} ms", overall_ms);

    let mut correct = true;
    for i in 0..10.min(DATA_SIZE) {
        if h_c[i] != h_a[i] + h_b[i] {
            correct = false;
            break;
        }
    }
    if correct {
        println!("Result verification: PASSED (first 10 elements are correct)");
    } else {
        println!("Result verification: FAILED");
    }
    println!();

    Ok(())
}

fn get_event_duration_ms(event: &Event) -> Result<f64, OclError> {
    let time_start = event.profiling_info(ProfilingInfo::Start)?;
    let time_end = event.profiling_info(ProfilingInfo::End)?;

    if let (ProfilingInfoResult::Start(start), ProfilingInfoResult::End(end)) = (time_start, time_end) {
        Ok((end - start) as f64 * 1e-6)
    } else {
        Ok(0.0)
    }
}

fn main() -> Result<(), OclError> {
    let platforms = Platform::list();
    if platforms.is_empty() {
        eprintln!("No OpenCL platforms found!");
        return Ok(());
    }

    println!("--- Discovered OpenCL Platforms and Devices ---");

    for (platform_idx, platform) in platforms.iter().enumerate() {
        let platform_name = platform.name()?;
        let platform_version = platform.version()?;
        println!("Platform {}: {} (Version: {})", platform_idx, platform_name, platform_version);

        let devices = Device::list_all(platform)?;
        if devices.is_empty() {
            println!("  No devices found for this platform.");
        } else {
            for (device_idx, device) in devices.iter().enumerate() {
                let device_name = device.name()?;
                let device_type = match device.info(DeviceInfo::Type)? {
                    DeviceInfoResult::Type(t) => t,
                    _ => DeviceType::DEFAULT,
                };
                let type_str = match device_type {
                    DeviceType::CPU => "CPU",
                    DeviceType::GPU => "GPU",
                    DeviceType::ACCELERATOR => "Accelerator",
                    _ => "Unknown",
                };
                println!("  Device {}: {} (Type: {})", device_idx, device_name, type_str);
                if let Err(e) = run_benchmark(*platform, *device) {
                    eprintln!("Error running benchmark: {}", e);
                }
            }
        }
    }
    Ok(())
}