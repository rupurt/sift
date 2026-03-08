use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_ZVEC_GIT_REF: &str = "v0.2.0";
const ZVEC_REPOSITORY: &str = "https://github.com/alibaba/zvec.git";

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let wrapper_dir = manifest_dir.join("zvec-c-wrapper");
    let zvec_src = resolve_zvec_source(&out_dir);
    let zvec_build = out_dir.join("zvec-build-cmake");
    let wrapper_build = out_dir.join("zvec-c-wrapper-build-cmake");

    emit_rerun_instructions(&manifest_dir, &wrapper_dir);

    let build_type = env::var("ZVEC_BUILD_TYPE").unwrap_or_else(|_| "Release".to_string());
    let parallel_jobs = env::var("ZVEC_BUILD_PARALLEL")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(num_cpus);

    build_zvec(&zvec_src, &zvec_build, &build_type, parallel_jobs);
    build_c_wrapper(
        &wrapper_dir,
        &wrapper_build,
        &zvec_src,
        &build_type,
        parallel_jobs,
    );
    generate_bindings(&wrapper_dir);
    link_libraries(&zvec_build.join("lib"), &wrapper_build);
}

fn emit_rerun_instructions(manifest_dir: &Path, wrapper_dir: &Path) {
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build.rs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("wrapper.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        wrapper_dir.join("CMakeLists.txt").display()
    );

    for source in [
        wrapper_dir.join("include"),
        wrapper_dir.join("src"),
        manifest_dir.join("src"),
    ] {
        println!("cargo:rerun-if-changed={}", source.display());
    }

    for env_var in [
        "BINDGEN_EXTRA_CLANG_ARGS",
        "CC",
        "CFLAGS",
        "CPPFLAGS",
        "CXX",
        "CXXFLAGS",
        "LIBCLANG_PATH",
        "NIX_CFLAGS_COMPILE",
        "TARGET",
        "ZVEC_BUILD_PARALLEL",
        "ZVEC_BUILD_TYPE",
        "ZVEC_CPU_ARCH",
        "ZVEC_GIT_REF",
        "ZVEC_OPENMP",
        "ZVEC_SRC_DIR",
    ] {
        println!("cargo:rerun-if-env-changed={env_var}");
    }
}

fn resolve_zvec_source(out_dir: &Path) -> PathBuf {
    if let Some(source_dir) = env::var_os("ZVEC_SRC_DIR").map(PathBuf::from) {
        if !source_dir.join("CMakeLists.txt").exists() {
            panic!(
                "ZVEC_SRC_DIR does not point to a zvec checkout: {}",
                source_dir.display()
            );
        }
        return source_dir;
    }

    let zvec_ref = env::var("ZVEC_GIT_REF").unwrap_or_else(|_| DEFAULT_ZVEC_GIT_REF.to_string());
    let zvec_src = out_dir.join("zvec-source");

    if !zvec_src.join("CMakeLists.txt").exists() {
        if zvec_src.exists() {
            fs::remove_dir_all(&zvec_src)
                .unwrap_or_else(|err| panic!("failed to clear stale zvec source: {err}"));
        }

        fs::create_dir_all(zvec_src.parent().expect("zvec source parent"))
            .expect("failed to create zvec source parent");

        println!(
            "cargo:warning=Cloning zvec {} into {}",
            zvec_ref,
            zvec_src.display()
        );
        run(
            Command::new("git").args([
                "clone",
                "--depth",
                "1",
                "--branch",
                &zvec_ref,
                "--recursive",
                ZVEC_REPOSITORY,
                zvec_src.to_str().expect("valid zvec source path"),
            ]),
            "git clone zvec",
        );
    } else {
        println!(
            "cargo:warning=Using cached zvec source at {}",
            zvec_src.display()
        );
    }

    zvec_src
}

fn build_zvec(src: &Path, build: &Path, build_type: &str, parallel_jobs: usize) {
    fs::create_dir_all(build).expect("failed to create zvec build directory");

    let mut configure = Command::new("cmake");
    configure.arg("-S").arg(src);
    configure.arg("-B").arg(build);
    configure.arg(format!("-DCMAKE_BUILD_TYPE={build_type}"));
    configure.arg("-DBUILD_PYTHON_BINDINGS=OFF");
    configure.arg("-DBUILD_TOOLS=OFF");
    configure.arg("-DCMAKE_POLICY_VERSION_MINIMUM=3.5");
    apply_cmake_compilers(&mut configure);

    if let Ok(arch) = env::var("ZVEC_CPU_ARCH") {
        configure.arg(format!("-DENABLE_{arch}=ON"));
    }

    if env::var("ZVEC_OPENMP")
        .map(|value| value == "ON" || value == "1")
        .unwrap_or(false)
    {
        configure.arg("-DENABLE_OPENMP=ON");
    }

    run(&mut configure, "cmake configure for zvec");

    let mut build_cmd = Command::new("cmake");
    build_cmd
        .arg("--build")
        .arg(build)
        .arg("--parallel")
        .arg(parallel_jobs.to_string());
    run(&mut build_cmd, "cmake build for zvec");
}

