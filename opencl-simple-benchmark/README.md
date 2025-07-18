# A simple OpenCL benchmark

## Compile and execute C++ code

```bash
$ g++ -o benchmark_cc benchmark.cc -lOpenCL && ./benchmark_cc
--- Discovered OpenCL Platforms and Devices ---

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Platform 0: AMD Accelerated Parallel Processing (Version: OpenCL 2.1 AMD-APP (3649.0))
  Device 0: gfx1103 (Type: GPU)
--- Benchmarking Device: gfx1103 (Platform: AMD Accelerated Parallel Processing) ---

--- Benchmark Results (1048576 elements) ---
Data Size: 4 MB
Write A (Host -> Device): 0.181164 ms
Write B (Host -> Device): 0.175193 ms
Kernel Execution Time:    0.174913 ms
Read C (Device -> Host):  0.296581 ms
Total Overall Time (measured by host clock): 2.80076 ms
Result verification: PASSED (first 10 elements are correct)

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Platform 1: NVIDIA CUDA (Version: OpenCL 3.0 CUDA 12.9.90)
  Device 0: NVIDIA RTX A500 Laptop GPU (Type: GPU)
--- Benchmarking Device: NVIDIA RTX A500 Laptop GPU (Platform: NVIDIA CUDA) ---

--- Benchmark Results (1048576 elements) ---
Data Size: 4 MB
Write A (Host -> Device): 0.50752 ms
Write B (Host -> Device): 0.476448 ms
Kernel Execution Time:    0.12288 ms
Read C (Device -> Host):  0.349088 ms
Total Overall Time (measured by host clock): 1.96069 ms
Result verification: PASSED (first 10 elements are correct)

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Platform 2: Portable Computing Language (Version: OpenCL 3.0 PoCL 5.0+debian  Linux, None+Asserts, RELOC, SPIR, LLVM 16.0.6, SLEEF, DISTRO, POCL_DEBUG)
  Device 0: cpu-skylake-avx512-AMD Ryzen 7 PRO 7840HS w/ Radeon 780M Graphics (Type: CPU)
--- Benchmarking Device: cpu-skylake-avx512-AMD Ryzen 7 PRO 7840HS w/ Radeon 780M Graphics (Platform: Portable Computing Language) ---

--- Benchmark Results (1048576 elements) ---
Data Size: 4 MB
Write A (Host -> Device): 1.55605 ms
Write B (Host -> Device): 1.53861 ms
Kernel Execution Time:    0.529754 ms
Read C (Device -> Host):  0.254307 ms
Total Overall Time (measured by host clock): 4.14077 ms
Result verification: PASSED (first 10 elements are correct)
```

## Compile and execute Rust code

```bash
$ cargo build --release && ./target/release/benchmark
--- Discovered OpenCL Platforms and Devices ---
Platform 0: Portable Computing Language (Version: OpenCL 3.0 PoCL 5.0+debian  Linux, None+Asserts, RELOC, SPIR, LLVM 16.0.6, SLEEF, DISTRO, POCL_DEBUG)
  Device 0: cpu-skylake-avx512-AMD Ryzen 7 PRO 7840HS w/ Radeon 780M Graphics (Type: CPU)
--- Benchmarking Device: cpu-skylake-avx512-AMD Ryzen 7 PRO 7840HS w/ Radeon 780M Graphics (Platform: Portable Computing Language) ---

--- Benchmark Results (1048576 elements) ---
Data Size: 4.00 MB
Write A (Host -> Device): 2.412511 ms
Write B (Host -> Device): 3.100151 ms
Kernel Execution Time:    0.928145 ms
Read C (Device -> Host):  2.421799 ms
Total Overall Time (measured by host clock): 9.366765 ms
Result verification: PASSED (first 10 elements are correct)

Platform 1: AMD Accelerated Parallel Processing (Version: OpenCL 2.1 AMD-APP (3649.0))
  Device 0: gfx1103 (Type: GPU)
--- Benchmarking Device: gfx1103 (Platform: AMD Accelerated Parallel Processing) ---

--- Benchmark Results (1048576 elements) ---
Data Size: 4.00 MB
Write A (Host -> Device): 0.203666 ms
Write B (Host -> Device): 0.175332 ms
Kernel Execution Time:    0.175132 ms
Read C (Device -> Host):  1.463683 ms
Total Overall Time (measured by host clock): 3.704097 ms
Result verification: PASSED (first 10 elements are correct)

Platform 2: NVIDIA CUDA (Version: OpenCL 3.0 CUDA 12.8.97)
  Device 0: NVIDIA RTX A500 Laptop GPU (Type: GPU)
--- Benchmarking Device: NVIDIA RTX A500 Laptop GPU (Platform: NVIDIA CUDA) ---

--- Benchmark Results (1048576 elements) ---
Data Size: 4.00 MB
Write A (Host -> Device): 0.353792 ms
Write B (Host -> Device): 0.338336 ms
Kernel Execution Time:    0.130048 ms
Read C (Device -> Host):  1.075552 ms
Total Overall Time (measured by host clock): 3.152933 ms
Result verification: PASSED (first 10 elements are correct)
```
