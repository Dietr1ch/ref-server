{ config, pkgs, ... }:

{
  languages = {
    # https://devenv.sh/languages/rust/
    rust = {
      enable = true;
      toolchainFile = ./rust-toolchain.toml;
    }; # ..languages.rust
  }; # ..languages

  services = {
    # https://devenv.sh/services/postgres/
    postgres = {
      enable = true;

      # Pin PostgreSQL version to avoid surprises when nixpkgs bumps the default
      package = pkgs.postgresql_18_jit;

      # Listen on Unix socket only (default) — diesel connects via $PGHOST
      listen_addresses = "";

      # Port number is used to generate the socket name, make it deterministic.
      # The system might be running Postgres and make this pick :5432 or :5433 on some systems
      port = 35432;

      initialDatabases = [
        {
          name = "demo";
        }
      ];
    }; # ..services.postgres
  }; # ..services

  packages = with pkgs; [
    pkg-config
    openssl
    libpq

    # Tools
    diesel-cli
    bacon
    just

    # Nix
    nixpkgs-fmt

    # Rust (./rust-toolchain.toml)
  ]; # ..packages

  env = {
    "DATABASE_URL" = "postgresql://${config.env."PGDATABASE"}?host=${config.env."PGHOST"}";
    "LISTEN_SOCKET" = "0.0.0.0:3000";

    # psql/libpq defaults — so `psql` connects to the project database.
    # PGHOST and PGPORT are set automatically by devenv's postgres service.
    "PGDATABASE" = "demo";
  }; # ..env

  enterShell = ''
    echo "🦀 ref-server dev shell"
    echo "   LISTEN_SOCKET=$LISTEN_SOCKET"
    echo ""
    echo "Run: devenv processes up  # start the services"
    echo "     psql                 # connect to the demo database"
    echo "     just run             # start the server"
  '';
}
