use super::github_release::{
    ArchiveFormat, GithubReleaseSpec, ReleaseAssetSpec, ReleaseSelector, ReleaseTarget,
};

macro_rules! asset {
    ($target:ident, $name:expr, $format:ident, $binary:expr) => {
        ReleaseAssetSpec {
            target: ReleaseTarget::$target,
            asset_name: $name,
            archive_format: ArchiveFormat::$format,
            binary_path: $binary,
            sha256: None,
        }
    };
}

const fn mib(value: u64) -> u64 {
    value * 1024 * 1024
}

const LARGE_LLVM_WARNING: &str =
    "Large LLVM release archive; allow several GB of download and extraction space.";

pub const RUST_ANALYZER_VERSION: &str = "2026-05-04";
const RUST_ANALYZER_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "rust-analyzer-x86_64-pc-windows-msvc.zip",
        Zip,
        "rust-analyzer.exe"
    ),
    asset!(
        WindowsArm64,
        "rust-analyzer-aarch64-pc-windows-msvc.zip",
        Zip,
        "rust-analyzer.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "rust-analyzer-x86_64-unknown-linux-gnu.gz",
        GzipBinary,
        "rust-analyzer"
    ),
    asset!(
        LinuxArm64Gnu,
        "rust-analyzer-aarch64-unknown-linux-gnu.gz",
        GzipBinary,
        "rust-analyzer"
    ),
    asset!(
        MacosX64,
        "rust-analyzer-x86_64-apple-darwin.gz",
        GzipBinary,
        "rust-analyzer"
    ),
    asset!(
        MacosArm64,
        "rust-analyzer-aarch64-apple-darwin.gz",
        GzipBinary,
        "rust-analyzer"
    ),
];
pub const RUST_ANALYZER_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "rust-lang",
    repo: "rust-analyzer",
    selector: ReleaseSelector::Tag(RUST_ANALYZER_VERSION),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(128),
    install_warning: None,
    assets: RUST_ANALYZER_ASSETS,
};

pub const LLVM_CLANGD_VERSION: &str = "22.1.5";
const LLVM_CLANGD_TAG: &str = "llvmorg-22.1.5";
const LLVM_CLANGD_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "clang+llvm-22.1.5-x86_64-pc-windows-msvc.tar.xz",
        TarXz,
        "clang+llvm-22.1.5-x86_64-pc-windows-msvc/bin/clangd.exe"
    ),
    asset!(
        WindowsArm64,
        "clang+llvm-22.1.5-aarch64-pc-windows-msvc.tar.xz",
        TarXz,
        "clang+llvm-22.1.5-aarch64-pc-windows-msvc/bin/clangd.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "LLVM-22.1.5-Linux-X64.tar.xz",
        TarXz,
        "LLVM-22.1.5-Linux-X64/bin/clangd"
    ),
    asset!(
        LinuxArm64Gnu,
        "LLVM-22.1.5-Linux-ARM64.tar.xz",
        TarXz,
        "LLVM-22.1.5-Linux-ARM64/bin/clangd"
    ),
    asset!(
        MacosArm64,
        "LLVM-22.1.5-macOS-ARM64.tar.xz",
        TarXz,
        "LLVM-22.1.5-macOS-ARM64/bin/clangd"
    ),
];
pub const LLVM_CLANGD_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "llvm",
    repo: "llvm-project",
    selector: ReleaseSelector::Tag(LLVM_CLANGD_TAG),
    max_size_bytes: mib(2300),
    max_extract_size_bytes: mib(10_240),
    install_warning: Some(LARGE_LLVM_WARNING),
    assets: LLVM_CLANGD_ASSETS,
};

pub const LUA_LS_VERSION: &str = "3.18.2";
const LUA_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "lua-language-server-3.18.2-win32-x64.zip",
        Zip,
        "bin/lua-language-server.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "lua-language-server-3.18.2-linux-x64.tar.gz",
        TarGz,
        "bin/lua-language-server"
    ),
    asset!(
        LinuxArm64Gnu,
        "lua-language-server-3.18.2-linux-arm64.tar.gz",
        TarGz,
        "bin/lua-language-server"
    ),
    asset!(
        MacosX64,
        "lua-language-server-3.18.2-darwin-x64.tar.gz",
        TarGz,
        "bin/lua-language-server"
    ),
    asset!(
        MacosArm64,
        "lua-language-server-3.18.2-darwin-arm64.tar.gz",
        TarGz,
        "bin/lua-language-server"
    ),
];
pub const LUA_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "LuaLS",
    repo: "lua-language-server",
    selector: ReleaseSelector::Tag(LUA_LS_VERSION),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(128),
    install_warning: None,
    assets: LUA_LS_ASSETS,
};

