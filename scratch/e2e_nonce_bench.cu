#include <cuda_runtime.h>
#include <omp.h>

#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>

#define CUDA_OK(call) do { \
    cudaError_t e_ = (call); \
    if (e_ != cudaSuccess) { \
        std::fprintf(stderr, "CUDA failure %s:%d: %s\n", __FILE__, __LINE__, cudaGetErrorString(e_)); \
        std::exit(2); \
    } \
} while (0)

__host__ __device__ static inline uint64_t mix_nonce(uint64_t x, uint32_t rounds) {
    for (uint32_t r = 0; r < rounds; ++r) {
        x ^= x >> 30;
        x *= 0xbf58476d1ce4e5b9ULL;
        x ^= x >> 27;
        x *= 0x94d049bb133111ebULL;
        x ^= x >> 31;
        x += 0x9e3779b97f4a7c15ULL ^ static_cast<uint64_t>(r);
    }
    return x;
}

__global__ void scan_kernel(const uint64_t* in, uint64_t* out, size_t n, uint32_t rounds) {
    size_t i = static_cast<size_t>(blockIdx.x) * blockDim.x + threadIdx.x;
    size_t stride = static_cast<size_t>(blockDim.x) * gridDim.x;
    for (; i < n; i += stride) out[i] = mix_nonce(in[i], rounds);
}

static inline uint64_t input_at(size_t i) {
    return 0xd1b54a32d192ed03ULL ^ (static_cast<uint64_t>(i) * 0x9e3779b97f4a7c15ULL);
}

static uint64_t checksum(const uint64_t* p, size_t n) {
    uint64_t s = 0x243f6a8885a308d3ULL;
    for (size_t i = 0; i < n; ++i) s = (s << 7 | s >> 57) ^ p[i];
    return s;
}

static double seconds_since(std::chrono::steady_clock::time_point t0) {
    return std::chrono::duration<double>(std::chrono::steady_clock::now() - t0).count();
}

int main(int argc, char** argv) {
    if (argc != 4 || (std::strcmp(argv[1], "cpu") && std::strcmp(argv[1], "gpu"))) {
        std::fprintf(stderr, "usage: %s cpu|gpu item_count rounds\n", argv[0]);
        return 2;
    }
    const bool gpu = std::strcmp(argv[1], "gpu") == 0;
    const size_t n = std::strtoull(argv[2], nullptr, 10);
    const uint32_t rounds = static_cast<uint32_t>(std::strtoul(argv[3], nullptr, 10));
    const size_t bytes = n * sizeof(uint64_t);
    if (!n || !rounds || bytes / sizeof(uint64_t) != n) return 2;

    const auto e2e0 = std::chrono::steady_clock::now();
    uint64_t* in = nullptr;
    uint64_t* out = nullptr;
    double compute_s = 0.0;
    int workers = 1;

    if (!gpu) {
        in = static_cast<uint64_t*>(std::malloc(bytes));
        out = static_cast<uint64_t*>(std::malloc(bytes));
        if (!in || !out) return 3;
        #pragma omp parallel for schedule(static)
        for (size_t i = 0; i < n; ++i) in[i] = input_at(i);
        const auto c0 = std::chrono::steady_clock::now();
        #pragma omp parallel
        {
            #pragma omp single
            workers = omp_get_num_threads();
            #pragma omp for schedule(static)
            for (size_t i = 0; i < n; ++i) out[i] = mix_nonce(in[i], rounds);
        }
        compute_s = seconds_since(c0);
    } else {
        CUDA_OK(cudaSetDevice(0));
        CUDA_OK(cudaFree(nullptr));
        CUDA_OK(cudaHostAlloc(&in, bytes, cudaHostAllocDefault));
        CUDA_OK(cudaHostAlloc(&out, bytes, cudaHostAllocDefault));
        #pragma omp parallel for schedule(static)
        for (size_t i = 0; i < n; ++i) in[i] = input_at(i);
        uint64_t *din = nullptr, *dout = nullptr;
        CUDA_OK(cudaMalloc(&din, bytes));
        CUDA_OK(cudaMalloc(&dout, bytes));
        CUDA_OK(cudaMemcpy(din, in, bytes, cudaMemcpyHostToDevice));
        cudaEvent_t a, b;
        CUDA_OK(cudaEventCreate(&a));
        CUDA_OK(cudaEventCreate(&b));
        CUDA_OK(cudaEventRecord(a));
        const int threads = 256;
        int blocks = static_cast<int>((n + threads - 1) / threads);
        if (blocks > 65535) blocks = 65535;
        scan_kernel<<<blocks, threads>>>(din, dout, n, rounds);
        CUDA_OK(cudaGetLastError());
        CUDA_OK(cudaEventRecord(b));
        CUDA_OK(cudaEventSynchronize(b));
        float kernel_ms = 0.0f;
        CUDA_OK(cudaEventElapsedTime(&kernel_ms, a, b));
        compute_s = kernel_ms / 1000.0;
        CUDA_OK(cudaMemcpy(out, dout, bytes, cudaMemcpyDeviceToHost));
        CUDA_OK(cudaDeviceSynchronize());
        CUDA_OK(cudaEventDestroy(a));
        CUDA_OK(cudaEventDestroy(b));
        CUDA_OK(cudaFree(din));
        CUDA_OK(cudaFree(dout));
        workers = blocks * threads;
    }

    const uint64_t sum = checksum(out, n);
    if (gpu) {
        CUDA_OK(cudaFreeHost(in));
        CUDA_OK(cudaFreeHost(out));
        CUDA_OK(cudaDeviceReset());
    } else {
        std::free(in);
        std::free(out);
    }
    const double e2e_s = seconds_since(e2e0);
    std::printf("mode=%s n=%zu rounds=%u workers=%d compute_s=%.6f e2e_s=%.6f compute_Mitems_s=%.3f e2e_Mitems_s=%.3f checksum=%016llx\n",
        gpu ? "gpu" : "cpu", n, rounds, workers, compute_s, e2e_s,
        n / compute_s / 1e6, n / e2e_s / 1e6,
        static_cast<unsigned long long>(sum));
    return 0;
}
