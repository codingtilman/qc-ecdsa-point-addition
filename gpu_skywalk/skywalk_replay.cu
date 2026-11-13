// Exact Skywalk/A2 operation-stream replay for independent paired evaluation.
// CUDA launch/error/timing structure and Keccak-f1600 are adapted from the
// MIT-licensed jieyilong/ecdsafail_gpu_toolkit (commit recorded in research log).
#include <cuda_runtime.h>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <algorithm>
#include <vector>

using u8 = uint8_t;
using u16 = uint16_t;
using u64 = uint64_t;
static constexpr u16 NO16 = 0xffffu;

#define CUDA_OK(x) do { cudaError_t e=(x); if(e!=cudaSuccess){ \
  std::fprintf(stderr,"CUDA %s:%d: %s\n",__FILE__,__LINE__,cudaGetErrorString(e)); std::exit(2); } } while(0)

#pragma pack(push,1)
struct GpuOp {
    u8 kind, pad0;
    u16 q2, q1, qt, ct, cc;
    u8 pad1[4];
};
#pragma pack(pop)
static_assert(sizeof(GpuOp) == 16, "packed op size");

struct StartState { u64 st[25]; u64 pt; };
struct Result { u64 value, phase, anc, tof; };

#define ROTL64(x,n) (((x)<<(n))|((x)>>(64-(n))))
__device__ __constant__ int kf_rotc[24] = {1,3,6,10,15,21,28,36,45,55,2,14,27,41,56,8,25,43,62,18,39,61,20,44};
__device__ __constant__ int kf_piln[24] = {10,7,11,17,18,3,5,16,8,21,24,4,15,23,19,13,12,2,20,14,22,9,6,1};
__device__ __constant__ u64 kf_rndc[24] = {
  0x0000000000000001ULL,0x0000000000008082ULL,0x800000000000808aULL,0x8000000080008000ULL,
  0x000000000000808bULL,0x0000000080000001ULL,0x8000000080008081ULL,0x8000000000008009ULL,
  0x000000000000008aULL,0x0000000000000088ULL,0x0000000080008009ULL,0x000000008000000aULL,
  0x000000008000808bULL,0x800000000000008bULL,0x8000000000008089ULL,0x8000000000008003ULL,
  0x8000000000008002ULL,0x8000000000000080ULL,0x000000000000800aULL,0x800000008000000aULL,
  0x8000000080008081ULL,0x8000000000008080ULL,0x0000000080000001ULL,0x8000000080008008ULL};

__device__ __forceinline__ void keccakf(u64 st[25]) {
    u64 t, bc[5];
    #pragma unroll 1
    for (int round=0; round<24; ++round) {
        #pragma unroll
        for(int i=0;i<5;i++) bc[i]=st[i]^st[i+5]^st[i+10]^st[i+15]^st[i+20];
        #pragma unroll
        for(int i=0;i<5;i++){ t=bc[(i+4)%5]^ROTL64(bc[(i+1)%5],1); for(int j=0;j<25;j+=5) st[j+i]^=t; }
        t=st[1];
        #pragma unroll
        for(int i=0;i<24;i++){ int j=kf_piln[i]; u64 z=st[j]; st[j]=ROTL64(t,kf_rotc[i]); t=z; }
        for(int j=0;j<25;j+=5){
            #pragma unroll
            for(int i=0;i<5;i++) bc[i]=st[j+i];
            #pragma unroll
            for(int i=0;i<5;i++) st[j+i]^=(~bc[(i+1)%5])&bc[(i+2)%5];
        }
        st[0]^=kf_rndc[round];
    }
}

struct Squeezer { u64 st[25]; int pt; };
__device__ __forceinline__ u64 next_u64(Squeezer &s) {
    if (s.pt == 136) { keccakf(s.st); s.pt=0; }
    u64 v=s.st[s.pt>>3]; s.pt+=8; return v;
}

