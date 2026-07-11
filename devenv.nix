{ config, pkgs, ... }:

{
  languages = {
    # https://devenv.sh/languages/rust/
    rust = {
      enable = true;
      toolchainFile = ./rust-toolchain.toml;
    }; # ..languages.rust

    # https://devenv.sh/languages/nix/
    nix = {
      enable = true;
      lsp = {
        enable = true;
      }; # ..languages.nix.lsp
    }; # ..languages.nix
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

      extensions = exts: with exts; [ pg_hint_plan ];

      initialDatabases = [ { name = "demo"; } ];
    }; # ..services.postgres

    # https://devenv.sh/services/keycloak/
    keycloak = {
      enable = true;
    }; # ..services.keycloak

    # https://devenv.sh/services/prometheus/
    prometheus = {
      enable = true;
      port = 39090;

      scrapeConfigs = [
        {
          job_name = "api";
          static_configs = [ { targets = [ config.env."API_LISTEN_SOCKET" ]; } ];
        }
        {
          job_name = "web";
          static_configs = [ { targets = [ config.env."WEB_LISTEN_SOCKET" ]; } ];
        }
        {
          job_name = "postgres";
          static_configs = [ { targets = [ config.env."PG_EXPORTER_LISTEN_SOCKET" ]; } ];
        }
      ];
    }; # ..services.prometheus
  }; # ..services

  # https://devenv.sh/processes/
  processes = {
    "static_web_server" = {
      exec = "SERVER_PORT=$WEB_LISTEN_PORT static-web-server --config-file $WEB_CONFIG_FILE";
    };
    "postgres_exporter" = {
      exec = "DATA_SOURCE_URI=$DATABASE_URL postgres_exporter --web.listen-address=$PG_EXPORTER_LISTEN_SOCKET";
    };
  }; # ..processes

  packages = with pkgs; [
    pkg-config
    openssl
    libpq

    # Tools
    diesel-cli
    bacon
    just

    # LSP
    vscode-langservers-extracted

    # Web server
    static-web-server

    # Monitoring
    prometheus-postgres-exporter

    # Nix
    nixfmt

    # Rust (./rust-toolchain.toml)
  ]; # ..packages

  env = {
    "DATABASE_URL" =
      "postgresql:///${config.env."PGDATABASE"}?host=${config.env."PGHOST"}&port=${toString config.services.postgres.port}";

    # The static web server
    "WEB_LISTEN_PORT" = "3000";
    "WEB_LISTEN_SOCKET" = "0.0.0.0:${config.env."WEB_LISTEN_PORT"}";
    "WEB_CONFIG_FILE" = ".config/web/server.toml";
    # The API server
    "API_LISTEN_SOCKET" = "0.0.0.0:3001";
    # Prometheus
    "PROMETHEUS_LISTEN_SOCKET" = "0.0.0.0:${toString config.services.prometheus.port}";
    # PostgreSQL exporter
    "PG_EXPORTER_LISTEN_SOCKET" = "0.0.0.0:39187";

    # psql/libpq defaults — so `psql` connects to the project database.
    # PGHOST and PGPORT are set automatically by devenv's postgres service.
    "PGDATABASE" = "demo";
  }; # ..env

  dotenv = {
    enable = true;
  }; # ..dotenv

  difftastic.enable = true;

  git-hooks = {
    # https://devenv.sh/reference/options/#git-hookshooks
    hooks = {
      check-symlinks.enable = true;
      ripsecrets.enable = true;

      treefmt = {
        enable = true;
        # Configuration: ./.treefmt.toml
      }; # ..git-hooks.hooks.treefmt

      clippy = {
        enable = true;
        settings = {
          allFeatures = true;
          denyWarnings = true;
        };
      }; # ..git-hooks.hooks.clippy
    }; # ..git-hooks.hooks
  }; # ..git-hooks

  enterShell = ''
    echo "🦀 ref-server dev shell"
    echo "   WEB_LISTEN_SOCKET=$WEB_LISTEN_SOCKET"
    echo "   API_LISTEN_SOCKET=$API_LISTEN_SOCKET"
    echo "   DATABASE_URL=$DATABASE_URL"
    echo "   PROMETHEUS_LISTEN_SOCKET=$PROMETHEUS_LISTEN_SOCKET"
    echo "   PG_EXPORTER_LISTEN_SOCKET=$PG_EXPORTER_LISTEN_SOCKET"
    echo ""
    echo "Run: just up  # start the services"
    echo "     psql     # connect to the demo database"
    echo "     just run # start the server"
  '';

  enterTest = ''
    # NOTE: This spins up services+processes prior to running
    just ci
  '';
}
