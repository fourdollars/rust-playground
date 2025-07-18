use cl_sys::*;
use libc::{c_void, size_t};
use std::ffi::CString;
use std::ptr;
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

fn main() {
    let platforms = get_platforms();
    if platforms.is_empty() {
        eprintln!("No OpenCL platforms found!");
        return;
    }

    println!("--- Discovered OpenCL Platforms and Devices ---");

    for (platform_idx, &platform) in platforms.iter().enumerate() {
        let platform_name = get_platform_info(platform, CL_PLATFORM_NAME);
        let platform_version = get_platform_info(platform, CL_PLATFORM_VERSION);
        println!("Platform {}: {} (Version: {})", platform_idx, platform_name, platform_version);

        let devices = get_devices(platform, CL_DEVICE_TYPE_ALL);
        if devices.is_empty() {
            println!("  No devices found for this platform.");
        } else {
            for (device_idx, &device) in devices.iter().enumerate() {
                let device_name = get_device_info(device, CL_DEVICE_NAME);
                let device_type: cl_device_type = get_device_info_raw(device, CL_DEVICE_TYPE);
                let type_str = match device_type {
                    CL_DEVICE_TYPE_CPU => "CPU",
                    CL_DEVICE_TYPE_GPU => "GPU",
                    CL_DEVICE_TYPE_ACCELERATOR => "Accelerator",
                    _ => "Unknown",
                };
                println!("  Device {}: {} (Type: {})", device_idx, device_name, type_str);
                run_benchmark(platform, device);
            }
        }
    }
}

fn get_platforms() -> Vec<cl_platform_id> {
    let mut num_platforms = 0;
    unsafe {
        clGetPlatformIDs(0, ptr::null_mut(), &mut num_platforms);
    }

    let mut platforms = Vec::with_capacity(num_platforms as usize);
    unsafe {
        clGetPlatformIDs(num_platforms, platforms.as_mut_ptr(), ptr::null_mut());
        platforms.set_len(num_platforms as usize);
    }
    platforms
}

fn get_platform_info(platform: cl_platform_id, param_name: cl_platform_info) -> String {
    let mut size = 0;
    unsafe {
        clGetPlatformInfo(platform, param_name, 0, ptr::null_mut(), &mut size);
    }

    if size == 0 {
        return String::new();
    }

    let mut result = vec![0u8; size];
    unsafe {
        clGetPlatformInfo(
            platform,
            param_name,
            size,
            result.as_mut_ptr() as *mut c_void,
            ptr::null_mut(),
        );
    }

    if let Some(pos) = result.iter().position(|&x| x == 0) {
        result.truncate(pos);
    }

    String::from_utf8(result).unwrap_or_default()
}

fn get_devices(platform: cl_platform_id, device_type: cl_device_type) -> Vec<cl_device_id> {
    let mut num_devices = 0;
    unsafe {
        clGetDeviceIDs(platform, device_type, 0, ptr::null_mut(), &mut num_devices);
    }

    let mut devices = Vec::with_capacity(num_devices as usize);
    unsafe {
        clGetDeviceIDs(platform, device_type, num_devices, devices.as_mut_ptr(), ptr::null_mut());
        devices.set_len(num_devices as usize);
    }
    devices
}

fn get_device_info(device: cl_device_id, param_name: cl_device_info) -> String {
    let mut size = 0;
    unsafe {
        clGetDeviceInfo(device, param_name, 0, ptr::null_mut(), &mut size);
    }

    if size == 0 {
        return String::new();
    }

    let mut result = vec![0u8; size];
    unsafe {
        clGetDeviceInfo(
            device,
            param_name,
            size,
            result.as_mut_ptr() as *mut c_void,
            ptr::null_mut(),
        );
    }

    if let Some(pos) = result.iter().position(|&x| x == 0) {
        result.truncate(pos);
    }

    String::from_utf8(result).unwrap_or_default()
}