pub const CLOJURE_LSP_VERSION: &str = "2026.05.05-12.58.26";
const CLOJURE_LSP_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "clojure-lsp-native-windows-amd64.zip",
        Zip,
        "clojure-lsp.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "clojure-lsp-native-linux-amd64.zip",
        Zip,
        "clojure-lsp"
    ),
    asset!(
        LinuxArm64Gnu,
        "clojure-lsp-native-linux-aarch64.zip",
        Zip,
        "clojure-lsp"
    ),
    asset!(
        MacosX64,
        "clojure-lsp-native-macos-amd64.zip",
        Zip,
        "clojure-lsp"
    ),
    asset!(
        MacosArm64,
        "clojure-lsp-native-macos-aarch64.zip",
        Zip,
        "clojure-lsp"
    ),
];
pub const CLOJURE_LSP_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "clojure-lsp",
    repo: "clojure-lsp",
    selector: ReleaseSelector::Tag(CLOJURE_LSP_VERSION),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(192),
    install_warning: None,
    assets: CLOJURE_LSP_ASSETS,
};

pub const DENO_VERSION: &str = "2.7.14";
const DENO_TAG: &str = "v2.7.14";
const DENO_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "deno-x86_64-pc-windows-msvc.zip",
        Zip,
        "deno.exe"
    ),
    asset!(
        WindowsArm64,
        "deno-aarch64-pc-windows-msvc.zip",
        Zip,
        "deno.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "deno-x86_64-unknown-linux-gnu.zip",
        Zip,
        "deno"
    ),
    asset!(
        LinuxArm64Gnu,
        "deno-aarch64-unknown-linux-gnu.zip",
        Zip,
        "deno"
    ),
    asset!(MacosX64, "deno-x86_64-apple-darwin.zip", Zip, "deno"),
    asset!(MacosArm64, "deno-aarch64-apple-darwin.zip", Zip, "deno"),
];
pub const DENO_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "denoland",
    repo: "deno",
    selector: ReleaseSelector::Tag(DENO_TAG),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(192),
    install_warning: None,
    assets: DENO_ASSETS,
};

pub const EXPERT_VERSION: &str = "0.1.4";
const EXPERT_TAG: &str = "v0.1.4";
const EXPERT_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "expert_windows_amd64.exe",
        RawBinary,
        "expert.exe"
    ),
    asset!(LinuxX64Gnu, "expert_linux_amd64", RawBinary, "expert"),
    asset!(LinuxArm64Gnu, "expert_linux_arm64", RawBinary, "expert"),
    asset!(MacosX64, "expert_darwin_amd64", RawBinary, "expert"),
    asset!(MacosArm64, "expert_darwin_arm64", RawBinary, "expert"),
];
pub const EXPERT_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "elixir-lang",
    repo: "expert",
    selector: ReleaseSelector::Tag(EXPERT_TAG),
    max_size_bytes: mib(80),
    max_extract_size_bytes: mib(80),
    install_warning: None,
    assets: EXPERT_ASSETS,
};

pub const NIMLANGSERVER_VERSION: &str = "1.14.0";
const NIMLANGSERVER_TAG: &str = "v1.14.0";
const NIMLANGSERVER_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "nimlangserver-windows-amd64.zip",
        Zip,
        "nimlangserver.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "nimlangserver-linux-amd64.tar.gz",
        TarGz,
        "nimlangserver"
    ),
    asset!(
        LinuxArm64Gnu,
        "nimlangserver-linux-arm64.tar.gz",
        TarGz,
        "nimlangserver"
    ),
    asset!(
        MacosX64,
        "nimlangserver-macos-amd64.zip",
        Zip,
        "nimlangserver"
    ),
    asset!(
        MacosArm64,
        "nimlangserver-macos-arm64.zip",
        Zip,
        "nimlangserver"
    ),
];
pub const NIMLANGSERVER_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "nim-lang",
    repo: "langserver",
    selector: ReleaseSelector::Tag(NIMLANGSERVER_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: NIMLANGSERVER_ASSETS,
};

pub const PERL_NAVIGATOR_VERSION: &str = "0.8.20";
const PERL_NAVIGATOR_TAG: &str = "v0.8.20";
const PERL_NAVIGATOR_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "perlnavigator-win-x86_64.zip",
        Zip,
        "perlnavigator-win-x86_64/perlnavigator.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "perlnavigator-linux-x86_64.zip",
        Zip,
        "perlnavigator-linux-x86_64/perlnavigator"
    ),
    asset!(
        MacosX64,
        "perlnavigator-macos-x86_64.zip",
        Zip,
        "perlnavigator-macos-x86_64/perlnavigator"
    ),
];
pub const PERL_NAVIGATOR_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "bscan",
    repo: "PerlNavigator",
    selector: ReleaseSelector::Tag(PERL_NAVIGATOR_TAG),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(128),
    install_warning: None,
    assets: PERL_NAVIGATOR_ASSETS,
};