template<bool COUNT_TOFFOLI>
__device__ __forceinline__ Result replay_batch(
    const GpuOp* ops, u64 nops, u64 nq, u64 nb, u64 shots, u64 batches,
    const u16* out_q, u64 nout, const u16* declared, u64 ndecl,
    const u64* initial, const u64* expected, const StartState* starts,
    u64 b, u64* mem, int value_only) {
    u64* q=mem;
    u64* bits=mem+nq;
    u64 stride=nq+nb;
    for(u64 i=0;i<nq;i++) q[i]=initial[b*stride+i];
    for(u64 i=0;i<nb;i++) bits[i]=initial[b*stride+nq+i];
    u64 phase=0, tof=0, base=~0ULL, stack[4];
    int depth=0;
    Squeezer sq;
    #pragma unroll
    for(int i=0;i<25;i++) sq.st[i]=starts[b].st[i];
    sq.pt=(int)starts[b].pt;

    #pragma unroll 1
    for(u64 i=0;i<nops;i++) {
        GpuOp o=ops[i];
        u64 cond=base;
        if(o.cc!=NO16) cond&=bits[o.cc];
        if constexpr (COUNT_TOFFOLI) {
            if(o.kind==13 || o.kind==14) tof+=(u64)__popcll(cond);
        }
        switch(o.kind) {
            case 0: phase^=cond; break; // Neg
            case 3: bits[o.ct]^=cond; break;
            case 4: bits[o.ct]&=~cond; break;
            case 5: bits[o.ct]|=cond; break;
            case 6: q[o.qt]^=cond; break;
            case 7: phase^=cond&q[o.qt]; break;
            case 8: q[o.qt]^=cond&q[o.q1]; break;
            case 9: phase^=cond&q[o.qt]&q[o.q1]; break;
            case 10: {
                u64 a=q[o.q1], t=q[o.qt];
                a^=t; t^=cond&a; a^=t;
                q[o.q1]=a; q[o.qt]=t;
                break;
            }
            case 11: {
                u64 r=next_u64(sq);
                if(!value_only) phase^=q[o.qt]&r&cond;
                q[o.qt]&=~cond;
                break;
            }
            case 12: {
                u64 r=next_u64(sq);
                bits[o.ct]&=~cond;
                bits[o.ct]^=r&cond;
                if(!value_only) phase^=q[o.qt]&r&cond;
                q[o.qt]&=~cond;
                break;
            }
            case 13: q[o.qt]^=cond&q[o.q1]&q[o.q2]; break;
            case 14: phase^=cond&q[o.qt]&q[o.q1]&q[o.q2]; break;
            case 15: stack[depth++]=base; base&=bits[o.cc]; break;
            case 16: base=stack[--depth]; break;
            default: break;
        }
    }
    u64 bs=shots-b*64; if(bs>64) bs=64;
    u64 mask=bs==64?~0ULL:((1ULL<<bs)-1);
    u64 bad=0;
    for(u64 i=0;i<nout;i++) {
        u16 w=out_q[i]; u64 actual=(w&0x8000u)?bits[w&0x7fffu]:q[w];
        bad|=actual^expected[b*nout+i];
    }
    for(u64 i=0;i<ndecl;i++) q[declared[i]]=0;
    u64 anc=0; for(u64 i=0;i<nq;i++) anc|=q[i];
    return {bad&mask,phase&mask,anc&mask,tof};
}

template<bool COUNT_TOFFOLI>
__global__ void replay_kernel(
    const GpuOp* ops, u64 nops, u64 nq, u64 nb, u64 shots, u64 batches,
    const u16* out_q, u64 nout, const u16* declared, u64 ndecl,
    const u64* initial, const u64* expected, const StartState* starts, Result* results,
    int value_only, u64 batches_per_block) {
    if (threadIdx.x != 0) return;
    extern __shared__ u64 mem[];
    u64 first=(u64)blockIdx.x*batches_per_block;
    for(u64 j=0;j<batches_per_block;j++) {
        u64 b=first+j;
        if(b>=batches) break;
        Result r=replay_batch<COUNT_TOFFOLI>(ops,nops,nq,nb,shots,batches,out_q,nout,declared,ndecl,
                                             initial,expected,starts,b,mem,value_only);
        results[b]=r;
        if(batches_per_block>1 && (r.value|r.phase|r.anc)) break;
    }
}

static u64 rd64(const u8*& p) { u64 x; std::memcpy(&x,p,8); p+=8; return x; }
template<class T> static std::vector<T> take(const u8*& p, size_t n) {
    std::vector<T> v(n); std::memcpy(v.data(),p,n*sizeof(T)); p+=n*sizeof(T); return v;
}