fn get_device_info_raw<T>(device: cl_device_id, param_name: cl_device_info) -> T {
    let mut result: T = unsafe { std::mem::zeroed() };
    unsafe {
        clGetDeviceInfo(
            device,
            param_name,
            std::mem::size_of::<T>(),
            &mut result as *mut _ as *mut c_void,
            ptr::null_mut(),
        );
    }
    result
}

fn run_benchmark(platform: cl_platform_id, device: cl_device_id) {
    let device_name = get_device_info(device, CL_DEVICE_NAME);
    let platform_name = get_platform_info(platform, CL_PLATFORM_NAME);
    println!("--- Benchmarking Device: {} (Platform: {}) ---", device_name, platform_name);

    let mut err = 0;
    let context = unsafe {
        clCreateContext(
            ptr::null(),
            1,
            &device,
            None,
            ptr::null_mut(),
            &mut err,
        )
    };
    if err != CL_SUCCESS {
        eprintln!("Failed to create context: {}", err);
        return;
    }

    let queue = unsafe { clCreateCommandQueue(context, device, CL_QUEUE_PROFILING_ENABLE, &mut err) };
    if err != CL_SUCCESS {
        eprintln!("Failed to create command queue: {}", err);
        return;
    }

    let kernel_c_str = CString::new(OPENCL_KERNEL).unwrap();
    let program = unsafe {
        clCreateProgramWithSource(
            context,
            1,
            &kernel_c_str.as_ptr(),
            &OPENCL_KERNEL.len(),
            &mut err,
        )
    };
    if err != CL_SUCCESS {
        eprintln!("Failed to create program: {}", err);
        return;
    }

    let build_err = unsafe { clBuildProgram(program, 1, &device, ptr::null(), None, ptr::null_mut()) };
    if build_err != CL_SUCCESS {
        eprintln!("Failed to build program: {}", build_err);
        let mut log_size = 0;
        unsafe {
            clGetProgramBuildInfo(
                program,
                device,
                CL_PROGRAM_BUILD_LOG,
                0,
                ptr::null_mut(),
                &mut log_size,
            );
        }
        let mut log = vec![0; log_size];
        unsafe {
            clGetProgramBuildInfo(
                program,
                device,
                CL_PROGRAM_BUILD_LOG,
                log_size,
                log.as_mut_ptr() as *mut c_void,
                ptr::null_mut(),
            );
        }
        eprintln!("Build log:
{}", String::from_utf8_lossy(&log));
        return;
    }

    const DATA_SIZE: usize = 1024 * 1024;
    let h_a = vec![1i32; DATA_SIZE];
    let h_b = vec![2i32; DATA_SIZE];
    let mut h_c = vec![0i32; DATA_SIZE];

    let d_a = unsafe {
        clCreateBuffer(
            context,
            CL_MEM_READ_ONLY | CL_MEM_HOST_WRITE_ONLY,
            (DATA_SIZE * std::mem::size_of::<i32>()) as size_t,
            ptr::null_mut(),
            &mut err,
        )
    };
    if err != CL_SUCCESS {
        eprintln!("Failed to create buffer d_a: {}", err);
        return;
    }

    let d_b = unsafe {
        clCreateBuffer(
            context,
            CL_MEM_READ_ONLY | CL_MEM_HOST_WRITE_ONLY,
            (DATA_SIZE * std::mem::size_of::<i32>()) as size_t,
            ptr::null_mut(),
            &mut err,
        )
    };
    if err != CL_SUCCESS {
        eprintln!("Failed to create buffer d_b: {}", err);
        return;
    }

    let d_c = unsafe {
        clCreateBuffer(
            context,
            CL_MEM_WRITE_ONLY | CL_MEM_HOST_READ_ONLY,
            (DATA_SIZE * std::mem::size_of::<i32>()) as size_t,
            ptr::null_mut(),
            &mut err,
        )
    };
    if err != CL_SUCCESS {
        eprintln!("Failed to create buffer d_c: {}", err);
        return;
    }

    let kernel_name = CString::new("vecadd").unwrap();
    let kernel = unsafe { clCreateKernel(program, kernel_name.as_ptr(), &mut err) };
    if err != CL_SUCCESS {
        eprintln!("Failed to create kernel: {}", err);
        return;
    }

    unsafe {
        clSetKernelArg(kernel, 0, std::mem::size_of::<cl_mem>(), &d_a as *const _ as *const c_void);
        clSetKernelArg(kernel, 1, std::mem::size_of::<cl_mem>(), &d_b as *const _ as *const c_void);
        clSetKernelArg(kernel, 2, std::mem::size_of::<cl_mem>(), &d_c as *const _ as *const c_void);
        clSetKernelArg(kernel, 3, std::mem::size_of::<i32>(), &(DATA_SIZE as i32) as *const _ as *const c_void);
    }

    let start_overall = Instant::now();

    let mut write_event_a = ptr::null_mut();
    unsafe {
        clEnqueueWriteBuffer(
            queue,
            d_a,
            CL_FALSE,
            0,
            (DATA_SIZE * std::mem::size_of::<i32>()) as size_t,
            h_a.as_ptr() as *const c_void,
            0,
            ptr::null(),
            &mut write_event_a,
        );
    }

    let mut write_event_b = ptr::null_mut();
    unsafe {
        clEnqueueWriteBuffer(
            queue,
            d_b,
            CL_FALSE,
            0,
            (DATA_SIZE * std::mem::size_of::<i32>()) as size_t,
            h_b.as_ptr() as *const c_void,
            0,
            ptr::null(),
            &mut write_event_b,
        );
    }

    let mut kernel_event = ptr::null_mut();
    let write_events = [write_event_a, write_event_b];
    unsafe {
        clEnqueueNDRangeKernel(
            queue,
            kernel,
            1,
            ptr::null(),
            &DATA_SIZE,
            ptr::null(),
            2,
            write_events.as_ptr(),
            &mut kernel_event,
        );
    }

    let mut read_event_c = ptr::null_mut();
    unsafe {
        clEnqueueReadBuffer(
            queue,
            d_c,
            CL_TRUE,
            0,
            (DATA_SIZE * std::mem::size_of::<i32>()) as size_t,
            h_c.as_mut_ptr() as *mut c_void,
            1,
            &kernel_event,
            &mut read_event_c,
        );
    }

    unsafe {
        clFinish(queue);
    }

    let overall_ms = start_overall.elapsed().as_secs_f64() * 1000.0;

    let write_a_ms = get_event_duration_ms(write_event_a);
    let write_b_ms = get_event_duration_ms(write_event_b);
    let kernel_ms = get_event_duration_ms(kernel_event);
    let read_c_ms = get_event_duration_ms(read_event_c);

    println!("
--- Benchmark Results ({} elements) ---", DATA_SIZE);
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

    unsafe {
        clReleaseEvent(write_event_a);
        clReleaseEvent(write_event_b);
        clReleaseEvent(kernel_event);
        clReleaseEvent(read_event_c);
        clReleaseKernel(kernel);
        clReleaseProgram(program);
        clReleaseMemObject(d_a);
        clReleaseMemObject(d_b);
        clReleaseMemObject(d_c);
        clReleaseCommandQueue(queue);
        clReleaseContext(context);
    }
}

fn get_event_duration_ms(event: cl_event) -> f64 {
    let mut time_start = 0;
    let mut time_end = 0;

    unsafe {
        clGetEventProfilingInfo(
            event,
            CL_PROFILING_COMMAND_START,
            std::mem::size_of::<u64>(),
            &mut time_start as *mut _ as *mut c_void,
            ptr::null_mut(),
        );
        clGetEventProfilingInfo(
            event,
            CL_PROFILING_COMMAND_END,
            std::mem::size_of::<u64>(),
            &mut time_end as *mut _ as *mut c_void,
            ptr::null_mut(),
        );
    }

    (time_end - time_start) as f64 * 1e-6
}
