#include <iostream>
#include <vector>
#include <string>

// Define CL_HPP_TARGET_OPENCL_VERSION to suppress warning and explicitly target OpenCL 3.0
#define CL_HPP_TARGET_OPENCL_VERSION 300
// Include opencl.hpp directly as cl2.hpp has been renamed
#include <CL/opencl.hpp>
#include <chrono>    // For high-resolution timing (optional, for overall execution)

// Define the OpenCL kernel code for vector addition
static const std::string opencl_kernel = R"(
    __kernel void vecadd
    (
        __global int *A,
        __global int *B,
        __global int *C,
        const int N
    )
    {
        int id = get_global_id(0);
        if (id < N) {
            C[id] = A[id] + B[id];
        }
    }
)";

// Function to print OpenCL errors
void print_cl_error(cl_int err) {
    std::cerr << "OpenCL Error: " << err << std::endl;
}

// Function to run benchmark on a specific OpenCL device
void run_benchmark(const cl::Platform& platform, const cl::Device& device) {
    std::string deviceName;
    device.getInfo(CL_DEVICE_NAME, &deviceName);
    std::string platformName;
    platform.getInfo(CL_PLATFORM_NAME, &platformName);

    std::cout << "--- Benchmarking Device: " << deviceName
              << " (Platform: " << platformName << ") ---" << std::endl;

    cl_int err;

    // --- 1. Create Context and Command Queue ---
    cl::Context context(device);
    // Enable profiling on the command queue to measure execution times
    cl::CommandQueue queue(context, device, CL_QUEUE_PROFILING_ENABLE, &err);
    if (err != CL_SUCCESS) {
        print_cl_error(err);
        std::cerr << "Failed to create command queue for device: " << deviceName << std::endl;
        return;
    }

    // --- 2. Build the OpenCL Program ---
    cl::Program program(context, opencl_kernel);
    err = program.build({device});
    if (err != CL_SUCCESS) {
        print_cl_error(err);
        std::cerr << "Failed to build kernel program for device: " << deviceName << std::endl;
        // Print build log for debugging if building fails
        std::string buildLog;
        program.getBuildInfo(device, CL_PROGRAM_BUILD_LOG, &buildLog);
        std::cerr << "Build Log:\n" << buildLog << std::endl;
        return;
    }

    // --- 3. Prepare Host Data ---
    const int DATA_SIZE = 1024 * 1024; // 1M elements for demonstration (approx. 4MB)
    std::vector<int> h_A(DATA_SIZE, 1);
    std::vector<int> h_B(DATA_SIZE, 2);
    std::vector<int> h_C(DATA_SIZE); // Result vector

    // --- 4. Create Device Buffers ---
    // CL_MEM_READ_ONLY / CL_MEM_WRITE_ONLY: Hints for memory access patterns
    // CL_MEM_HOST_WRITE_ONLY / CL_MEM_HOST_READ_ONLY: Hints for host access patterns
    cl::Buffer d_A(context, CL_MEM_READ_ONLY | CL_MEM_HOST_WRITE_ONLY, sizeof(int) * DATA_SIZE, nullptr, &err);
    if (err != CL_SUCCESS) { print_cl_error(err); std::cerr << "Failed to create buffer d_A." << std::endl; return; }
    cl::Buffer d_B(context, CL_MEM_READ_ONLY | CL_MEM_HOST_WRITE_ONLY, sizeof(int) * DATA_SIZE, nullptr, &err);
    if (err != CL_SUCCESS) { print_cl_error(err); std::cerr << "Failed to create buffer d_B." << std::endl; return; }
    cl::Buffer d_C(context, CL_MEM_WRITE_ONLY | CL_MEM_HOST_READ_ONLY, sizeof(int) * DATA_SIZE, nullptr, &err);
    if (err != CL_SUCCESS) { print_cl_error(err); std::cerr << "Failed to create buffer d_C." << std::endl; return; }

    // --- 5. Create Kernel Object and Set Arguments ---
    cl::Kernel kernel(program, "vecadd", &err);
    if (err != CL_SUCCESS) { print_cl_error(err); std::cerr << "Failed to create kernel 'vecadd'." << std::endl; return; }
    kernel.setArg(0, d_A);
    kernel.setArg(1, d_B);
    kernel.setArg(2, d_C);
    kernel.setArg(3, DATA_SIZE);

    // --- 6. Perform Benchmark Operations ---
    cl::Event writeEventA, writeEventB, kernelEvent, readEventC;

    // Data transfer: Host to Device (non-blocking)
    auto start_overall = std::chrono::high_resolution_clock::now();
    queue.enqueueWriteBuffer(d_A, CL_FALSE, 0, sizeof(int) * DATA_SIZE, h_A.data(), nullptr, &writeEventA);
    queue.enqueueWriteBuffer(d_B, CL_FALSE, 0, sizeof(int) * DATA_SIZE, h_B.data(), nullptr, &writeEventB);

    // Enqueue Kernel (waits for write events to complete)
    std::vector<cl::Event> writeEvents = {writeEventA, writeEventB};
    cl::NDRange globalWorkSize(DATA_SIZE);
    // cl::NullRange lets OpenCL automatically choose a local work size.
    queue.enqueueNDRangeKernel(kernel, cl::NullRange, globalWorkSize, cl::NullRange, &writeEvents, &kernelEvent);

    // Data transfer: Device to Host (blocking, waits for kernel completion)
    std::vector<cl::Event> kernelDependencies = {kernelEvent};
    queue.enqueueReadBuffer(d_C, CL_TRUE, 0, sizeof(int) * DATA_SIZE, h_C.data(), &kernelDependencies, &readEventC);

    // Finish all commands in the queue to ensure profiling data is available
    queue.finish();

    auto end_overall = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double, std::milli> overall_ms = end_overall - start_overall;

    // --- 7. Get Profiling Info ---
    cl_ulong timeStart_writeA, timeEnd_writeA;
    cl_ulong timeStart_writeB, timeEnd_writeB;
    cl_ulong timeStart_kernel, timeEnd_kernel;
    cl_ulong timeStart_readC, timeEnd_readC;

    writeEventA.getProfilingInfo(CL_PROFILING_COMMAND_START, &timeStart_writeA);
    writeEventA.getProfilingInfo(CL_PROFILING_COMMAND_END, &timeEnd_writeA);

    writeEventB.getProfilingInfo(CL_PROFILING_COMMAND_START, &timeStart_writeB);
    writeEventB.getProfilingInfo(CL_PROFILING_COMMAND_END, &timeEnd_writeB);

    kernelEvent.getProfilingInfo(CL_PROFILING_COMMAND_START, &timeStart_kernel);
    kernelEvent.getProfilingInfo(CL_PROFILING_COMMAND_END, &timeEnd_kernel);

    readEventC.getProfilingInfo(CL_PROFILING_COMMAND_START, &timeStart_readC);
    readEventC.getProfilingInfo(CL_PROFILING_COMMAND_END, &timeEnd_readC);

    double writeAMs = (double)(timeEnd_writeA - timeStart_writeA) * 1e-6;
    double writeBMs = (double)(timeEnd_writeB - timeStart_writeB) * 1e-6;
    double kernelMs = (double)(timeEnd_kernel - timeStart_kernel) * 1e-6;
    double readCMs = (double)(timeEnd_readC - timeStart_readC) * 1e-6;

    std::cout << "\n--- Benchmark Results (" << DATA_SIZE << " elements) ---" << std::endl;
    std::cout << "Data Size: " << DATA_SIZE * sizeof(int) / (1024.0 * 1024.0) << " MB" << std::endl;
    std::cout << "Write A (Host -> Device): " << writeAMs << " ms" << std::endl;
    std::cout << "Write B (Host -> Device): " << writeBMs << " ms" << std::endl;
    std::cout << "Kernel Execution Time:    " << kernelMs << " ms" << std::endl;
    std::cout << "Read C (Device -> Host):  " << readCMs << " ms" << std::endl;
    std::cout << "Total Overall Time (measured by host clock): " << overall_ms.count() << " ms" << std::endl;

    // --- 8. Verify Results (Optional) ---
    bool correct = true;
    for (int i = 0; i < 10 && i < DATA_SIZE; ++i) { // Check first 10 elements
        if (h_C[i] != (h_A[i] + h_B[i])) {
            correct = false;
            break;
        }
    }
    if (correct) {
        std::cout << "Result verification: PASSED (first 10 elements are correct)" << std::endl;
    } else {
        std::cout << "Result verification: FAILED" << std::endl;
    }
    // Removed the ~~~~~ separator from here as per request
}