fn build_c_wrapper(
    wrapper_dir: &Path,
    build: &Path,
    zvec_src: &Path,
    build_type: &str,
    parallel_jobs: usize,
) {
    fs::create_dir_all(build).expect("failed to create wrapper build directory");

    let mut configure = Command::new("cmake");
    configure.arg("-S").arg(wrapper_dir);
    configure.arg("-B").arg(build);
    configure.arg(format!("-DZVEC_SRC_DIR={}", zvec_src.display()));
    configure.arg(format!("-DCMAKE_BUILD_TYPE={build_type}"));
    apply_cmake_compilers(&mut configure);
    run(&mut configure, "cmake configure for zvec C wrapper");

    let mut build_cmd = Command::new("cmake");
    build_cmd
        .arg("--build")
        .arg(build)
        .arg("--parallel")
        .arg(parallel_jobs.to_string());
    run(&mut build_cmd, "cmake build for zvec C wrapper");
}

fn apply_cmake_compilers(command: &mut Command) {
    if let Ok(cc) = env::var("CC") {
        command.arg(format!("-DCMAKE_C_COMPILER={cc}"));
    }

    if let Ok(cxx) = env::var("CXX") {
        command.arg(format!("-DCMAKE_CXX_COMPILER={cxx}"));
    }
}

fn generate_bindings(wrapper_dir: &Path) {
    let header = wrapper_dir.join("include/zvec_c.h");
    let out_path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));

    let mut builder = bindgen::Builder::default()
        .header(header.to_str().expect("invalid zvec_c.h path"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_comments(true);

    for arg in bindgen_clang_args() {
        builder = builder.clang_arg(arg);
    }

    let bindings = builder.generate().expect("unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("couldn't write bindings");
}

fn bindgen_clang_args() -> Vec<String> {
    let mut args = Vec::new();
    let mut seen = BTreeSet::new();

    for flag in collect_env_flags("BINDGEN_EXTRA_CLANG_ARGS")
        .into_iter()
        .chain(collect_env_flags("CPPFLAGS"))
        .chain(collect_env_flags("CFLAGS"))
        .chain(collect_env_flags("NIX_CFLAGS_COMPILE"))
        .chain(compiler_include_args())
    {
        if seen.insert(flag.clone()) {
            args.push(flag);
        }
    }

    args
}

fn collect_env_flags(name: &str) -> Vec<String> {
    env::var(name)
        .ok()
        .map(|value| value.split_whitespace().map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

fn compiler_include_args() -> Vec<String> {
    let compiler = env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let output = Command::new(&compiler)
        .args(["-E", "-x", "c", "-", "-v"])
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };

    parse_compiler_search_dirs(&String::from_utf8_lossy(&output.stderr))
        .into_iter()
        .map(|dir| format!("-isystem{dir}"))
        .collect()
}

fn parse_compiler_search_dirs(stderr: &str) -> Vec<String> {
    let mut include_dirs = Vec::new();
    let mut in_search_list = false;

    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed == "#include <...> search starts here:" {
            in_search_list = true;
            continue;
        }

        if trimmed == "End of search list." {
            break;
        }

        if in_search_list && !trimmed.is_empty() {
            include_dirs.push(trimmed.to_string());
        }
    }

    include_dirs
}

fn link_libraries(zvec_lib: &Path, wrapper_build: &Path) {
    println!("cargo:rustc-link-search=native={}", wrapper_build.display());
    println!("cargo:rustc-link-lib=static=zvec_c_wrapper");

    println!("cargo:rustc-link-search=native={}", zvec_lib.display());

    let external_lib = zvec_lib
        .parent()
        .expect("zvec build dir")
        .join("external/usr/local/lib");
    println!("cargo:rustc-link-search=native={}", external_lib.display());

    let arrow_build = zvec_lib
        .parent()
        .expect("zvec build dir")
        .join("thirdparty/arrow/arrow/src/ARROW.BUILD-build");
    println!(
        "cargo:rustc-link-search=native={}",
        arrow_build.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        arrow_build.join("release").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        arrow_build.join("re2_ep-install/lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        arrow_build.join("utf8proc_ep-install/lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        arrow_build
            .join("zlib_ep/src/zlib_ep-install/lib")
            .display()
    );

    let boost_build = arrow_build.join("_deps/boost-build/libs");
    for library in [
        "atomic",
        "charconv",
        "chrono",
        "container",
        "date_time",
        "locale",
        "thread",
    ] {
        println!(
            "cargo:rustc-link-search=native={}",
            boost_build.join(library).display()
        );
    }

    let lz4_build = zvec_lib
        .parent()
        .expect("zvec build dir")
        .join("thirdparty/lz4/lz4/src/Lz4.BUILD/lib");
    println!("cargo:rustc-link-search=native={}", lz4_build.display());

    for library in ["zvec_core", "zvec_ailego", "zvec_db"] {
        println!("cargo:rustc-link-lib=static:+whole-archive={library}");
    }

    for library in [
        "parquet",
        "arrow_acero",
        "arrow_dataset",
        "arrow_compute",
        "arrow",
        "arrow_bundled_dependencies",
        "roaring",
        "rocksdb",
        "lz4",
        "protobuf",
        "protoc",
        "boost_thread",
        "boost_atomic",
        "boost_chrono",
        "boost_container",
        "boost_date_time",
        "boost_locale",
        "boost_charconv",
        "glog",
        "gflags_nothreads",
        "antlr4-runtime",
    ] {
        println!("cargo:rustc-link-lib=static:+whole-archive={library}");
    }

    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");
}

fn run(command: &mut Command, context: &str) {
    println!("cargo:warning=Running: {:?}", command);
    let status = command.status().unwrap_or_else(|_| {
        panic!("failed to execute command: {context}");
    });
    if !status.success() {
        panic!("command failed ({context}): {:?}", command);
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(4)
}
