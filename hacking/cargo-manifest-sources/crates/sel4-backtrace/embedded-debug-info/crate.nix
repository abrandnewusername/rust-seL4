{ mk, localCrates, versions }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info";
  dependencies = {
    addr2line = { version = versions.addr2line; default-features = false; };
    object = { version = versions.object; default-features = false; features = [ "read" ]; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-addr2line-context-helper
  ];
  nix.meta.requirements = [ "sel4" ];
}
