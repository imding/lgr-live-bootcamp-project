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
          ];

          shellHook = ''
            if [ -f .config/helix ]; then
                cp -r .config/helix ~/.config/helix
            fi

            if [ -f .config/jujutsu.toml ]; then
                mkdir -p ~/.config/jj
                cp .config/jujutsu.toml ~/.config/jj/config.toml
            fi
          '';
        };
      });
    };
}