int main() {
    // --- 1. Get all OpenCL Platforms ---
    std::vector<cl::Platform> platforms;
    cl_int err = cl::Platform::get(&platforms);
    if (err != CL_SUCCESS) {
        print_cl_error(err);
        std::cerr << "Error getting OpenCL platforms. Exiting." << std::endl;
        return -1;
    }

    if (platforms.empty()) {
        std::cerr << "No OpenCL platforms found! Please ensure OpenCL drivers are installed." << std::endl;
        return -1;
    }

    std::cout << "--- Discovered OpenCL Platforms and Devices ---" << std::endl;

    // --- 2. Enumerate and Benchmark all Platforms and Devices ---
    int platformIdx = 0;
    for (const auto& platform : platforms) {
        std::string platformName;
        platform.getInfo(CL_PLATFORM_NAME, &platformName);

        // Add the long separator ONLY before the NVIDIA CUDA platform
        if (platformName.find("NVIDIA CUDA") != std::string::npos && platformIdx > 0) {
            std::cout << "\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~" << std::endl;
        }

        std::string platformVersion;
        platform.getInfo(CL_PLATFORM_VERSION, &platformVersion);
        std::cout << "Platform " << platformIdx << ": " << platformName << " (Version: " << platformVersion << ")" << std::endl;

        std::vector<cl::Device> devices;
        err = platform.getDevices(CL_DEVICE_TYPE_ALL, &devices); // Get all device types for this platform
        if (err != CL_SUCCESS) {
            std::cerr << "  Error getting devices for platform " << platformName << ": " << err << std::endl;
            platformIdx++;
            continue;
        }

        if (devices.empty()) {
            std::cout << "  No devices found for this platform." << std::endl;
        } else {
            int deviceIdx = 0;
            for (const auto& device : devices) {
                std::string deviceName;
                device.getInfo(CL_DEVICE_NAME, &deviceName);
                cl_device_type deviceType;
                device.getInfo(CL_DEVICE_TYPE, &deviceType);
                std::string typeStr = (deviceType == CL_DEVICE_TYPE_CPU) ? "CPU" :
                                      (deviceType == CL_DEVICE_TYPE_GPU) ? "GPU" :
                                      (deviceType == CL_DEVICE_TYPE_ACCELERATOR) ? "Accelerator" : "Unknown";

                std::cout << "  Device " << deviceIdx << ": " << deviceName << " (Type: " << typeStr << ")" << std::endl;

                // Call the benchmark function for each discovered device
                run_benchmark(platform, device);

                deviceIdx++;
            }
        }
        platformIdx++;
    }

    return 0;
}
