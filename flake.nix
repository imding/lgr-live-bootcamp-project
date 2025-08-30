{
  description = "Development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        commonPackages = with pkgs; [
          git
          helix
          jujutsu
          nil
        ];

        optionalPackages = {
        };

        getOptionalPackages = let
          enabled = builtins.getEnv "NIX_SHELL_OPTIONS";
          splitOptions = if enabled != "" then builtins.split " " enabled else [];
          options = builtins.filter (x: builtins.isString x && x != "") splitOptions;
        in
          builtins.concatLists (map (opt:
            if builtins.hasAttr opt optionalPackages
            then optionalPackages.${opt}
            else []
          ) options);
      in
      {
        devShells.default = pkgs.mkShellNoCC {
          packages = commonPackages ++ getOptionalPackages;

          shellHook = ''
            if [ -f .config/helix.toml ]; then
              mkdir -p ~/.config/helix
              cp .config/helix.toml ~/.config/helix/config.toml
            fi

            if [ -f .config/jujutsu.toml ]; then
              cp .config/jujutsu.toml ~/.config/jj/config.toml
            fi
          '';
        };
      }
    );
}