pub const V_ANALYZER_VERSION: &str = "0.0.6";
const V_ANALYZER_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "v-analyzer-windows-x86_64.zip",
        Zip,
        "v-analyzer.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "v-analyzer-linux-x86_64.zip",
        Zip,
        "v-analyzer"
    ),
    asset!(MacosX64, "v-analyzer-darwin-x86_64.zip", Zip, "v-analyzer"),
    asset!(MacosArm64, "v-analyzer-darwin-arm64.zip", Zip, "v-analyzer"),
];
pub const V_ANALYZER_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "vlang",
    repo: "v-analyzer",
    selector: ReleaseSelector::Tag(V_ANALYZER_VERSION),
    max_size_bytes: mib(16),
    max_extract_size_bytes: mib(32),
    install_warning: None,
    assets: V_ANALYZER_ASSETS,
};

pub const SERVE_D_VERSION: &str = "0.7.6";
const SERVE_D_TAG: &str = "v0.7.6";
const SERVE_D_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "serve-d_0.7.6-windows-x86_64.zip",
        Zip,
        "serve-d.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "serve-d_0.7.6-linux-x86_64.tar.gz",
        TarGz,
        "serve-d"
    ),
    asset!(
        MacosX64,
        "serve-d_0.7.6-osx-x86_64.tar.gz",
        TarGz,
        "serve-d"
    ),
];
pub const SERVE_D_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "Pure-D",
    repo: "serve-d",
    selector: ReleaseSelector::Tag(SERVE_D_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: SERVE_D_ASSETS,
};

pub const GLEAM_VERSION: &str = "1.16.0";
const GLEAM_TAG: &str = "v1.16.0";
const GLEAM_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "gleam-v1.16.0-x86_64-pc-windows-msvc.zip",
        Zip,
        "gleam.exe"
    ),
    asset!(
        WindowsArm64,
        "gleam-v1.16.0-aarch64-pc-windows-msvc.zip",
        Zip,
        "gleam.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "gleam-v1.16.0-x86_64-unknown-linux-musl.tar.gz",
        TarGz,
        "gleam"
    ),
    asset!(
        LinuxArm64Gnu,
        "gleam-v1.16.0-aarch64-unknown-linux-musl.tar.gz",
        TarGz,
        "gleam"
    ),
    asset!(
        MacosX64,
        "gleam-v1.16.0-x86_64-apple-darwin.tar.gz",
        TarGz,
        "gleam"
    ),
    asset!(
        MacosArm64,
        "gleam-v1.16.0-aarch64-apple-darwin.tar.gz",
        TarGz,
        "gleam"
    ),
];
pub const GLEAM_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "gleam-lang",
    repo: "gleam",
    selector: ReleaseSelector::Tag(GLEAM_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: GLEAM_ASSETS,
};

pub const ZLS_VERSION: &str = "0.16.0";
const ZLS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "zls-x86_64-windows.zip", Zip, "zls.exe"),
    asset!(WindowsArm64, "zls-aarch64-windows.zip", Zip, "zls.exe"),
    asset!(LinuxX64Gnu, "zls-x86_64-linux.tar.xz", TarXz, "zls"),
    asset!(LinuxArm64Gnu, "zls-aarch64-linux.tar.xz", TarXz, "zls"),
    asset!(MacosX64, "zls-x86_64-macos.tar.xz", TarXz, "zls"),
    asset!(MacosArm64, "zls-aarch64-macos.tar.xz", TarXz, "zls"),
];
pub const ZLS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "zigtools",
    repo: "zls",
    selector: ReleaseSelector::Tag(ZLS_VERSION),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: ZLS_ASSETS,
};

pub const MILLET_VERSION: &str = "0.15.1";
const MILLET_TAG: &str = "v0.15.1";
const MILLET_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "millet-ls-x86_64-pc-windows-msvc.gz",
        GzipBinary,
        "millet-ls.exe"
    ),
    asset!(
        WindowsArm64,
        "millet-ls-aarch64-pc-windows-msvc.gz",
        GzipBinary,
        "millet-ls.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "millet-ls-x86_64-unknown-linux-gnu.gz",
        GzipBinary,
        "millet-ls"
    ),
    asset!(
        LinuxArm64Gnu,
        "millet-ls-aarch64-unknown-linux-gnu.gz",
        GzipBinary,
        "millet-ls"
    ),
    asset!(
        MacosX64,
        "millet-ls-x86_64-apple-darwin.gz",
        GzipBinary,
        "millet-ls"
    ),
    asset!(
        MacosArm64,
        "millet-ls-aarch64-apple-darwin.gz",
        GzipBinary,
        "millet-ls"
    ),
];
pub const MILLET_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "azdavis",
    repo: "millet",
    selector: ReleaseSelector::Tag(MILLET_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: MILLET_ASSETS,
};

