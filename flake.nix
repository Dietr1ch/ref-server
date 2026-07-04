{
  description = "demo-server: axum + diesel + PostgreSQL demo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Diesel CLI for managing migrations
            diesel-cli

            # PostgreSQL client (psql, pg_isready, etc.)
            postgresql

            # Build-time dependencies
            pkg-config
            openssl

            # Project helpers
            just

            # Environment variable loading
            direnv
          ];

          # Environment variables for development
          DATABASE_URL = "postgres://demo:demo@localhost:5432/demo";
          BIND_ADDR = "0.0.0.0:3000";

          shellHook = ''
            echo "🦀 demo-server dev shell"
            echo "   DATABASE_URL=$DATABASE_URL"
            echo "   BIND_ADDR=$BIND_ADDR"
            echo ""
            echo "Run: cargo run          # start the server"
            echo "     diesel setup       # create database & run migrations"
            echo "     diesel migration run  # run pending migrations"
          '';
        };
      }
    );
}
