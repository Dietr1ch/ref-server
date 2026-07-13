{ config, pkgs, ... }:

{

  # Profiles
  # ========
  profiles = {

    # Frontend
    # --------
    "frontend".module = { config, ... }: {
      languages = {
        # https://devenv.sh/languages/rust/
        rust = {
          enable = true;
          toolchainFile = ./rust-toolchain.toml;
        }; # ..$frontend.languages.rust
      }; # ..$frontend.languages

      services = {
        # https://devenv.sh/services/prometheus/
        prometheus = {
          scrapeConfigs = [
            {
              job_name = "api";
              static_configs = [ { targets = [ config.env."API_LISTEN_SOCKET" ]; } ];
            }
            {
              job_name = "web";
              static_configs = [ { targets = [ config.env."WEB_LISTEN_SOCKET" ]; } ];
            }
          ];
        }; # ..$frontend.services.prometheus
      }; # ..$frontend.services

      # https://devenv.sh/processes/
      processes = {
        "static_web_server" = {
          exec = "SERVER_PORT=$WEB_LISTEN_PORT static-web-server --config-file $WEB_CONFIG_FILE";
        };
      }; # ..$frontend.processes

      packages = with pkgs; [
        pkg-config
        openssl
        libpq

        # Tools
        diesel-cli

        # LSP
        vscode-langservers-extracted

        # Web server
        static-web-server

        # Rust (./rust-toolchain.toml)
      ]; # ..$frontend.packages

      env = {
        "ENV_FRONTEND" = "enabled";

        # The static web server
        "WEB_LISTEN_PORT" = "3000";
        "WEB_LISTEN_SOCKET" = "0.0.0.0:${config.env."WEB_LISTEN_PORT"}";
        "WEB_CONFIG_FILE" = ".config/web/server.toml";

        # The API server
        "API_LISTEN_SOCKET" = "0.0.0.0:3001";
      }; # ..$frontend.env

      enterShell = ''
        echo "   WEB_LISTEN_SOCKET=$WEB_LISTEN_SOCKET"
        echo "   API_LISTEN_SOCKET=$API_LISTEN_SOCKET"
      '';
    }; # ..$frontend

    # Backend
    # -------
    "backend".module = { config, ... }: {
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
        }; # ..$backend.services.postgres

        # https://devenv.sh/services/prometheus/
        prometheus = {
          scrapeConfigs = [
            {
              job_name = "postgres";
              static_configs = [ { targets = [ config.env."PG_EXPORTER_LISTEN_SOCKET" ]; } ];
            }
          ];
        }; # ..$backend.services.prometheus

      }; # ..$backend.services

      # https://devenv.sh/processes/
      processes = {
        "postgres_exporter" = {
          exec = "DATA_SOURCE_URI=$DATABASE_URL postgres_exporter --web.listen-address=$PG_EXPORTER_LISTEN_SOCKET";
        };
      }; # ..$backend.processes

      packages = with pkgs; [
        # Tools
        diesel-cli

        # Monitoring
        prometheus-postgres-exporter
      ]; # ..$backend.packages

      env = {
        "ENV_BACKEND" = "enabled";

        # Postgres
        "DATABASE_URL" =
          "postgresql:///${config.env."PGDATABASE"}?host=${config.env."PGHOST"}&port=${toString config.services.postgres.port}";
        # PostgreSQL exporter
        "PG_EXPORTER_LISTEN_SOCKET" = "0.0.0.0:39187";
      }; # ..$backend.env

      enterShell = ''
        echo "   DATABASE_URL=$DATABASE_URL"
        echo "   PG_EXPORTER_LISTEN_SOCKET=$PG_EXPORTER_LISTEN_SOCKET"
      '';

    }; # ..$backend

    # Monitoring
    # ----------
    "monitoring".module = { config, ... }: {
      services = {
        # https://devenv.sh/services/prometheus/
        prometheus = {
          enable = true;
          port = 39090;
        }; # ..$monitoring.services.prometheus
      }; # ..$monitoring.services

      env = {
        "ENV_MONITORING" = "enabled";

        # Prometheus
        "PROMETHEUS_LISTEN_SOCKET" = "0.0.0.0:${toString config.services.prometheus.port}";
      }; # ..$monitoring.env

      enterShell = ''
        echo "   PROMETHEUS_LISTEN_SOCKET=$PROMETHEUS_LISTEN_SOCKET"
      '';

    }; # ..$monitoring

    # DataLab
    # -------
    "datalab".module = { ... }: {
      # TODO: Consider extending backend to get data

      languages = {
        # https://devenv.sh/languages/python/
        python = {
          enable = true;

          lsp = {
            package = pkgs.ruff;
          };

          venv.enable = true;
          package = pkgs.python3.withPackages (
            ps: with ps; [
              duckdb
              numpy
              polars
            ]
          );
        }; # ..$datalab.languages.rust
      }; # ..$datalab.languages

      packages = with pkgs; [
        duckdb
        gnuplot
      ]; # ..$datalab.packages

      env = {
        "ENV_DATALAB" = "enabled";
      }; # ..$datalab.env
    }; # ..$datalab

    # CI
    # --
    "ci" = {
      extends = [
        "frontend"
        "backend"
      ];
    }; # ..$ci
  }; # ..profiles

  # Languages
  # =========
  languages = {
    # https://devenv.sh/languages/nix/
    nix = {
      enable = true;
      lsp = {
        enable = true;
      }; # ..languages.nix.lsp
    }; # ..languages.nix
  }; # ..languages

  # Packages
  # ========
  packages = with pkgs; [
    # Tools
    bacon
    just
  ]; # ..packages

  # Environment
  # ===========
  env = {
    "ENV_SHARED" = "enabled";

    # NOTE: PGHOST only defined within the $backend profile...
    # "DATABASE_URL" =
    #   "postgresql:///${config.env."PGDATABASE"}?host=${config.env."PGHOST"}&port=${toString config.services.postgres.port}";

    "PGDATABASE" = "demo";
  }; # ..env

  # Misc
  # ====
  dotenv = {
    enable = true;
  }; # ..dotenv

  difftastic.enable = true;

  git-hooks = {
    # https://devenv.sh/reference/options/#git-hookshooks
    hooks = {
      check-symlinks.enable = true;
      ripsecrets.enable = true;

      # https://devenv.sh/reference/options/#git-hookshookstreefmt
      treefmt = {
        enable = true;

        # Configuration: ./.treefmt.toml
        settings = {
          formatters = with pkgs; [
            # rustfmt already included in (./rust-toolchain.toml)
            nixfmt
            ruff
          ]; # ..git-hooks.hooks.treefmt.settings.formatters
        }; # ..git-hooks.hooks.treefmt.settings

      }; # ..git-hooks.hooks.treefmt

      # https://devenv.sh/reference/options/#git-hookshooksclippy
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
    echo ""
    echo "🦀 ref-server dev shell"
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
