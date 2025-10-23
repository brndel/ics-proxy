{
  config,
  lib,
  pkgs,
  ...
}:

let
  rust = pkgs.rust-bin.stable."1.89.0".default;
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  binary = rustPlatform.buildRustPackage {
    pname = "ics-proxy";
    version = "0.1.0";

    src = pkgs.lib.cleanSource ./.;
    cargoLock.lockFile = ./Cargo.lock;

    nativeBuildInputs = with pkgs; [ pkg-config ];
    buildInputs = with pkgs; [ openssl ];
  };
  cfg = config.services.ics-proxy;
in
{
  options.services.ics-proxy = {
    enable = lib.mkEnableOption "ics-proxy";
    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
    };
    port = lib.mkOption {
      type = lib.types.port;
      default = 9187;
    };
  };

  config = lib.mkIf cfg.enable {

    users.users.ics-proxy = {
      isSystemUser = true;
      group = "ics-proxy";
      home = "/var/lib/ics-proxy";
    };
    users.groups.ics-proxy = { };

    systemd.services.ics-proxy = {
      description = "Ics Proxy Server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      path = with pkgs; [ openssl ];

      environment = {
        HOST = cfg.host;
        PORT = toString cfg.port;
      };

      serviceConfig = {
        User = "ics-proxy";
        Group = "ics-proxy";
        WorkingDirectory = "/var/lib/ics-proxy";
        ExecStart = "${binary}";

        RestartSec = 5;
        Restart = "always";
      };
      
      restartIfChanged = true;
    };
  };
}
