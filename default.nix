{
  config,
  lib,
  pkgs,
  ...
}:

let
  binary = pkgs.rustPlatform.buildRustPackage {
    pname = "ics-proxy";
    version = "0.1.0";

    src = pkgs.lib.cleanSource ./.;
    cargoLock.lockFile = ./Cargo.lock;
  };
  cfg = config.services.ics-proxy;
  configFile = pkgs.writeText "config" cfg.hello-name;
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

      environment = {
        HOST = cfg.host;
        PORT = cfg.port;
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

    environment.etc."nixos-test/config".source = configFile;
    environment.systemPackages = [ binary ];
  };
}
