PET-MONITOR-APP(1)

# NAME

pet-monitor-app - a simple and secure pet monitor for Linux.

# SYNOPSIS

*pet-monitor-app* [_OPTIONS_] *set-password* _password_

*pet-monitor-app* [_OPTIONS_] *regen-secret*

*pet-monitor-app* *start* [_OPTIONS_]

*pet-monitor-app* [_OPTIONS_] _-h_|_--help_

*pet-monitor-app* [_OPTIONS_] _-V_|_--version_

# DESCRIPTION

pet-monitor-app is an HTTP server. It captures video from a camera device and
streams it over HTTP to a web client, which can update video settings remotely.
The stream is password-authenticated. This manual documents the command-line
interface used to control the server.

# SUBCOMMANDS

*set-password* _password_
	Change the password.

*regen-secret*
	Regenerate the secret used to sign authentication tokens. This will cause all
	users to be signed out, but will not affect the password.

*start* [_OPTIONS_]
	Start the server.

# OPTIONS

*-h*, *--help*
	Print a help message and exit.

*-V*, *--version*
	Print the current version and exit.

*-c*, *--config* _<path>_
	Path to the config file to use. See *pet-monitor-app*(5) for more information.

# SERVER OPTIONS

These command-line flags are only available on the *start* subcommand. They
override the values set in the configuration file.

*-p*, *--port* _<port>_
	The port for the server to listen on.

*--no-stream*
	Disables video streaming.

# ENVIRONMENT

*RUST_LOG* = _error_|_warn_|_info_|_debug_|_trace_
	The log level for the server.
	
	Default: _warn_

*DISABLE_VALIDATE_CONFIG* = [_1_]
	Whether to disable video config validation. This should not be set unless
	you know what you are doing.

# SEE ALSO

*pet-monitor-app*(5)

# AUTHORS

Developed and maintained by Sam Nystrom <sam@samnystrom.dev>. Source code
is available at https://github.com/Stonks3141/pet-monitor-app.
