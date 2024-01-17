use std::env;
use std::path::{Path, PathBuf};

use cc::Build;

fn compile_bindings(out_path: &Path) {
    let bindings = bindgen::Builder::default()
        .header("./binding.h")
        .blocklist_function("tokenCallback")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn compile_opencl(cx: &mut Build, cxx: &mut Build) {
    cx.flag("-DGGML_USE_CLBLAST");
    cxx.flag("-DGGML_USE_CLBLAST");

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=OpenCL");
        println!("cargo:rustc-link-lib=clblast");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=OpenCL");
        println!("cargo:rustc-link-lib=clblast");
    }

    cxx.file("./llama.cpp/ggml-opencl.cpp");
}

fn compile_openblas(cx: &mut Build) {
    cx.flag("-DGGML_USE_OPENBLAS")
        .include("/usr/local/include/openblas")
        .include("/usr/local/include/openblas");
    println!("cargo:rustc-link-lib=openblas");
}

fn compile_blis(cx: &mut Build) {
    cx.flag("-DGGML_USE_OPENBLAS")
        .include("/usr/local/include/blis")
        .include("/usr/local/include/blis");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=blis");
}

fn compile_cuda(cxx_flags: &str) {
    println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
    println!("cargo:rustc-link-search=native=/opt/cuda/lib64");

    if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
        println!(
            "cargo:rustc-link-search=native={}/targets/x86_64-linux/lib",
            cuda_path
        );
    }

    let libs = "cublas culibos cudart cublasLt pthread dl rt";

    for lib in libs.split_whitespace() {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let mut nvcc = cc::Build::new();

    let env_flags = vec![
        ("LLAMA_CUDA_DMMV_X=32", "-DGGML_CUDA_DMMV_X"),
        ("LLAMA_CUDA_DMMV_Y=1", "-DGGML_CUDA_DMMV_Y"),
        ("LLAMA_CUDA_KQUANTS_ITER=2", "-DK_QUANTS_PER_ITERATION"),
    ];

    let nvcc_flags = "--forward-unknown-to-host-compiler -arch=native ";

    for nvcc_flag in nvcc_flags.split_whitespace() {
        nvcc.flag(nvcc_flag);
    }

    for cxx_flag in cxx_flags.split_whitespace() {
        nvcc.flag(cxx_flag);
    }

    for env_flag in env_flags {
        let mut flag_split = env_flag.0.split('=');
        if let Ok(val) = std::env::var(flag_split.next().unwrap()) {
            nvcc.flag(&format!("{}={}", env_flag.1, val));
        } else {
            nvcc.flag(&format!("{}={}", env_flag.1, flag_split.next().unwrap()));
        }
    }

    nvcc.compiler("nvcc")
        .file("./llama.cpp/ggml-cuda.cu")
        .flag("-Wno-pedantic")
        .include("./llama.cpp/ggml-cuda.h")
        .compile("ggml-cuda");
}

fn compile_ggml(cx: &mut Build, cx_flags: &str) {
    for cx_flag in cx_flags.split_whitespace() {
        cx.flag(cx_flag);
    }

    cx.include("./llama.cpp")
        .file("./llama.cpp/ggml.c")
        .file("./llama.cpp/ggml-alloc.c")
        .file("./llama.cpp/ggml-backend.c")
        .file("./llama.cpp/ggml-quants.c")
        .cpp(false)
        .define("_GNU_SOURCE", None)
        .define("GGML_USE_K_QUANTS", None)
        .compile("ggml");
}

fn compile_metal(cx: &mut Build, cxx: &mut Build) {
    cx.flag("-DGGML_USE_METAL").flag("-DGGML_METAL_NDEBUG");
    cxx.flag("-DGGML_USE_METAL");

    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    println!("cargo:rustc-link-lib=framework=MetalKit");

    cx.include("./llama.cpp/ggml-metal.h")
        .file("./llama.cpp/ggml-metal.m");
}

fn compile_llama(cxx: &mut Build, cxx_flags: &str, out_path: &PathBuf, ggml_type: &str) {
    for cxx_flag in cxx_flags.split_whitespace() {
        cxx.flag(cxx_flag);
    }

    let ggml_obj = PathBuf::from(&out_path).join("llama.cpp/ggml.o");

    cxx.object(ggml_obj);

    if !ggml_type.is_empty() {
        let ggml_feature_obj =
            PathBuf::from(&out_path).join(format!("llama.cpp/ggml-{}.o", ggml_type));
        cxx.object(ggml_feature_obj);
    }

    cxx.shared_flag(true)
        .file("./llama.cpp/common/common.cpp")
        .file("./llama.cpp/llama.cpp")
        .file("./binding.cpp")
        .cpp(true)
        .compile("binding");
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("No out dir found"));

    compile_bindings(&out_path);

    let mut cx_flags = String::from("");
    let mut cxx_flags = String::from("");

    // check if os is linux
    // if so, add -fPIC to cxx_flags
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        cx_flags.push_str(" -std=c11 -Wall -Wextra -Wpedantic -Wcast-qual -Wdouble-promotion -Wshadow -Wstrict-prototypes -Wpointer-arith -pthread -march=native -mtune=native");
        cxx_flags.push_str(" -std=c++11 -Wall -Wdeprecated-declarations -Wunused-but-set-variable -Wextra -Wpedantic -Wcast-qual -Wno-unused-function -Wno-multichar -fPIC -pthread -march=native -mtune=native");
    } else if cfg!(target_os = "windows") {
        cx_flags.push_str(" /W4 /Wall /wd4820 /wd4710 /wd4711 /wd4820 /wd4514");
        cxx_flags.push_str(" /W4 /Wall /wd4820 /wd4710 /wd4711 /wd4820 /wd4514");
    }

    let mut cx = cc::Build::new();

    let mut cxx = cc::Build::new();

    let mut ggml_type = String::new();

    cxx.include("./llama.cpp/common")
        .include("./llama.cpp")
        .include("./include_shims");

    if cfg!(feature = "opencl") {
        compile_opencl(&mut cx, &mut cxx);
        ggml_type = "opencl".to_string();
    } else if cfg!(feature = "openblas") {
        compile_openblas(&mut cx);
    } else if cfg!(feature = "blis") {
        compile_blis(&mut cx);
    } else if cfg!(feature = "metal") && cfg!(target_os = "macos") {
        compile_metal(&mut cx, &mut cxx);
        ggml_type = "metal".to_string();
    }

    if cfg!(feature = "cuda") {
        cx_flags.push_str(" -DGGML_USE_CUBLAS");
        cxx_flags.push_str(" -DGGML_USE_CUBLAS");

        cx.include("/usr/local/cuda/include")
            .include("/opt/cuda/include");
        cxx.include("/usr/local/cuda/include")
            .include("/opt/cuda/include");

        if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
            cx.include(format!("{}/targets/x86_64-linux/include", cuda_path));
            cxx.include(format!("{}/targets/x86_64-linux/include", cuda_path));
        }

        compile_ggml(&mut cx, &cx_flags);

        compile_cuda(&cxx_flags);

        compile_llama(&mut cxx, &cxx_flags, &out_path, "cuda");
    } else {
        compile_ggml(&mut cx, &cx_flags);

        compile_llama(&mut cxx, &cxx_flags, &out_path, &ggml_type);
    }
}
