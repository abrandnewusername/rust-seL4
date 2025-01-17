{ mk, localCrates, versions, virtioDriversWith}:

mk {
  package.name = "microkit-http-server-example-virtio-blk-driver";
  dependencies = rec {
    inherit (versions) log;

    virtio-drivers = virtioDriversWith [];

    sel4-externally-shared.features = [ "unstable" ];
    sel4-microkit = { default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4
    sel4-sync
    sel4-logging
    sel4-immediate-sync-once-cell
    sel4-externally-shared
    sel4-shared-ring-buffer
    sel4-shared-ring-buffer-block-io-types
    sel4-bounce-buffer-allocator

    microkit-http-server-example-virtio-hal-impl
  ];
}