pub const TAPLO_VERSION: &str = "0.10.0";
const TAPLO_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "taplo-windows-x86_64.zip", Zip, "taplo.exe"),
    asset!(WindowsArm64, "taplo-windows-aarch64.zip", Zip, "taplo.exe"),
    asset!(LinuxX64Gnu, "taplo-linux-x86_64.gz", GzipBinary, "taplo"),
    asset!(LinuxArm64Gnu, "taplo-linux-aarch64.gz", GzipBinary, "taplo"),
    asset!(MacosX64, "taplo-darwin-x86_64.gz", GzipBinary, "taplo"),
    asset!(MacosArm64, "taplo-darwin-aarch64.gz", GzipBinary, "taplo"),
];
pub const TAPLO_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "tamasfe",
    repo: "taplo",
    selector: ReleaseSelector::Tag(TAPLO_VERSION),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: TAPLO_ASSETS,
};

pub const DOCKER_LS_VERSION: &str = "0.20.1";
const DOCKER_LS_TAG: &str = "v0.20.1";
const DOCKER_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "docker-language-server-windows-amd64-v0.20.1.exe",
        RawBinary,
        "docker-language-server.exe"
    ),
    asset!(
        WindowsArm64,
        "docker-language-server-windows-arm64-v0.20.1.exe",
        RawBinary,
        "docker-language-server.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "docker-language-server-linux-amd64-v0.20.1",
        RawBinary,
        "docker-language-server"
    ),
    asset!(
        LinuxArm64Gnu,
        "docker-language-server-linux-arm64-v0.20.1",
        RawBinary,
        "docker-language-server"
    ),
    asset!(
        MacosX64,
        "docker-language-server-darwin-amd64-v0.20.1",
        RawBinary,
        "docker-language-server"
    ),
    asset!(
        MacosArm64,
        "docker-language-server-darwin-arm64-v0.20.1",
        RawBinary,
        "docker-language-server"
    ),
];
pub const DOCKER_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "docker",
    repo: "docker-language-server",
    selector: ReleaseSelector::Tag(DOCKER_LS_TAG),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(96),
    install_warning: None,
    assets: DOCKER_LS_ASSETS,
};

pub const CUE_VERSION: &str = "0.16.1";
const CUE_TAG: &str = "v0.16.1";
const CUE_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "cue_v0.16.1_windows_amd64.zip", Zip, "cue.exe"),
    asset!(
        WindowsArm64,
        "cue_v0.16.1_windows_arm64.zip",
        Zip,
        "cue.exe"
    ),
    asset!(LinuxX64Gnu, "cue_v0.16.1_linux_amd64.tar.gz", TarGz, "cue"),
    asset!(
        LinuxArm64Gnu,
        "cue_v0.16.1_linux_arm64.tar.gz",
        TarGz,
        "cue"
    ),
    asset!(MacosX64, "cue_v0.16.1_darwin_amd64.tar.gz", TarGz, "cue"),
    asset!(MacosArm64, "cue_v0.16.1_darwin_arm64.tar.gz", TarGz, "cue"),
];
pub const CUE_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "cue-lang",
    repo: "cue",
    selector: ReleaseSelector::Tag(CUE_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(96),
    install_warning: None,
    assets: CUE_ASSETS,
};

pub const KCL_VERSION: &str = "0.11.2";
const KCL_TAG: &str = "v0.11.2";
const KCL_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "kclvm-v0.11.2-windows.zip",
        Zip,
        "bin/kcl-language-server.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "kclvm-v0.11.2-linux-amd64.tar.gz",
        TarGz,
        "bin/kcl-language-server"
    ),
    asset!(
        LinuxArm64Gnu,
        "kclvm-v0.11.2-linux-arm64.tar.gz",
        TarGz,
        "bin/kcl-language-server"
    ),
    asset!(
        MacosX64,
        "kclvm-v0.11.2-darwin-amd64.tar.gz",
        TarGz,
        "bin/kcl-language-server"
    ),
    asset!(
        MacosArm64,
        "kclvm-v0.11.2-darwin-arm64.tar.gz",
        TarGz,
        "bin/kcl-language-server"
    ),
];
pub const KCL_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "kcl-lang",
    repo: "kcl",
    selector: ReleaseSelector::Tag(KCL_TAG),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(192),
    install_warning: None,
    assets: KCL_ASSETS,
};

