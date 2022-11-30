{
  description = "rust2prod_axum";

  inputs = {
    # to access packages from repo
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # to create a development shell with bells and the wistles!
    devshell.url = "github:numtide/devshell";
    # a helper to build rust packages
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # abstraction tools to help creat a flake
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    devshell,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [devshell.overlay];
      };

      inherit (pkgs) lib stdenv;

      craneLib = crane.lib.${system};
      src = craneLib.cleanCargoSource ./.;

      # If one needs to customize the build environment mostly only needed for
      # macos dependencies or frameworks.
      buildInputs =
        [
          # pkgs.openssl
          # pkgs.pkg-config
        ]
        ++ lib.optionals stdenv.isDarwin (lib.attrVals ["libiconv"] pkgs);

      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src buildInputs;
      };

      # Build the actual crate itself, reusing the dependency
      # artifacts from above.
      rust2pro = craneLib.buildPackage {
        inherit cargoArtifacts src buildInputs;
      };
    in {
      checks =
        {
          # Build the crate as part of `nix flake check` for convenience
          rust2prod = rust2pro;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-crate-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src buildInputs;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          # nixme-doc = craneLib.cargoDoc {
          #   inherit cargoArtifacts src;
          # };

          # Check formatting
          nixme-fmt = craneLib.cargoFmt {
            inherit src buildInputs;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          nixme-nextest = craneLib.cargoNextest {
            inherit cargoArtifacts src buildInputs;
            partitions = 1;
            partitionType = "count";
          };
        }
        // lib.optionalAttrs (system == "x86_64-linux") {
          # NB: cargo-tarpaulin only supports x86_64 systems
          # Check code coverage (note: this will not upload coverage anywhere)
          nixme-coverage = craneLib.cargoTarpaulin {
            inherit cargoArtifacts src buildInputs;
          };
        };

      packages.default = rust2pro;

      apps.default = flake-utils.lib.mkApp {
        drv = rust2pro;
      };

      devShells.default = let
        dbUser = "postgres";
        dbPassword = "password";
        dbPort = "5433";
        dbName = "newsletter";
        dbContainerName = "postgres_rust2prod";
      in let
        databaseUrl = "postgres://${dbUser}:${dbPassword}@localhost:${dbPort}/${dbName}";
      in
        pkgs.devshell.mkShell {
          name = "default development shell";

          env = [
            {
              name = "DB_USER";
              value = dbUser;
            }

            {
              name = "DB_PASSWORD";
              value = dbPassword;
            }

            {
              name = "DB_NAME";
              value = dbName;
            }

            {
              name = "DB_PORT";
              value = dbPort;
            }

            {
              name = "DATABASE_URL";
              value = databaseUrl;
            }

            {
              name = "DB_CONTAINER_NAME";
              value = dbContainerName;
            }

            {
              name = "PGPASSWORD";
              value = dbPassword;
            }
          ];

          # inputsFrom = builtins.attrValues self.checks;
          # Extra inputs can be added here
          packages = with pkgs; [
            # openssl
            # pkg-config
            cargo
            rustc
            cargo
            sqlx-cli
          ];
          commands = [
            {
              name = "pgcreate";
              category = "postgres";
              help = "|> Creates a docker container for postgresql with the name of ${dbContainerName}.";
              command = ''
                docker run \
                 -e POSTGRES_USER=${dbUser} \
                 -e POSTGRES_PASSWORD=${dbPassword} \
                 -e POSTGRES_DB=${dbName} \
                 --name ${dbContainerName} \
                 -p "${dbPort}":5432 \
                 -d postgres \
                 postgres -N 1000
              '';
            }

            {
              name = "pgstart";
              category = "postgres";
              help = "|> Starts and already existing postgresql docker container which has the name of ${dbContainerName}.";
              command = ''
                docker start ${dbContainerName}
              '';
            }

            {
              name = "pgkill";
              help = "|> Kills the postgres container with the name of ${dbContainerName}.";
              category = "postgres";
              command = ''
                docker kill ${dbContainerName}
              '';
            }

            {
              name = "pgcreatedb";
              category = "postgres";
              help = "|> Creates a database";
              command = "${pkgs.sqlx-cli}/bin/sqlx database create";
            }
            {
              name = "pgmigrate";
              category = "postgres";
              command = "${pkgs.sqlx-cli}/bin/sqlx migrate run";
            }
            {
              name = "pg";
              category = "postgres";
              help = "|> Enter psql shell";
              command = "psql -h localhost -p ${dbPort} -U ${dbUser} ${dbName}";
            }
          ];

          #   shellHook =
          #     ''
          #       # this ensures that every executed command is printed on screen
          #       # set -x
          #       # exits on the first fail
          #       # set -eo pipefail
          #     ''
          #     + ''
          #       if [ "$(echo "0")" == "$(echo "$( docker container ls --filter=name=${dbContainerName} -a -q)" | wc -l)" ] ; then
          #         docker run \
          #         -e POSTGRES_USER=${dbUser} \
          #         -e POSTGRES_PASSWORD=${dbPassword} \
          #         -e POSTGRES_DB=${dbName} \
          #         --name ${dbContainerName} \
          #         -p "${dbPort}":5432 \
          #         -d postgres \
          #         postgres -N 1000
          #       else
          #           docker start ${dbContainerName}
          #       fi
          #     ''
          #     + ''
          #       until psql -h "localhost" -U "${dbUser}" -p "${dbPort}" -d "${dbName}" -c "\q"; do
          #       # >&2 echo "${dbName} is still unavailable - sleeping"
          #       sleep 1
          #       done
          #       >&2 echo "Postgres is up and running on port ${dbPort}!"
          #     ''
          #     + "\n"
          #     + " sqlx migrate run";
        };
    });
}
