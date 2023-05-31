{ mk, versions }:

mk {
  package.name = "sel4cp-macros";
  lib.proc-macro = true;
  dependencies = {
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (versions) proc-macro2 quote;
  };
  nix.meta.requirements = [ "linux" ];
}