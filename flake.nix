{
  description = "Deep Time development environment (Rust core + NeoForge 1.21.1 mod)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      perSystem = { config, self', pkgs, lib, system, ... }: let
        # The one Rust version. rust-toolchain.toml (read by rustup in CI) must name the
        # same channel; checks.toolchain-channel fails the flake check when they drift.
        rustChannel = "1.98.1";

        # The eight native targets of plan §6.4. The dev shell carries all their
        # standard libraries so cargo-zigbuild and cargo-xwin can cross-build from here;
        # rust-toolchain.toml lists none, so CI jobs install only what they need.
        rustTargets = [
          "x86_64-unknown-linux-gnu"
          "aarch64-unknown-linux-gnu"
          "x86_64-unknown-linux-musl"
          "aarch64-unknown-linux-musl"
          "x86_64-pc-windows-msvc"
          "aarch64-pc-windows-msvc"
          "x86_64-apple-darwin"
          "aarch64-apple-darwin"
        ];
        rust = pkgs.rust-bin.stable.${rustChannel}.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = rustTargets;
        };

        java = pkgs.jdk21;

        # nixpkgs wraps cargo-zigbuild with `--prefix PATH : <zig>/bin` (currently zig
        # 0.16), so putting zig_0_13 in the shell alone would change nothing for it. The
        # override replaces the wrapped zig with 0.13.0, the version cargo-zigbuild
        # 0.23.4 documents.
        cargoZigbuild = pkgs.cargo-zigbuild.override { zig = pkgs.zig_0_13; };

        # Client runs (gametest-client.sh style) need GL/X on LD_LIBRARY_PATH. Linux only: the
        # shell also runs on Ben's Apple-silicon Mac, where builds and servers are offloaded
        # (tools/remote/mac.sh) and no client is ever started.
        clientLibs = lib.optionals pkgs.stdenv.hostPlatform.isLinux (with pkgs; [
          libGL
          libx11
          libxext
          libxcursor
          libxrandr
          libxi
          libxxf86vm
          libpulseaudio
          openal
          flite
          udev
        ]);

        # The GPU compute path (docs/design/gpu-compute.md) loads Vulkan at run time: the loader on
        # LD_LIBRARY_PATH, and Mesa's lavapipe (a CPU Vulkan driver) added to the drivers it finds,
        # so the GPU tests run on the test VM, which has no GPU. VK_ADD_DRIVER_FILES adds to the
        # system's drivers instead of replacing them, so a real GPU still wins where there is one.
        gpuLibs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.vulkan-loader ];
        lavapipe = "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.${pkgs.stdenv.hostPlatform.parsed.cpu.name}.json";
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rust
            cargoZigbuild
            pkgs.zig_0_13
            pkgs.cargo-xwin
            # cargo-xwin expects clang-cl, lld-link and llvm-lib on PATH and nixpkgs
            # does not propagate them; it downloads only the Windows SDK and CRT itself.
            pkgs.llvmPackages.clang-unwrapped
            pkgs.llvmPackages.lld
            pkgs.llvmPackages.llvm
            pkgs.cargo-deny
            java
            pkgs.gradle
            pkgs.python3
            pkgs.git            # the Mac's /usr/bin/git is a stub without Xcode's tools
            pkgs.mcrcon
            pkgs.unzip
            pkgs.zip
            pkgs.jq
            pkgs.binutils
          ];
          buildInputs = clientLibs ++ gpuLibs;
          env = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            LD_LIBRARY_PATH = lib.makeLibraryPath (clientLibs ++ gpuLibs);
            VK_ADD_DRIVER_FILES = lavapipe;
          } // {
            JAVA_HOME = "${java.home}";
            RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
          };
        };

        # `nix run .#natives`: the eight natives into target/natives-dist/ with natives.json (plan
        # §4.4, P0-08), from the checkout the command runs in. It re-enters this flake's dev shell
        # rather than listing the tools again: the build scripts of the workspace need a host C
        # linker, which on macOS only the shell's stdenv sets up.
        apps.natives = {
          type = "app";
          program = lib.getExe (pkgs.writeShellApplication {
            name = "deeptime-natives";
            text = ''
              exec nix develop '${inputs.self}#default' -c bash ${./tools/natives/build-natives.sh} "$@"
            '';
          });
          meta.description = "Cross-build the eight Deep Time natives into target/natives-dist";
        };

        checks.toolchain-channel = let
          toml = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        in
          assert lib.assertMsg (toml.toolchain.channel == rustChannel)
            "rust-toolchain.toml channel ${toml.toolchain.channel} != flake rustChannel ${rustChannel}";
          pkgs.runCommand "toolchain-channel-ok" { } "touch $out";
      };
    };
}
