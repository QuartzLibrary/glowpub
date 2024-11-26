{
  pkgs,
  # The parsed rust-toolchain.toml file.
  # (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml))
  rust-toolchain,
}:
{
  buildInputs = [
    pkgs.clang
    pkgs.llvmPackages.bintools
    pkgs.rustup

    pkgs.pkg-config
    pkgs.openssl
  ];

  RUSTC_VERSION = rust-toolchain.toolchain.channel;
  # https://github.com/rust-lang/rust-bindgen#environment-variables
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/

    # Initialize rustup if needed
    if ! command -v rustup &> /dev/null; then
      rustup-init -y --no-modify-path
    fi

    # Ensure the specified toolchain is installed
    rustup toolchain install $RUSTC_VERSION
    rustup default $RUSTC_VERSION
  '';

  # Add precompiled library to rustc search path
  # RUSTFLAGS = (
  #   builtins.map (a: ''-L ${a}/lib'') [
  #     # add libraries here (e.g. pkgs.libvmi)
  #   ]
  # );
  # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
  #   # load external libraries that you need in your rust project here
  #   # pkgs.???
  # ];
  # Add glibc, clang, glib, and other headers to bindgen search path
  BINDGEN_EXTRA_CLANG_ARGS =
    # Includes normal include path
    (builtins.map (a: ''-I"${a}/include"'') [
      # add dev libraries here (e.g. pkgs.libvmi.dev)
      pkgs.glibc.dev
    ])
    # Includes with special directory paths
    ++ [
      ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
      ''-I"${pkgs.glib.dev}/include/glib-2.0"''
      ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
    ];
}
