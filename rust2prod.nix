{pkgs ? import <nixpkgs> {}}:
with pkgs; let
  entrypoint = "./target/release/rust2prod";
in
  dockerTools.buildImage {
    name = "rust2prod";
    runAsRoot = ''
      #!${stdenv.shell}
      ${dockerTools.shadowSetup}
      ${pkgs.cargo}/bin/cargo build --release
    '';
    config = {
      Entrypoint = [entrypoint];
      WorkingDir = "/app";
      ExposedPorts = {
        "5432/tcp" = {};
      };
    };
  }
