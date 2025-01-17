{ mk, localCrates, versions  }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info-cli";
  dependencies = {
    inherit (versions) num;
    clap = "3.2.23";
  };
  nix.local.dependencies = with localCrates; [
    sel4-render-elf-with-data
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
