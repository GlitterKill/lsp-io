use anyhow::Result;
use ignore::{DirEntry, WalkBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Broad grouping used by the GUI so a large registry stays scannable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageCategory {
    Programming,
    Web,
    Config,
    Data,
    Build,
    Infra,
    Shader,
    Hardware,
    Proof,
    Framework,
    DomainSpecific,
}

impl LanguageCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Programming => "Programming",
            Self::Web => "Web",
            Self::Config => "Config",
            Self::Data => "Data",
            Self::Build => "Build",
            Self::Infra => "Infra",
            Self::Shader => "Shader",
            Self::Hardware => "Hardware",
            Self::Proof => "Proof",
            Self::Framework => "Framework",
            Self::DomainSpecific => "Domain-specific",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Programming => "programming",
            Self::Web => "web",
            Self::Config => "config",
            Self::Data => "data",
            Self::Build => "build",
            Self::Infra => "infra",
            Self::Shader => "shader",
            Self::Hardware => "hardware",
            Self::Proof => "proof",
            Self::Framework => "framework",
            Self::DomainSpecific => "domain-specific",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionConfidence {
    High,
    Medium,
    Low,
}

impl DetectionConfidence {
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

/// A language, framework, or standalone file domain that LSP-IO can detect and
/// map to a recommended language server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageKind {
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Go,
    Java,
    CSharp,
    Ruby,
    Kotlin,
    Cpp,
    Scala,
    Lua,
    Swift,
    Php,
    Ada,
    Bash,
    Clojure,
    Dart,
    Deno,
    Elixir,
    Erlang,
    FSharp,
    Fortran,
    Haskell,
    Julia,
    Nim,
    Ocaml,
    Perl,
    PowerShell,
    R,
    Racket,
    Raku,
    ReScript,
    V,
    Vala,
    Zig,
    Nix,
    Ballerina,
    Chapel,
    Crystal,
    Dlang,
    Elm,
    Gleam,
    Groovy,
    Haxe,
    Idris2,
    Lean4,
    Coq,
    CommonLisp,
    StandardMl,
    Html,
    Css,
    Json,
    Angular,
    Astro,
    Svelte,
    Vue,
    Mdx,
    TailwindCss,
    Emmet,
    GraphQl,
    Yaml,
    Xml,
    Toml,
    Docker,
    Terraform,
    Cue,
    Jsonnet,
    Kcl,
    Bicep,
    Ansible,
    Helm,
    CMake,
    Meson,
    Just,
    Make,
    Nginx,
    Systemd,
    GitHubActions,
    GitLabCi,
    Protobuf,
    Thrift,
    Sql,
    PostgresSql,
    PromQl,
    OpenApi,
    Glsl,
    Wgsl,
    Hlsl,
    Qml,
    OpenCl,
    SystemVerilog,
    Vhdl,
    Veryl,
    Dot,
    Markdown,
    Latex,
    Typst,
    RobotFramework,
    Gherkin,
    Rego,
    Puppet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub kind: LanguageKind,
    pub category: LanguageCategory,
    pub confidence: DetectionConfidence,
    pub evidence: String,
}

#[derive(Debug)]
pub struct LanguageMetadata {
    pub kind: LanguageKind,
    pub display_name: &'static str,
    pub category: LanguageCategory,
    pub manifest_markers: &'static [&'static str],
    pub extension_markers: &'static [&'static str],
    pub directory_markers: &'static [&'static str],
    pub min_extension_evidence: usize,
    pub extension_confidence: DetectionConfidence,
}

