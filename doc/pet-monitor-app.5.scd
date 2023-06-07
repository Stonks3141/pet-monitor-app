PET-MONITOR-APP(5)

# NAME

pet-monitor-app - a simple and secure pet monitor for Linux.

# SYNOPSIS

The config file is located at _$XDG\_CONFIG\_HOME/pet-monitor-app/config.toml_.
*XDG\_CONFIG\_HOME* defaults to _~/.config_. A different path can be set by
passing the *--config* command-line option. The file is in the TOML format.

# GENERAL OPTIONS

These options are top-level TOML keys.

*password_hash* = _<hash>_
	The argon2 hash of the password. This option should only be edited using the
	CLI's *set-password* subcommand described in *pet-monitor-app*(1).

*jwt_secret* = _<base64>_
	The 32-byte secret used for signing JSON web tokens. This option should only be
	edited using the CLI's *regen-secret* subcommand described in
	*pet-monitor-app*(1).

*jwt_timeout* = _<seconds>_
	The duration in seconds that JSON web tokens should remain valid for. Users
	will have to reenter the password after this amount of time.
	
	Default: _604800_ (7 days)

*host* = _<ip>_
	The IP address for the server to listen on.
	
	Default: _127.0.0.1_

*port* = _<integer>_
	The port for the server to listen on.
	
	Default: _8080_

# VIDEO OPTIONS

These options are also top-level TOML keys.

*device* = _<path>_
	The V4L2 device to capture video on.
	
	Default: _/dev/video0_

*format* = _<fourcc>_
	The FourCC code of the format to capture video in.
	
	Default: _YUYV_

*resolution* = [_<width>_, _<height>_]
	The resolution in pixels to capture video in.
	
	Default: [_640_, _480_]

*interval* = [_<num>_, _<den>_]
	The frame interval to capture video in, represented as a fraction.
	
	Default: [_1_, _30_]

# SEE ALSO

*pet-monitor-app*(1)

# AUTHORS

Developed and maintained by Sam Nystrom <sam@samnystrom.dev>. Source code is
available at https://github.com/Stonks3141/pet-monitor-app.