pub const HELM_LS_VERSION: &str = "0.5.4";
const HELM_LS_TAG: &str = "v0.5.4";
const HELM_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "helm_ls_windows_amd64.exe",
        RawBinary,
        "helm_ls.exe"
    ),
    asset!(LinuxX64Gnu, "helm_ls_linux_amd64", RawBinary, "helm_ls"),
    asset!(LinuxArm64Gnu, "helm_ls_linux_arm64", RawBinary, "helm_ls"),
    asset!(MacosX64, "helm_ls_darwin_amd64", RawBinary, "helm_ls"),
    asset!(MacosArm64, "helm_ls_darwin_arm64", RawBinary, "helm_ls"),
];
pub const HELM_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "mrjosh",
    repo: "helm-ls",
    selector: ReleaseSelector::Tag(HELM_LS_TAG),
    max_size_bytes: mib(160),
    max_extract_size_bytes: mib(160),
    install_warning: None,
    assets: HELM_LS_ASSETS,
};

pub const NEOCMAKELSP_VERSION: &str = "0.10.2";
const NEOCMAKELSP_TAG: &str = "v0.10.2";
const NEOCMAKELSP_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "neocmakelsp-x86_64-pc-windows-msvc.zip",
        Zip,
        "neocmakelsp.exe"
    ),
    asset!(
        WindowsArm64,
        "neocmakelsp-aarch64-pc-windows-msvc.zip",
        Zip,
        "neocmakelsp.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "neocmakelsp-x86_64-unknown-linux-gnu.tar.gz",
        TarGz,
        "neocmakelsp"
    ),
    asset!(
        LinuxArm64Gnu,
        "neocmakelsp-aarch64-unknown-linux-gnu.tar.gz",
        TarGz,
        "neocmakelsp"
    ),
    asset!(
        MacosX64,
        "neocmakelsp-universal-apple-darwin.tar.gz",
        TarGz,
        "neocmakelsp"
    ),
    asset!(
        MacosArm64,
        "neocmakelsp-universal-apple-darwin.tar.gz",
        TarGz,
        "neocmakelsp"
    ),
];
pub const NEOCMAKELSP_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "Decodetalkers",
    repo: "neocmakelsp",
    selector: ReleaseSelector::Tag(NEOCMAKELSP_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: NEOCMAKELSP_ASSETS,
};

pub const MESONLSP_VERSION: &str = "5.0.2";
const MESONLSP_TAG: &str = "v5.0.2";
const MESONLSP_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "mesonlsp-x86_64-pc-windows-gnu.zip",
        Zip,
        "mesonlsp.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "mesonlsp-x86_64-unknown-linux-musl.zip",
        Zip,
        "mesonlsp"
    ),
    asset!(
        LinuxArm64Gnu,
        "mesonlsp-aarch64-unknown-linux-musl.zip",
        Zip,
        "mesonlsp"
    ),
    asset!(
        MacosX64,
        "mesonlsp-x86_64-apple-darwin.zip",
        Zip,
        "mesonlsp"
    ),
    asset!(
        MacosArm64,
        "mesonlsp-aarch64-apple-darwin.zip",
        Zip,
        "mesonlsp"
    ),
];
pub const MESONLSP_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "JCWasmx86",
    repo: "mesonlsp",
    selector: ReleaseSelector::Tag(MESONLSP_TAG),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(192),
    install_warning: None,
    assets: MESONLSP_ASSETS,
};

pub const JUST_LSP_VERSION: &str = "0.4.4";
const JUST_LSP_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "just-lsp-0.4.4-x86_64-pc-windows-msvc.zip",
        Zip,
        "just-lsp.exe"
    ),
    asset!(
        WindowsArm64,
        "just-lsp-0.4.4-aarch64-pc-windows-msvc.zip",
        Zip,
        "just-lsp.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "just-lsp-0.4.4-x86_64-unknown-linux-gnu.tar.gz",
        TarGz,
        "just-lsp"
    ),
    asset!(
        LinuxArm64Gnu,
        "just-lsp-0.4.4-aarch64-unknown-linux-gnu.tar.gz",
        TarGz,
        "just-lsp"
    ),
    asset!(
        MacosX64,
        "just-lsp-0.4.4-x86_64-apple-darwin.tar.gz",
        TarGz,
        "just-lsp"
    ),
    asset!(
        MacosArm64,
        "just-lsp-0.4.4-aarch64-apple-darwin.tar.gz",
        TarGz,
        "just-lsp"
    ),
];
pub const JUST_LSP_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "terror",
    repo: "just-lsp",
    selector: ReleaseSelector::Tag(JUST_LSP_VERSION),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: JUST_LSP_ASSETS,
};