impl Language {
    pub const ALL: &[LanguageKind] = &[
        LanguageKind::TypeScript,
        LanguageKind::JavaScript,
        LanguageKind::Python,
        LanguageKind::Rust,
        LanguageKind::Go,
        LanguageKind::Java,
        LanguageKind::CSharp,
        LanguageKind::Ruby,
        LanguageKind::Kotlin,
        LanguageKind::Cpp,
        LanguageKind::Scala,
        LanguageKind::Lua,
        LanguageKind::Swift,
        LanguageKind::Php,
        LanguageKind::Ada,
        LanguageKind::Bash,
        LanguageKind::Clojure,
        LanguageKind::Dart,
        LanguageKind::Deno,
        LanguageKind::Elixir,
        LanguageKind::Erlang,
        LanguageKind::FSharp,
        LanguageKind::Fortran,
        LanguageKind::Haskell,
        LanguageKind::Julia,
        LanguageKind::Nim,
        LanguageKind::Ocaml,
        LanguageKind::Perl,
        LanguageKind::PowerShell,
        LanguageKind::R,
        LanguageKind::Racket,
        LanguageKind::Raku,
        LanguageKind::ReScript,
        LanguageKind::V,
        LanguageKind::Vala,
        LanguageKind::Zig,
        LanguageKind::Nix,
        LanguageKind::Ballerina,
        LanguageKind::Chapel,
        LanguageKind::Crystal,
        LanguageKind::Dlang,
        LanguageKind::Elm,
        LanguageKind::Gleam,
        LanguageKind::Groovy,
        LanguageKind::Haxe,
        LanguageKind::Idris2,
        LanguageKind::Lean4,
        LanguageKind::Coq,
        LanguageKind::CommonLisp,
        LanguageKind::StandardMl,
        LanguageKind::Html,
        LanguageKind::Css,
        LanguageKind::Json,
        LanguageKind::Angular,
        LanguageKind::Astro,
        LanguageKind::Svelte,
        LanguageKind::Vue,
        LanguageKind::Mdx,
        LanguageKind::TailwindCss,
        LanguageKind::Emmet,
        LanguageKind::GraphQl,
        LanguageKind::Yaml,
        LanguageKind::Xml,
        LanguageKind::Toml,
        LanguageKind::Docker,
        LanguageKind::Terraform,
        LanguageKind::Cue,
        LanguageKind::Jsonnet,
        LanguageKind::Kcl,
        LanguageKind::Bicep,
        LanguageKind::Ansible,
        LanguageKind::Helm,
        LanguageKind::CMake,
        LanguageKind::Meson,
        LanguageKind::Just,
        LanguageKind::Make,
        LanguageKind::Nginx,
        LanguageKind::Systemd,
        LanguageKind::GitHubActions,
        LanguageKind::GitLabCi,
        LanguageKind::Protobuf,
        LanguageKind::Thrift,
        LanguageKind::Sql,
        LanguageKind::PostgresSql,
        LanguageKind::PromQl,
        LanguageKind::OpenApi,
        LanguageKind::Glsl,
        LanguageKind::Wgsl,
        LanguageKind::Hlsl,
        LanguageKind::Qml,
        LanguageKind::OpenCl,
        LanguageKind::SystemVerilog,
        LanguageKind::Vhdl,
        LanguageKind::Veryl,
        LanguageKind::Dot,
        LanguageKind::Markdown,
        LanguageKind::Latex,
        LanguageKind::Typst,
        LanguageKind::RobotFramework,
        LanguageKind::Gherkin,
        LanguageKind::Rego,
        LanguageKind::Puppet,
    ];

    pub fn name(&self) -> &'static str {
        self.kind.name()
    }

    pub fn display_name(&self) -> &'static str {
        self.kind.display_name()
    }
}

impl LanguageKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Java => "java",
            Self::CSharp => "csharp",
            Self::Ruby => "ruby",
            Self::Kotlin => "kotlin",
            Self::Cpp => "cpp",
            Self::Scala => "scala",
            Self::Lua => "lua",
            Self::Swift => "swift",
            Self::Php => "php",
            Self::Ada => "ada",
            Self::Bash => "bash",
            Self::Clojure => "clojure",
            Self::Dart => "dart",
            Self::Deno => "deno",
            Self::Elixir => "elixir",
            Self::Erlang => "erlang",
            Self::FSharp => "fsharp",
            Self::Fortran => "fortran",
            Self::Haskell => "haskell",
            Self::Julia => "julia",
            Self::Nim => "nim",
            Self::Ocaml => "ocaml",
            Self::Perl => "perl",
            Self::PowerShell => "powershell",
            Self::R => "r",
            Self::Racket => "racket",
            Self::Raku => "raku",
            Self::ReScript => "rescript",
            Self::V => "v",
            Self::Vala => "vala",
            Self::Zig => "zig",
            Self::Nix => "nix",
            Self::Ballerina => "ballerina",
            Self::Chapel => "chapel",
            Self::Crystal => "crystal",
            Self::Dlang => "d",
            Self::Elm => "elm",
            Self::Gleam => "gleam",
            Self::Groovy => "groovy",
            Self::Haxe => "haxe",
            Self::Idris2 => "idris2",
            Self::Lean4 => "lean4",
            Self::Coq => "coq",
            Self::CommonLisp => "common-lisp",
            Self::StandardMl => "standard-ml",
            Self::Html => "html",
            Self::Css => "css",
            Self::Json => "json",
            Self::Angular => "angular",
            Self::Astro => "astro",
            Self::Svelte => "svelte",
            Self::Vue => "vue",
            Self::Mdx => "mdx",
            Self::TailwindCss => "tailwind-css",
            Self::Emmet => "emmet",
            Self::GraphQl => "graphql",
            Self::Yaml => "yaml",
            Self::Xml => "xml",
            Self::Toml => "toml",
            Self::Docker => "docker",
            Self::Terraform => "terraform",
            Self::Cue => "cue",
            Self::Jsonnet => "jsonnet",
            Self::Kcl => "kcl",
            Self::Bicep => "bicep",
            Self::Ansible => "ansible",
            Self::Helm => "helm",
            Self::CMake => "cmake",
            Self::Meson => "meson",
            Self::Just => "just",
            Self::Make => "make",
            Self::Nginx => "nginx",
            Self::Systemd => "systemd",
            Self::GitHubActions => "github-actions",
            Self::GitLabCi => "gitlab-ci",
            Self::Protobuf => "protobuf",
            Self::Thrift => "thrift",
            Self::Sql => "sql",
            Self::PostgresSql => "postgres-sql",
            Self::PromQl => "promql",
            Self::OpenApi => "openapi",
            Self::Glsl => "glsl",
            Self::Wgsl => "wgsl",
            Self::Hlsl => "hlsl",
            Self::Qml => "qml",
            Self::OpenCl => "opencl",
            Self::SystemVerilog => "systemverilog",
            Self::Vhdl => "vhdl",
            Self::Veryl => "veryl",
            Self::Dot => "dot",
            Self::Markdown => "markdown",
            Self::Latex => "latex",
            Self::Typst => "typst",
            Self::RobotFramework => "robot-framework",
            Self::Gherkin => "gherkin",
            Self::Rego => "rego",
            Self::Puppet => "puppet",
        }
    }

    pub fn display_name(self) -> &'static str {
        self.metadata().display_name
    }

    pub fn category(self) -> LanguageCategory {
        self.metadata().category
    }

    pub fn metadata(self) -> &'static LanguageMetadata {
        LANGUAGE_METADATA
            .iter()
            .find(|metadata| metadata.kind == self)
            .expect("every LanguageKind must have metadata")
    }

    pub fn with_evidence(self, evidence: String, confidence: DetectionConfidence) -> Language {
        Language {
            kind: self,
            category: self.category(),
            confidence,
            evidence,
        }
    }
}

