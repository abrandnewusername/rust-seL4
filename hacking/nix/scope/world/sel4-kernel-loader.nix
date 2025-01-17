{ lib, buildPackages, writeText
, buildCrateInLayersHere, buildSysroot, crateUtils
, crates, bareMetalRustTargetInfo
, libclangPath
, seL4RustEnvVars, seL4ForBoot, seL4ForUserspace
, kernelLoaderConfig
}:

let
  rustTargetInfo = bareMetalRustTargetInfo;
  rustTargetName = rustTargetInfo.name;
  rustTargetPath = rustTargetInfo.path;

  rootCrate = crates.sel4-kernel-loader;

  release = true;
  # release = false;

  profile = if release then "release" else "dev";

  profiles = crateUtils.clobber [
    {
      profile.release = {
        lto = true;
      };
    }
    {
      profile.${profile} = {
        # overflow-checks = true; # TODO
        codegen-units = 1;
        incremental = false;
        # debug = 2;
      };
    }
  ];

  sysroot = buildSysroot {
    inherit release rustTargetInfo;
    extraManifest = profiles;
  };

in
buildCrateInLayersHere {

  inherit rootCrate;
  inherit release;

  rustTargetInfo = bareMetalRustTargetInfo;

  features = [];

  commonModifications = {
    modifyManifest = lib.flip lib.recursiveUpdate profiles;
    modifyConfig = lib.flip lib.recursiveUpdate {
      target.${rustTargetName}.rustflags = [
        "--sysroot" sysroot
      ];
    };
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      LIBCLANG_PATH = libclangPath;

      dontStrip = true;
      dontPatchELF = true;
    });
  };

  lastLayerModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars //{
      SEL4_KERNEL_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON kernelLoaderConfig);

      # SEL4_KERNEL = "${seL4ForBoot}/bin/kernel.elf";

      passthru = (super.passthru or {}) // {
        elf = "${self.finalPackage}/bin/${rootCrate.name}";
      };
    });
  };

}