pub const BUF_VERSION: &str = "1.69.0";
const BUF_TAG: &str = "v1.69.0";
const BUF_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "buf-Windows-x86_64.zip", Zip, "buf/bin/buf.exe"),
    asset!(
        WindowsArm64,
        "buf-Windows-arm64.zip",
        Zip,
        "buf/bin/buf.exe"
    ),
    asset!(LinuxX64Gnu, "buf-Linux-x86_64.tar.gz", TarGz, "buf/bin/buf"),
    asset!(
        LinuxArm64Gnu,
        "buf-Linux-aarch64.tar.gz",
        TarGz,
        "buf/bin/buf"
    ),
    asset!(MacosX64, "buf-Darwin-x86_64.tar.gz", TarGz, "buf/bin/buf"),
    asset!(MacosArm64, "buf-Darwin-arm64.tar.gz", TarGz, "buf/bin/buf"),
];
pub const BUF_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "bufbuild",
    repo: "buf",
    selector: ReleaseSelector::Tag(BUF_TAG),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(256),
    install_warning: None,
    assets: BUF_ASSETS,
};

pub const POSTGRES_LS_VERSION: &str = "0.24.0";
const POSTGRES_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "postgrestools_x86_64-pc-windows-msvc.exe",
        RawBinary,
        "postgrestools.exe"
    ),
    asset!(
        WindowsArm64,
        "postgrestools_aarch64-pc-windows-msvc.exe",
        RawBinary,
        "postgrestools.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "postgrestools_x86_64-unknown-linux-gnu",
        RawBinary,
        "postgrestools"
    ),
    asset!(
        LinuxArm64Gnu,
        "postgrestools_aarch64-unknown-linux-gnu",
        RawBinary,
        "postgrestools"
    ),
    asset!(
        MacosX64,
        "postgrestools_x86_64-apple-darwin",
        RawBinary,
        "postgrestools"
    ),
    asset!(
        MacosArm64,
        "postgrestools_aarch64-apple-darwin",
        RawBinary,
        "postgrestools"
    ),
];
pub const POSTGRES_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "supabase-community",
    repo: "postgres-language-server",
    selector: ReleaseSelector::Tag(POSTGRES_LS_VERSION),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: POSTGRES_LS_ASSETS,
};

pub const GLSL_ANALYZER_VERSION: &str = "1.7.1";
const GLSL_ANALYZER_TAG: &str = "v1.7.1";
const GLSL_ANALYZER_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "x86_64-windows.zip",
        Zip,
        "bin/glsl_analyzer.exe"
    ),
    asset!(
        WindowsArm64,
        "aarch64-windows.zip",
        Zip,
        "bin/glsl_analyzer.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "x86_64-linux-musl.zip",
        Zip,
        "bin/glsl_analyzer"
    ),
    asset!(
        LinuxArm64Gnu,
        "aarch64-linux-musl.zip",
        Zip,
        "bin/glsl_analyzer"
    ),
    asset!(MacosX64, "x86_64-macos.zip", Zip, "bin/glsl_analyzer"),
    asset!(MacosArm64, "aarch64-macos.zip", Zip, "bin/glsl_analyzer"),
];
pub const GLSL_ANALYZER_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "nolanderc",
    repo: "glsl_analyzer",
    selector: ReleaseSelector::Tag(GLSL_ANALYZER_TAG),
    max_size_bytes: mib(16),
    max_extract_size_bytes: mib(32),
    install_warning: None,
    assets: GLSL_ANALYZER_ASSETS,
};

pub const WGSL_ANALYZER_VERSION: &str = "2026-04-26";
const WGSL_ANALYZER_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "wgsl-analyzer-x86_64-pc-windows-msvc.zip",
        Zip,
        "wgsl-analyzer.exe"
    ),
    asset!(
        WindowsArm64,
        "wgsl-analyzer-aarch64-pc-windows-msvc.zip",
        Zip,
        "wgsl-analyzer.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "wgsl-analyzer-x86_64-unknown-linux-gnu.gz",
        GzipBinary,
        "wgsl-analyzer"
    ),
    asset!(
        LinuxArm64Gnu,
        "wgsl-analyzer-aarch64-unknown-linux-gnu.gz",
        GzipBinary,
        "wgsl-analyzer"
    ),
    asset!(
        MacosArm64,
        "wgsl-analyzer-aarch64-apple-darwin.gz",
        GzipBinary,
        "wgsl-analyzer"
    ),
];
pub const WGSL_ANALYZER_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "wgsl-analyzer",
    repo: "wgsl-analyzer",
    selector: ReleaseSelector::Tag(WGSL_ANALYZER_VERSION),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: WGSL_ANALYZER_ASSETS,
};