const LANGUAGE_METADATA: &[LanguageMetadata] = &[
    meta(
        LanguageKind::TypeScript,
        "TypeScript",
        LanguageCategory::Programming,
        &["tsconfig.json"],
        &["ts", "tsx"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::JavaScript,
        "JavaScript",
        LanguageCategory::Programming,
        &["package.json"],
        &["js", "jsx", "mjs", "cjs"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Python,
        "Python",
        LanguageCategory::Programming,
        &[
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
            "pipfile",
        ],
        &["py", "pyi"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Rust,
        "Rust",
        LanguageCategory::Programming,
        &["cargo.toml"],
        &["rs"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Go,
        "Go",
        LanguageCategory::Programming,
        &["go.mod"],
        &["go"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Java,
        "Java",
        LanguageCategory::Programming,
        &["pom.xml", "build.gradle"],
        &["java"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::CSharp,
        "C#",
        LanguageCategory::Programming,
        &[".sln"],
        &["cs", "csproj"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Ruby,
        "Ruby",
        LanguageCategory::Programming,
        &["gemfile"],
        &["rb", "rake"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Kotlin,
        "Kotlin",
        LanguageCategory::Programming,
        &["build.gradle.kts", "settings.gradle.kts"],
        &["kt", "kts"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Cpp,
        "C / C++",
        LanguageCategory::Programming,
        &["cmakelists.txt", "compile_commands.json"],
        &["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Scala,
        "Scala",
        LanguageCategory::Programming,
        &["build.sbt"],
        &["scala", "sc"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Lua,
        "Lua",
        LanguageCategory::Programming,
        &[".luarc.json", ".luacheckrc", "selene.toml", "stylua.toml"],
        &["lua"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Swift,
        "Swift",
        LanguageCategory::Programming,
        &["package.swift"],
        &["swift"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Php,
        "PHP",
        LanguageCategory::Programming,
        &["composer.json"],
        &["php"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Ada,
        "Ada / SPARK",
        LanguageCategory::Programming,
        &["alire.toml"],
        &["adb", "ads"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Bash,
        "Bash",
        LanguageCategory::Programming,
        &[".bashrc", ".bash_profile"],
        &["sh", "bash", "bats"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Clojure,
        "Clojure",
        LanguageCategory::Programming,
        &["deps.edn", "project.clj", "shadow-cljs.edn"],
        &["clj", "cljs", "cljc", "edn"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Dart,
        "Dart",
        LanguageCategory::Programming,
        &["pubspec.yaml"],
        &["dart"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Deno,
        "Deno",
        LanguageCategory::Programming,
        &["deno.json", "deno.jsonc"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Elixir,
        "Elixir",
        LanguageCategory::Programming,
        &["mix.exs"],
        &["ex", "exs"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Erlang,
        "Erlang",
        LanguageCategory::Programming,
        &["rebar.config", "erlang.mk"],
        &["erl", "hrl"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::FSharp,
        "F#",
        LanguageCategory::Programming,
        &["paket.dependencies"],
        &["fs", "fsx", "fsi", "fsproj"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Fortran,
        "Fortran",
        LanguageCategory::Programming,
        &[],
        &["f", "for", "f90", "f95", "f03", "f08"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Haskell,
        "Haskell",
        LanguageCategory::Programming,
        &["cabal.project", "stack.yaml", "package.yaml"],
        &["hs", "lhs"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Julia,
        "Julia",
        LanguageCategory::Programming,
        &["project.toml"],
        &["jl"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Nim,
        "Nim",
        LanguageCategory::Programming,
        &[],
        &["nim", "nims"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Ocaml,
        "OCaml / Reason",
        LanguageCategory::Programming,
        &["dune-project"],
        &["ml", "mli", "re", "rei"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Perl,
        "Perl",
        LanguageCategory::Programming,
        &["cpanfile", "makefile.pl"],
        &["pl", "pm", "t"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::PowerShell,
        "PowerShell",
        LanguageCategory::Programming,
        &[],
        &["ps1", "psm1", "psd1"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::R,
        "R",
        LanguageCategory::Programming,
        &["description"],
        &["r", "rmd", "qmd"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Racket,
        "Racket",
        LanguageCategory::Programming,
        &["info.rkt"],
        &["rkt"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Raku,
        "Raku",
        LanguageCategory::Programming,
        &["meta6.json"],
        &["raku", "rakumod", "rakutest"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::ReScript,
        "ReScript",
        LanguageCategory::Programming,
        &["rescript.json", "bsconfig.json"],
        &["res", "resi"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::V,
        "V",
        LanguageCategory::Programming,
        &["v.mod"],
        &["vsh"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Vala,
        "Vala",
        LanguageCategory::Programming,
        &[],
        &["vala", "vapi"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Zig,
        "Zig",
        LanguageCategory::Programming,
        &["build.zig", "build.zig.zon"],
        &["zig", "zon"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Nix,
        "Nix",
        LanguageCategory::Programming,
        &["flake.nix", "default.nix"],
        &["nix"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Ballerina,
        "Ballerina",
        LanguageCategory::Programming,
        &["ballerina.toml"],
        &["bal"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Chapel,
        "Chapel",
        LanguageCategory::Programming,
        &[],
        &["chpl"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Crystal,
        "Crystal",
        LanguageCategory::Programming,
        &["shard.yml"],
        &["cr"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Dlang,
        "D",
        LanguageCategory::Programming,
        &["dub.json", "dub.sdl"],
        &["d", "di"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Elm,
        "Elm",
        LanguageCategory::Programming,
        &["elm.json"],
        &["elm"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Gleam,
        "Gleam",
        LanguageCategory::Programming,
        &["gleam.toml"],
        &["gleam"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Groovy,
        "Groovy",
        LanguageCategory::Programming,
        &[],
        &["groovy", "gradle"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Haxe,
        "Haxe",
        LanguageCategory::Programming,
        &["haxelib.json"],
        &["hx", "hxml"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Idris2,
        "Idris2",
        LanguageCategory::Programming,
        &[],
        &["idr", "lidr"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Lean4,
        "Lean 4",
        LanguageCategory::Proof,
        &["lakefile.lean", "lean-toolchain"],
        &["lean"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Coq,
        "Coq",
        LanguageCategory::Proof,
        &["_coqproject"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::CommonLisp,
        "Common Lisp",
        LanguageCategory::Programming,
        &[],
        &["lisp", "lsp", "asd"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::StandardMl,
        "Standard ML",
        LanguageCategory::Programming,
        &[],
        &["sml", "sig", "fun"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Html,
        "HTML",
        LanguageCategory::Web,
        &["index.html"],
        &["html", "htm"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Css,
        "CSS / Less / Sass",
        LanguageCategory::Web,
        &[],
        &["css", "less", "scss", "sass"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Json,
        "JSON",
        LanguageCategory::Data,
        &[],
        &["json", "jsonc"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Angular,
        "Angular",
        LanguageCategory::Framework,
        &["angular.json"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Astro,
        "Astro",
        LanguageCategory::Framework,
        &["astro.config.mjs", "astro.config.ts"],
        &["astro"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Svelte,
        "Svelte",
        LanguageCategory::Framework,
        &["svelte.config.js", "svelte.config.ts"],
        &["svelte"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Vue,
        "Vue",
        LanguageCategory::Framework,
        &["vue.config.js"],
        &["vue"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Mdx,
        "MDX",
        LanguageCategory::Web,
        &[],
        &["mdx"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::TailwindCss,
        "Tailwind CSS",
        LanguageCategory::Framework,
        &[
            "tailwind.config.js",
            "tailwind.config.ts",
            "tailwind.config.cjs",
            "tailwind.config.mjs",
        ],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Emmet,
        "Emmet",
        LanguageCategory::Web,
        &[],
        &["html", "css", "scss", "sass", "less"],
        &[],
        3,
        DetectionConfidence::Low,
    ),
    meta(
        LanguageKind::GraphQl,
        "GraphQL",
        LanguageCategory::Data,
        &[],
        &["graphql", "gql"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Yaml,
        "YAML",
        LanguageCategory::Data,
        &[".yamllint", ".yamllint.yaml"],
        &["yaml", "yml"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Xml,
        "XML",
        LanguageCategory::Data,
        &[],
        &["xml", "xsd", "xsl", "xslt"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Toml,
        "TOML",
        LanguageCategory::Data,
        &["taplo.toml"],
        &["toml"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Docker,
        "Docker / Compose / Bake",
        LanguageCategory::Infra,
        &[
            "dockerfile",
            "compose.yaml",
            "compose.yml",
            "docker-compose.yaml",
            "docker-compose.yml",
            "docker-bake.hcl",
        ],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Terraform,
        "Terraform",
        LanguageCategory::Infra,
        &[],
        &["tf", "tfvars"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Cue,
        "CUE",
        LanguageCategory::Config,
        &["cue.mod"],
        &["cue"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Jsonnet,
        "Jsonnet",
        LanguageCategory::Config,
        &["jsonnetfile.json"],
        &["jsonnet", "libsonnet"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Kcl,
        "KCL",
        LanguageCategory::Config,
        &["kcl.mod"],
        &["k"],
        &[],
        1,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Bicep,
        "Bicep",
        LanguageCategory::Infra,
        &[],
        &["bicep"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Ansible,
        "Ansible",
        LanguageCategory::Infra,
        &["ansible.cfg"],
        &[],
        &["roles", "playbooks"],
        1,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Helm,
        "Helm",
        LanguageCategory::Infra,
        &["chart.yaml"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::CMake,
        "CMake",
        LanguageCategory::Build,
        &["cmakelists.txt"],
        &["cmake"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Meson,
        "Meson",
        LanguageCategory::Build,
        &["meson.build", "meson_options.txt"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Just,
        "Just",
        LanguageCategory::Build,
        &["justfile", ".justfile"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Make,
        "Make",
        LanguageCategory::Build,
        &["makefile", "gnumakefile"],
        &["mk"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Nginx,
        "Nginx",
        LanguageCategory::Infra,
        &["nginx.conf"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Systemd,
        "systemd",
        LanguageCategory::Infra,
        &[],
        &["service", "timer", "socket", "mount", "target", "path"],
        &[],
        2,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::GitHubActions,
        "GitHub Actions",
        LanguageCategory::Infra,
        &[],
        &[],
        &[".github/workflows"],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::GitLabCi,
        "GitLab CI",
        LanguageCategory::Infra,
        &[".gitlab-ci.yml", ".gitlab-ci.yaml"],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Protobuf,
        "Protocol Buffers",
        LanguageCategory::Data,
        &["buf.yaml", "buf.work.yaml"],
        &["proto"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Thrift,
        "Thrift",
        LanguageCategory::Data,
        &[],
        &["thrift"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Sql,
        "SQL",
        LanguageCategory::Data,
        &[],
        &["sql"],
        &[],
        1,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::PostgresSql,
        "Postgres SQL",
        LanguageCategory::Data,
        &["postgrestools.jsonc"],
        &["pgsql"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::PromQl,
        "PromQL",
        LanguageCategory::DomainSpecific,
        &["prometheus.yml", "prometheus.yaml"],
        &["promql"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::OpenApi,
        "OpenAPI",
        LanguageCategory::Data,
        &[
            "openapi.yaml",
            "openapi.yml",
            "openapi.json",
            "swagger.yaml",
            "swagger.yml",
            "swagger.json",
        ],
        &[],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Glsl,
        "GLSL",
        LanguageCategory::Shader,
        &[],
        &["glsl", "vert", "frag", "geom", "comp"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Wgsl,
        "WGSL",
        LanguageCategory::Shader,
        &[],
        &["wgsl"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Hlsl,
        "HLSL",
        LanguageCategory::Shader,
        &[],
        &["hlsl", "fx", "fxh"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Qml,
        "QML",
        LanguageCategory::Framework,
        &[],
        &["qml"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::OpenCl,
        "OpenCL",
        LanguageCategory::Shader,
        &[],
        &["cl", "opencl"],
        &[],
        1,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::SystemVerilog,
        "SystemVerilog",
        LanguageCategory::Hardware,
        &[],
        &["sv", "svh"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Vhdl,
        "VHDL",
        LanguageCategory::Hardware,
        &[],
        &["vhd", "vhdl"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Veryl,
        "Veryl",
        LanguageCategory::Hardware,
        &["veryl.toml"],
        &["veryl"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Dot,
        "Graphviz DOT",
        LanguageCategory::DomainSpecific,
        &[],
        &["dot", "gv"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Markdown,
        "Markdown",
        LanguageCategory::Data,
        &[".marksman.toml"],
        &["md", "markdown"],
        &[],
        3,
        DetectionConfidence::Medium,
    ),
    meta(
        LanguageKind::Latex,
        "LaTeX",
        LanguageCategory::DomainSpecific,
        &[],
        &["tex", "bib", "sty", "cls"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Typst,
        "Typst",
        LanguageCategory::DomainSpecific,
        &[],
        &["typ"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::RobotFramework,
        "Robot Framework",
        LanguageCategory::DomainSpecific,
        &[],
        &["robot", "resource"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Gherkin,
        "Gherkin / Cucumber",
        LanguageCategory::DomainSpecific,
        &[],
        &["feature"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Rego,
        "Rego",
        LanguageCategory::DomainSpecific,
        &[],
        &["rego"],
        &[],
        1,
        DetectionConfidence::High,
    ),
    meta(
        LanguageKind::Puppet,
        "Puppet",
        LanguageCategory::Infra,
        &["puppetfile"],
        &["pp"],
        &[],
        1,
        DetectionConfidence::High,
    ),
];

const fn meta(
    kind: LanguageKind,
    display_name: &'static str,
    category: LanguageCategory,
    manifest_markers: &'static [&'static str],
    extension_markers: &'static [&'static str],
    directory_markers: &'static [&'static str],
    min_extension_evidence: usize,
    extension_confidence: DetectionConfidence,
) -> LanguageMetadata {
    LanguageMetadata {
        kind,
        display_name,
        category,
        manifest_markers,
        extension_markers,
        directory_markers,
        min_extension_evidence,
        extension_confidence,
    }
}

#[derive(Default)]
struct EvidenceBucket {
    manifests: Vec<String>,
    extensions: Vec<String>,
    directories: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanProfile {
    Recommendation,
    AllEvidence,
}

/// Scan manifests, directory markers, and source extensions. Manifest and
/// directory markers are high signal; extension-only matches must satisfy each
/// language's threshold to avoid one-off snippets dominating large repos.
pub fn scan_languages(root: &Path) -> Result<Vec<Language>> {
    scan_languages_with_profile(root, ScanProfile::Recommendation)
}

pub fn scan_languages_with_profile(root: &Path, profile: ScanProfile) -> Result<Vec<Language>> {
    let mut evidence: HashMap<LanguageKind, EvidenceBucket> = HashMap::new();

    let mut builder = WalkBuilder::new(root);
    builder
        .max_depth(Some(5))
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true);
    let root_for_filter = root.to_path_buf();
    builder.filter_entry(move |entry| should_descend(&root_for_filter, entry, profile));

    for entry in builder.build() {
        let entry = entry?;
        let path = entry.path();
        let Some(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            record_directory_markers(root, path, &mut evidence);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        let display_path = relative_display(root, path);

        for metadata in LANGUAGE_METADATA {
            if metadata
                .manifest_markers
                .iter()
                .any(|marker| marker.eq_ignore_ascii_case(&filename))
                || matches_csproj_or_sln(metadata.kind, &filename)
            {
                evidence
                    .entry(metadata.kind)
                    .or_default()
                    .manifests
                    .push(display_path.clone());
            }

            if let Some(extension) = &extension
                && metadata
                    .extension_markers
                    .iter()
                    .any(|marker| marker.eq_ignore_ascii_case(extension))
            {
                evidence
                    .entry(metadata.kind)
                    .or_default()
                    .extensions
                    .push(display_path.clone());
            }
        }
    }

    let mut languages = Vec::new();
    for kind in Language::ALL {
        let Some(bucket) = evidence.remove(kind) else {
            continue;
        };
        let metadata = kind.metadata();
        let has_strong_marker = !bucket.manifests.is_empty() || !bucket.directories.is_empty();
        let has_enough_extensions = bucket.extensions.len() >= metadata.min_extension_evidence;

        if !has_strong_marker && !has_enough_extensions {
            continue;
        }

        let confidence = if !bucket.manifests.is_empty() {
            DetectionConfidence::High
        } else if !bucket.directories.is_empty() {
            DetectionConfidence::Medium
        } else {
            metadata.extension_confidence
        };

        languages.push(kind.with_evidence(summarize_evidence(bucket), confidence));
    }

    Ok(languages)
}

fn record_directory_markers(
    root: &Path,
    path: &Path,
    evidence: &mut HashMap<LanguageKind, EvidenceBucket>,
) {
    for metadata in LANGUAGE_METADATA {
        if metadata
            .directory_markers
            .iter()
            .any(|marker| matches_directory_marker(root, path, marker))
        {
            evidence
                .entry(metadata.kind)
                .or_default()
                .directories
                .push(relative_display(root, path));
        }
    }
}

fn matches_csproj_or_sln(kind: LanguageKind, filename: &str) -> bool {
    kind == LanguageKind::CSharp && (filename.ends_with(".csproj") || filename.ends_with(".sln"))
}

fn matches_directory_marker(root: &Path, path: &Path, marker: &str) -> bool {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .components()
        .collect::<PathBuf>()
        .display()
        .to_string()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let marker = marker.replace('\\', "/").to_ascii_lowercase();
    relative == marker || relative.ends_with(&format!("/{marker}"))
}

fn summarize_evidence(mut bucket: EvidenceBucket) -> String {
    let mut paths = Vec::new();
    paths.append(&mut bucket.manifests);
    paths.append(&mut bucket.directories);
    paths.append(&mut bucket.extensions);
    paths.sort();
    paths.dedup();

    let total = paths.len();
    if total <= 5 {
        return paths.join(", ");
    }

    let shown = paths.into_iter().take(5).collect::<Vec<_>>().join(", ");
    format!("{shown}, +{} more", total - 5)
}

fn should_descend(root: &Path, entry: &DirEntry, profile: ScanProfile) -> bool {
    let components = normalized_components(root, entry.path());
    if components.is_empty() {
        return true;
    }

    if components.iter().any(|component| {
        matches!(
            component.as_str(),
            ".git"
                | ".hg"
                | ".svn"
                | "node_modules"
                | "target"
                | "vendor"
                | ".venv"
                | "venv"
                | "__pycache__"
                | ".gradle"
                | ".idea"
                | "build"
                | "dist"
                | ".tmp"
                | ".worktrees"
                | ".sisyphus"
                | ".codex"
                | ".claude"
        )
    }) {
        return false;
    }

    if profile == ScanProfile::Recommendation
        && [
            &["tests", "fixtures"][..],
            &["tests", "integration", "fixtures"][..],
            &["tests", "stress", "fixtures"][..],
            &["dist-tests"][..],
            &[".benchmark"][..],
        ]
        .iter()
        .any(|sequence| has_component_sequence(&components, sequence))
    {
        return false;
    }

    true
}

fn has_component_sequence(components: &[String], sequence: &[&str]) -> bool {
    components.windows(sequence.len()).any(|window| {
        window
            .iter()
            .map(String::as_str)
            .eq(sequence.iter().copied())
    })
}

fn normalized_components(root: &Path, path: &Path) -> Vec<String> {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => {
                Some(value.to_string_lossy().to_ascii_lowercase())
            }
            _ => None,
        })
        .collect()
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .collect::<PathBuf>()
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_manifest_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        std::fs::write(dir.path().join("Package.swift"), "").unwrap();
        std::fs::write(dir.path().join(".luarc.json"), "{}").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        let kinds: Vec<_> = found.iter().map(|l| l.kind).collect();

        assert!(kinds.contains(&LanguageKind::Python));
        assert!(kinds.contains(&LanguageKind::Swift));
        assert!(kinds.contains(&LanguageKind::Lua));
    }

    #[test]
    fn skips_dependency_directories() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("node_modules").join("pkg");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("package.json"), "{}").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn scan_respects_gitignore_for_recommendations() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".gitignore"),
            ".tmp/\n.worktrees/\n.sisyphus/\n.codex/\n.claude/\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src").join("index.ts"), "").unwrap();
        std::fs::write(dir.path().join("src").join("server.ts"), "").unwrap();

        let ansible_dir = dir.path().join(".tmp").join("ansible-lint-oss");
        std::fs::create_dir_all(&ansible_dir).unwrap();
        std::fs::write(ansible_dir.join("ansible.cfg"), "").unwrap();
        let html_dir = dir.path().join(".worktrees").join("feature").join("ui");
        std::fs::create_dir_all(&html_dir).unwrap();
        std::fs::write(html_dir.join("index.html"), "").unwrap();
        let rust_dir = dir.path().join(".sisyphus").join("repo").join("rs");
        std::fs::create_dir_all(&rust_dir).unwrap();
        std::fs::write(rust_dir.join("smoke.rs"), "").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        let kinds: Vec<_> = found.iter().map(|language| language.kind).collect();

        assert!(kinds.contains(&LanguageKind::TypeScript));
        assert!(!kinds.contains(&LanguageKind::Ansible));
        assert!(!kinds.contains(&LanguageKind::Html));
        assert!(!kinds.contains(&LanguageKind::Rust));
    }

    #[test]
    fn recommendation_scan_ignores_fixture_only_evidence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src").join("server.ts"), "").unwrap();
        std::fs::write(dir.path().join("src").join("client.ts"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("native").join("src")).unwrap();
        std::fs::write(dir.path().join("native").join("src").join("lib.rs"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("scripts")).unwrap();
        std::fs::write(dir.path().join("scripts").join("install.sh"), "").unwrap();
        std::fs::write(dir.path().join("scripts").join("run.sh"), "").unwrap();

        let fixture = dir
            .path()
            .join("tests")
            .join("integration")
            .join("fixtures")
            .join("example-plugin");
        std::fs::create_dir_all(&fixture).unwrap();
        std::fs::write(fixture.join("symbols.ex"), "").unwrap();
        std::fs::write(fixture.join("symbols.exs"), "").unwrap();
        let stress = dir
            .path()
            .join("tests")
            .join("stress")
            .join("fixtures")
            .join("web");
        std::fs::create_dir_all(&stress).unwrap();
        std::fs::write(stress.join("index.html"), "").unwrap();
        std::fs::write(stress.join("style.css"), "").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        let kinds: Vec<_> = found.iter().map(|language| language.kind).collect();

        assert!(kinds.contains(&LanguageKind::TypeScript));
        assert!(kinds.contains(&LanguageKind::Rust));
        assert!(kinds.contains(&LanguageKind::Bash));
        assert!(!kinds.contains(&LanguageKind::Elixir));
        assert!(!kinds.contains(&LanguageKind::Html));
        assert!(!kinds.contains(&LanguageKind::Css));

        let all_evidence =
            scan_languages_with_profile(dir.path(), ScanProfile::AllEvidence).unwrap();
        let all_kinds: Vec<_> = all_evidence.iter().map(|language| language.kind).collect();

        assert!(all_kinds.contains(&LanguageKind::Elixir));
        assert!(all_kinds.contains(&LanguageKind::Html));
    }

    #[test]
    fn extension_detection_requires_language_threshold() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("one.json"), "{}").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        assert!(
            !found
                .iter()
                .any(|language| language.kind == LanguageKind::Json)
        );

        std::fs::write(dir.path().join("two.json"), "{}").unwrap();
        let found = scan_languages(dir.path()).unwrap();
        assert!(
            found
                .iter()
                .any(|language| language.kind == LanguageKind::Json)
        );
    }

    #[test]
    fn detects_representative_broad_categories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "").unwrap();
        std::fs::write(dir.path().join("index.html"), "").unwrap();
        std::fs::write(dir.path().join("config.toml"), "").unwrap();
        std::fs::write(dir.path().join("main.tf"), "").unwrap();
        std::fs::write(dir.path().join("shader.wgsl"), "").unwrap();
        std::fs::write(dir.path().join("top.vhdl"), "").unwrap();
        std::fs::write(dir.path().join("theorem.lean"), "").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        let categories: Vec<_> = found.iter().map(|language| language.category).collect();

        assert!(categories.contains(&LanguageCategory::Programming));
        assert!(categories.contains(&LanguageCategory::Web));
        assert!(categories.contains(&LanguageCategory::Data));
        assert!(categories.contains(&LanguageCategory::Infra));
        assert!(categories.contains(&LanguageCategory::Shader));
        assert!(categories.contains(&LanguageCategory::Hardware));
        assert!(categories.contains(&LanguageCategory::Proof));
    }

    #[test]
    fn directory_markers_detect_infra_domains() {
        let dir = tempfile::tempdir().unwrap();
        let workflows = dir.path().join(".github").join("workflows");
        std::fs::create_dir_all(&workflows).unwrap();
        std::fs::write(workflows.join("ci.yml"), "").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        assert!(
            found
                .iter()
                .any(|language| language.kind == LanguageKind::GitHubActions)
        );
    }

    #[test]
    fn ambiguous_v_extension_needs_project_marker() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.v"), "").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        assert!(
            !found
                .iter()
                .any(|language| language.kind == LanguageKind::V)
        );
        assert!(
            !found
                .iter()
                .any(|language| language.kind == LanguageKind::Coq)
        );

        std::fs::write(dir.path().join("v.mod"), "").unwrap();
        let found = scan_languages(dir.path()).unwrap();
        assert!(
            found
                .iter()
                .any(|language| language.kind == LanguageKind::V)
        );

        let coq_dir = tempfile::tempdir().unwrap();
        std::fs::write(coq_dir.path().join("_CoqProject"), "").unwrap();
        std::fs::write(coq_dir.path().join("main.v"), "").unwrap();
        let found = scan_languages(coq_dir.path()).unwrap();
        assert!(
            found
                .iter()
                .any(|language| language.kind == LanguageKind::Coq)
        );
    }

    #[test]
    fn single_systemd_like_extension_is_not_enough() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("example.path"), "").unwrap();

        let found = scan_languages(dir.path()).unwrap();
        assert!(
            !found
                .iter()
                .any(|language| language.kind == LanguageKind::Systemd)
        );

        std::fs::write(dir.path().join("example.service"), "").unwrap();
        let found = scan_languages(dir.path()).unwrap();
        assert!(
            found
                .iter()
                .any(|language| language.kind == LanguageKind::Systemd)
        );
    }

    #[test]
    fn all_language_kinds_have_metadata() {
        assert_eq!(Language::ALL.len(), LANGUAGE_METADATA.len());
        for kind in Language::ALL {
            assert_eq!(kind.metadata().kind, *kind);
        }
    }
}
