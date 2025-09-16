{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
          system: function nixpkgs.legacyPackages.${system}
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            git
            helix
            jujutsu
            nil
            postgresql_15
          ];

          shellHook = ''
            if [ -f .config/helix ]; then
                cp -r .config/helix ~/.config/helix
            fi

            if [ -f .config/jujutsu.toml ]; then
                mkdir -p ~/.config/jj
                cp .config/jujutsu.toml ~/.config/jj/config.toml
            fi

            DB_PATH=./auth-service/.local.pg
            DB_LOG_PATH=./auth-service/.local.pg.log
            DB_NAME=local
            DB_USER=rusty

            pg_ctl stop -D $DB_PATH

            if [ -f auth-service/.local.pg ]; then
                echo 'Local Postgres data folder exists, starting...'
                pg_ctl start -D $DB_PATH -l $DB_LOG_PATH
            else
                echo 'Initialising local database...'
                pg_ctl init -D $DB_PATH -o "-E UTF8 -U $DB_USER"

                echo 'Starting local database...'
                pg_ctl start -D $DB_PATH -l $DB_LOG_PATH

                echo 'Creating local database...'
                createdb -h localhost -p 5432 -U $DB_USER $DB_NAME
            fi
          '';
        };
      });
    };
}