pub const OPENCL_LS_VERSION: &str = "0.6.3";
const OPENCL_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "opencl-language-server-win32-x86_64.zip",
        Zip,
        "opencl-language-server.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "opencl-language-server-linux-x86_64.tar.gz",
        TarGz,
        "opencl-language-server"
    ),
    asset!(
        LinuxArm64Gnu,
        "opencl-language-server-linux-arm64.tar.gz",
        TarGz,
        "opencl-language-server"
    ),
    asset!(
        MacosX64,
        "opencl-language-server-darwin-x86_64.tar.gz",
        TarGz,
        "opencl-language-server"
    ),
    asset!(
        MacosArm64,
        "opencl-language-server-darwin-arm64.tar.gz",
        TarGz,
        "opencl-language-server"
    ),
];
pub const OPENCL_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "Galarius",
    repo: "opencl-language-server",
    selector: ReleaseSelector::Tag(OPENCL_LS_VERSION),
    max_size_bytes: mib(16),
    max_extract_size_bytes: mib(32),
    install_warning: None,
    assets: OPENCL_LS_ASSETS,
};

pub const VERIBLE_VERSION: &str = "v0.0-4053-g89d4d98a";
const VERIBLE_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "verible-v0.0-4053-g89d4d98a-win64.zip",
        Zip,
        "verible-v0.0-4053-g89d4d98a-win64/verible-verilog-ls.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "verible-v0.0-4053-g89d4d98a-linux-static-x86_64.tar.gz",
        TarGz,
        "verible-v0.0-4053-g89d4d98a/bin/verible-verilog-ls"
    ),
    asset!(
        LinuxArm64Gnu,
        "verible-v0.0-4053-g89d4d98a-linux-static-arm64.tar.gz",
        TarGz,
        "verible-v0.0-4053-g89d4d98a/bin/verible-verilog-ls"
    ),
    asset!(
        MacosX64,
        "verible-v0.0-4053-g89d4d98a-macOS.tar.gz",
        TarGz,
        "verible-v0.0-4053-g89d4d98a/bin/verible-verilog-ls"
    ),
    asset!(
        MacosArm64,
        "verible-v0.0-4053-g89d4d98a-macOS.tar.gz",
        TarGz,
        "verible-v0.0-4053-g89d4d98a/bin/verible-verilog-ls"
    ),
];
pub const VERIBLE_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "chipsalliance",
    repo: "verible",
    selector: ReleaseSelector::Tag(VERIBLE_VERSION),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(192),
    install_warning: None,
    assets: VERIBLE_ASSETS,
};

pub const VHDL_LS_VERSION: &str = "0.86.0";
const VHDL_LS_TAG: &str = "v0.86.0";
const VHDL_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "vhdl_ls-x86_64-pc-windows-msvc.zip",
        Zip,
        "vhdl_ls-x86_64-pc-windows-msvc/bin/vhdl_ls.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "vhdl_ls-x86_64-unknown-linux-gnu.zip",
        Zip,
        "vhdl_ls-x86_64-unknown-linux-gnu/bin/vhdl_ls"
    ),
    asset!(
        MacosArm64,
        "vhdl_ls-aarch64-apple-darwin.zip",
        Zip,
        "vhdl_ls-aarch64-apple-darwin/bin/vhdl_ls"
    ),
];
pub const VHDL_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "VHDL-LS",
    repo: "rust_hdl",
    selector: ReleaseSelector::Tag(VHDL_LS_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(128),
    install_warning: None,
    assets: VHDL_LS_ASSETS,
};

pub const VERYL_LS_VERSION: &str = "0.20.0";
const VERYL_LS_TAG: &str = "v0.20.0";
const VERYL_LS_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "veryl-x86_64-windows.zip", Zip, "veryl-ls.exe"),
    asset!(
        WindowsArm64,
        "veryl-aarch64-windows.zip",
        Zip,
        "veryl-ls.exe"
    ),
    asset!(LinuxX64Gnu, "veryl-x86_64-linux.zip", Zip, "veryl-ls"),
    asset!(LinuxArm64Gnu, "veryl-aarch64-linux.zip", Zip, "veryl-ls"),
    asset!(MacosX64, "veryl-x86_64-mac.zip", Zip, "veryl-ls"),
    asset!(MacosArm64, "veryl-aarch64-mac.zip", Zip, "veryl-ls"),
];
pub const VERYL_LS_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "veryl-lang",
    repo: "veryl",
    selector: ReleaseSelector::Tag(VERYL_LS_TAG),
    max_size_bytes: mib(64),
    max_extract_size_bytes: mib(160),
    install_warning: None,
    assets: VERYL_LS_ASSETS,
};