int main(int argc, char** argv) {
    if(argc!=2){ std::fprintf(stderr,"usage: skywalk_replay FILE.pack\n"); return 2; }
    auto full0=std::chrono::steady_clock::now();
    std::FILE* f=std::fopen(argv[1],"rb"); if(!f){std::perror("open");return 2;}
    std::fseek(f,0,SEEK_END); long sz=std::ftell(f); std::rewind(f);
    std::vector<u8> file((size_t)sz); if(std::fread(file.data(),1,file.size(),f)!=file.size()) return 2; std::fclose(f);
    const u8* p=file.data();
    if(std::memcmp(p,"SKYGPU1\0",8)!=0){std::fprintf(stderr,"bad pack magic\n");return 2;} p+=8;
    u64 version=rd64(p), nops=rd64(p), nq=rd64(p), nb=rd64(p), shots=rd64(p), batches=rd64(p);
    u64 nmeas=rd64(p), nout=rd64(p), ndecl=rd64(p), depth=rd64(p);
    if((version!=1 && version!=2) || depth>4){std::fprintf(stderr,"bad header\n");return 2;}
    auto ops=take<GpuOp>(p,nops); auto outq=take<u16>(p,nout); auto declared=take<u16>(p,ndecl);
    auto initial=take<u64>(p,batches*(nq+nb)); auto expected=take<u64>(p,batches*nout);
    auto starts=take<StartState>(p,batches);
    std::vector<Result> refs;
    if(version==1) refs=take<Result>(p,batches);
    if((size_t)(p-file.data())!=file.size()){std::fprintf(stderr,"trailing bytes %zu\n",file.size()-(p-file.data()));return 2;}
    bool value_only=std::getenv("SKYWALK_VALUE_ONLY") && std::strcmp(std::getenv("SKYWALK_VALUE_ONLY"),"1")==0;
    bool screen_only=std::getenv("SKYWALK_SCREEN_ONLY") && std::strcmp(std::getenv("SKYWALK_SCREEN_ONLY"),"1")==0;
    u64 early_batches=1;
    if(const char* v=std::getenv("SKYWALK_EARLY_BATCHES_PER_NONCE")) early_batches=std::strtoull(v,nullptr,10);
    if(early_batches<1 || batches%early_batches!=0){std::fprintf(stderr,"bad SKYWALK_EARLY_BATCHES_PER_NONCE\n");return 2;}
    u64 serialized_nops=nops;
    if(value_only) {
        ops.erase(std::remove_if(ops.begin(),ops.end(),[](const GpuOp& o){
            return o.kind==0 || o.kind==7 || o.kind==9 || o.kind==14;
        }),ops.end());
        nops=ops.size();
    }
    auto load1=std::chrono::steady_clock::now();

    CUDA_OK(cudaSetDevice(0)); CUDA_OK(cudaFree(0));
    auto ctx1=std::chrono::steady_clock::now();
    GpuOp *dops; u16 *dout,*ddecl; u64 *dinit,*dexp; StartState *dst; Result *dres;
    CUDA_OK(cudaMalloc(&dops,ops.size()*sizeof(GpuOp))); CUDA_OK(cudaMalloc(&dout,outq.size()*sizeof(u16)));
    CUDA_OK(cudaMalloc(&ddecl,declared.size()*sizeof(u16))); CUDA_OK(cudaMalloc(&dinit,initial.size()*sizeof(u64)));
    CUDA_OK(cudaMalloc(&dexp,expected.size()*sizeof(u64))); CUDA_OK(cudaMalloc(&dst,starts.size()*sizeof(StartState)));
    CUDA_OK(cudaMalloc(&dres,batches*sizeof(Result)));
    CUDA_OK(cudaMemset(dres,0,batches*sizeof(Result)));
    CUDA_OK(cudaMemcpy(dops,ops.data(),ops.size()*sizeof(GpuOp),cudaMemcpyHostToDevice));
    CUDA_OK(cudaMemcpy(dout,outq.data(),outq.size()*sizeof(u16),cudaMemcpyHostToDevice));
    CUDA_OK(cudaMemcpy(ddecl,declared.data(),declared.size()*sizeof(u16),cudaMemcpyHostToDevice));
    CUDA_OK(cudaMemcpy(dinit,initial.data(),initial.size()*sizeof(u64),cudaMemcpyHostToDevice));
    CUDA_OK(cudaMemcpy(dexp,expected.data(),expected.size()*sizeof(u64),cudaMemcpyHostToDevice));
    CUDA_OK(cudaMemcpy(dst,starts.data(),starts.size()*sizeof(StartState),cudaMemcpyHostToDevice));
    auto copy1=std::chrono::steady_clock::now();

    cudaEvent_t a,z; CUDA_OK(cudaEventCreate(&a)); CUDA_OK(cudaEventCreate(&z)); CUDA_OK(cudaEventRecord(a));
    size_t shmem=(nq+nb)*sizeof(u64);
    if(screen_only) {
        replay_kernel<false><<<(unsigned)(batches/early_batches),1,shmem>>>(dops,nops,nq,nb,shots,batches,dout,nout,ddecl,ndecl,dinit,dexp,dst,dres,value_only?1:0,early_batches);
    } else {
        replay_kernel<true><<<(unsigned)(batches/early_batches),1,shmem>>>(dops,nops,nq,nb,shots,batches,dout,nout,ddecl,ndecl,dinit,dexp,dst,dres,value_only?1:0,early_batches);
    }
    CUDA_OK(cudaGetLastError()); CUDA_OK(cudaEventRecord(z)); CUDA_OK(cudaEventSynchronize(z));
    float kernel_ms=0; CUDA_OK(cudaEventElapsedTime(&kernel_ms,a,z));
    std::vector<Result> got(batches); CUDA_OK(cudaMemcpy(got.data(),dres,batches*sizeof(Result),cudaMemcpyDeviceToHost));
    if(const char* mask_out=std::getenv("SKYWALK_MASK_OUT")) {
        std::FILE* mf=std::fopen(mask_out,"wb"); if(!mf){std::perror("mask output");return 2;}
        if(std::fwrite(got.data(),sizeof(Result),got.size(),mf)!=got.size()){std::perror("mask write");return 2;}
        if(std::fclose(mf)!=0){std::perror("mask close");return 2;}
        std::fprintf(stderr,"MASK_OUT path=%s records=%zu bytes=%zu\n",mask_out,got.size(),got.size()*sizeof(Result));
    }
    auto result1=std::chrono::steady_clock::now();

    u64 value=0,phase=0,anc=0,any=0,tof=0,dis_v=0,dis_p=0,dis_a=0,dis_t=0;
    for(size_t i=0;i<batches;i++){
        value+=__builtin_popcountll(got[i].value); phase+=__builtin_popcountll(got[i].phase);
        anc+=__builtin_popcountll(got[i].anc); tof+=got[i].tof;
        any+=__builtin_popcountll(got[i].value|got[i].phase|got[i].anc);
        if(version==1){
            dis_v+=__builtin_popcountll(got[i].value^refs[i].value);
            dis_p+=__builtin_popcountll(got[i].phase^refs[i].phase);
            dis_a+=__builtin_popcountll(got[i].anc^refs[i].anc);
            dis_t+=(got[i].tof!=refs[i].tof);
        }
    }
    CUDA_OK(cudaFree(dops)); CUDA_OK(cudaFree(dout)); CUDA_OK(cudaFree(ddecl)); CUDA_OK(cudaFree(dinit));
    CUDA_OK(cudaFree(dexp)); CUDA_OK(cudaFree(dst)); CUDA_OK(cudaFree(dres)); CUDA_OK(cudaEventDestroy(a)); CUDA_OK(cudaEventDestroy(z));
    CUDA_OK(cudaDeviceReset());
    auto full1=std::chrono::steady_clock::now();
    auto sec=[](auto x,auto y){return std::chrono::duration<double>(y-x).count();};
    std::printf("GPU mode=%s%s early_batches_per_nonce=%llu shots=%llu batches=%llu ops=%llu serialized_ops=%llu nq=%llu nb=%llu rng_words_per_batch=%llu\n",
        value_only?"value_only":"exact",screen_only?"_screen":"",(unsigned long long)early_batches,
        (unsigned long long)shots,(unsigned long long)batches,(unsigned long long)nops,
        (unsigned long long)serialized_nops,
        (unsigned long long)nq,(unsigned long long)nb,(unsigned long long)nmeas);
    std::printf("GPU value=%llu phase_shots=%llu anc_shots=%llu any_shots=%llu T_total=%llu T_mean=%.3f\n",
        (unsigned long long)value,(unsigned long long)phase,(unsigned long long)anc,(unsigned long long)any,
        (unsigned long long)tof,(double)tof/(double)shots);
    if(version==1){
        std::printf("AGREEMENT value_bits=%llu phase_bits=%llu anc_bits=%llu toffoli_batches=%llu\n",
            (unsigned long long)dis_v,(unsigned long long)dis_p,(unsigned long long)dis_a,(unsigned long long)dis_t);
    } else {
        std::printf("AGREEMENT skipped=no_cpu_refs (GPU screen pack v2)\n");
    }
    std::printf("TIMING file_load=%.3fs cuda_context=%.3fs alloc_h2d=%.3fs kernel=%.3fs d2h_reduce=%.3fs cold_total=%.3fs\n",
        sec(full0,load1),sec(load1,ctx1),sec(ctx1,copy1),kernel_ms/1000.0,sec(copy1,result1)-kernel_ms/1000.0,sec(full0,full1));
    return version==1 && (dis_v|dis_p|dis_a|dis_t)?1:0;
}
