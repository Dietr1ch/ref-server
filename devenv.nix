{ pkgs, ... }:

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
      package = pkgs.postgresql_16;

      initialDatabases = [
        {
          name = "demo";
        }
      ];
      initialScript = ''
        CREATE USER demo WITH PASSWORD 'demo';
        GRANT ALL ON DATABASE demo TO demo;
      '';
    }; # ..services.postgres
  }; # ..services

  packages = with pkgs; [
    pkg-config
    openssl

    # Tools
    diesel-cli
    just

    # Nix
    nixpkgs-fmt

    # Rust (./rust-toolchain.toml)
  ]; # ..packages

  env = {
    # Application config
    "DATABASE_URL" = "postgres://demo:demo@localhost:5432/demo";
    "LISTEN_SOCKET" = "0.0.0.0:3000";

    # psql/libpq defaults — so `psql` connects to the project database.
    # PGHOST and PGPORT are set automatically by devenv's postgres service.
    "PGUSER" = "demo";
    "PGDATABASE" = "demo";
    "PGPASSWORD" = "demo";
  }; # ..env

  enterShell = ''
    echo "🦀 demo-server dev shell"
    echo "   DATABASE_URL=$DATABASE_URL"
    echo "   LISTEN_SOCKET=$LISTEN_SOCKET"
    echo ""
    echo "Run: just run      # start the server"
    echo "     just setup     # run diesel setup (idempotent)"
    echo "     psql           # connect to the demo database"
  '';
}