pub const MARKSMAN_VERSION: &str = "2026-02-08";
const MARKSMAN_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "marksman.exe", RawBinary, "marksman.exe"),
    asset!(LinuxX64Gnu, "marksman-linux-x64", RawBinary, "marksman"),
    asset!(LinuxArm64Gnu, "marksman-linux-arm64", RawBinary, "marksman"),
    asset!(MacosX64, "marksman-macos", RawBinary, "marksman"),
    asset!(MacosArm64, "marksman-macos", RawBinary, "marksman"),
];
pub const MARKSMAN_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "artempyanykh",
    repo: "marksman",
    selector: ReleaseSelector::Tag(MARKSMAN_VERSION),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(96),
    install_warning: None,
    assets: MARKSMAN_ASSETS,
};

pub const TEXLAB_VERSION: &str = "5.25.1";
const TEXLAB_TAG: &str = "v5.25.1";
const TEXLAB_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(WindowsX64, "texlab-x86_64-windows.zip", Zip, "texlab.exe"),
    asset!(
        WindowsArm64,
        "texlab-aarch64-windows.zip",
        Zip,
        "texlab.exe"
    ),
    asset!(LinuxX64Gnu, "texlab-x86_64-linux.tar.gz", TarGz, "texlab"),
    asset!(
        LinuxArm64Gnu,
        "texlab-aarch64-linux.tar.gz",
        TarGz,
        "texlab"
    ),
    asset!(MacosX64, "texlab-x86_64-macos.tar.gz", TarGz, "texlab"),
    asset!(MacosArm64, "texlab-aarch64-macos.tar.gz", TarGz, "texlab"),
];
pub const TEXLAB_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "latex-lsp",
    repo: "texlab",
    selector: ReleaseSelector::Tag(TEXLAB_TAG),
    max_size_bytes: mib(32),
    max_extract_size_bytes: mib(64),
    install_warning: None,
    assets: TEXLAB_ASSETS,
};

pub const TINYMIST_VERSION: &str = "0.14.16";
const TINYMIST_TAG: &str = "v0.14.16";
const TINYMIST_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "tinymist-x86_64-pc-windows-msvc.zip",
        Zip,
        "tinymist.exe"
    ),
    asset!(
        WindowsArm64,
        "tinymist-aarch64-pc-windows-msvc.zip",
        Zip,
        "tinymist.exe"
    ),
    asset!(
        LinuxX64Gnu,
        "tinymist-x86_64-unknown-linux-gnu.tar.gz",
        TarGz,
        "tinymist"
    ),
    asset!(
        LinuxArm64Gnu,
        "tinymist-aarch64-unknown-linux-gnu.tar.gz",
        TarGz,
        "tinymist"
    ),
    asset!(
        MacosX64,
        "tinymist-x86_64-apple-darwin.tar.gz",
        TarGz,
        "tinymist"
    ),
    asset!(
        MacosArm64,
        "tinymist-aarch64-apple-darwin.tar.gz",
        TarGz,
        "tinymist"
    ),
];
pub const TINYMIST_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "Myriad-Dreamin",
    repo: "tinymist",
    selector: ReleaseSelector::Tag(TINYMIST_TAG),
    max_size_bytes: mib(96),
    max_extract_size_bytes: mib(256),
    install_warning: None,
    assets: TINYMIST_ASSETS,
};

pub const REGAL_VERSION: &str = "0.40.0";
const REGAL_TAG: &str = "v0.40.0";
const REGAL_ASSETS: &[ReleaseAssetSpec] = &[
    asset!(
        WindowsX64,
        "regal_Windows_x86_64.exe",
        RawBinary,
        "regal.exe"
    ),
    asset!(LinuxX64Gnu, "regal_Linux_x86_64", RawBinary, "regal"),
    asset!(LinuxArm64Gnu, "regal_Linux_arm64", RawBinary, "regal"),
    asset!(MacosX64, "regal_Darwin_x86_64", RawBinary, "regal"),
    asset!(MacosArm64, "regal_Darwin_arm64", RawBinary, "regal"),
];
pub const REGAL_RELEASE_SPEC: GithubReleaseSpec = GithubReleaseSpec {
    owner: "StyraInc",
    repo: "regal",
    selector: ReleaseSelector::Tag(REGAL_TAG),
    max_size_bytes: mib(80),
    max_extract_size_bytes: mib(80),
    install_warning: None,
    assets: REGAL_ASSETS,
};
