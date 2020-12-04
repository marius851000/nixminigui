{ pkgs, user_config }:

pkgs.factorio.override {
	releaseType = user_config.releaseType;
	username = user_config.username;
	token = user_config.token;
	experimental = user_config.stable == "false";
}
