# Changelog

## [0.3.1](https://github.com/Stonks3141/pet-monitor-app/compare/pet-monitor-app-v0.3.0...pet-monitor-app-v0.3.1) (2023-03-12)


### Features

* Add env var to disable config validation ([a179542](https://github.com/Stonks3141/pet-monitor-app/commit/a17954265d7ee0d1612efbc4462d8f7d5dc74a5b))
* Capture spantraces when errors occur ([674ca95](https://github.com/Stonks3141/pet-monitor-app/commit/674ca958cfb2dde1fa4010e5ee666fbb0ef25e7b))
* Read password from stdin instead of an arg ([cde54af](https://github.com/Stonks3141/pet-monitor-app/commit/cde54af670db1ff4ce9637e0e1c477c5def47983))


### Bug Fixes

* Add rate-limiting for POST /login.html ([75cae10](https://github.com/Stonks3141/pet-monitor-app/commit/75cae106d10764d358b8e91940826322e8f17daa)), closes [#49](https://github.com/Stonks3141/pet-monitor-app/issues/49)
* Fix routing for login and stream pages ([63ea248](https://github.com/Stonks3141/pet-monitor-app/commit/63ea2486aad9dfd2b911c7ebcbc8675324c6f2f5))


### Reverts

* Go back to clap ([aa7c4c5](https://github.com/Stonks3141/pet-monitor-app/commit/aa7c4c5031f3e082684fbf58c753b68eb9a06ffa)), closes [#71](https://github.com/Stonks3141/pet-monitor-app/issues/71)

## [0.3.0](https://github.com/Stonks3141/pet-monitor-app/compare/pet-monitor-app-v0.3.0...pet-monitor-app-v0.3.0) (2023-02-11)


### Features

* Use env var to enable static asset reloading ([f568a60](https://github.com/Stonks3141/pet-monitor-app/commit/f568a60a29c9501c569198572495bdfad9b67f11))


### Bug Fixes

* Fix form decoding of tuples ([4b28b01](https://github.com/Stonks3141/pet-monitor-app/commit/4b28b015863c761be351cd2001b2293f1afabf04)), closes [#55](https://github.com/Stonks3141/pet-monitor-app/issues/55)
* Move dependencies into workspace root ([819b85e](https://github.com/Stonks3141/pet-monitor-app/commit/819b85e879bba57a4cc781ce24b6ce112d4e3ebc))
* **pet-monitor-app:** Redirect after saving config ([9b95dc7](https://github.com/Stonks3141/pet-monitor-app/commit/9b95dc74bd2fb671fbcaf49a9a206f8f1844380c))
* Redirect to login page on auth error ([3512fcb](https://github.com/Stonks3141/pet-monitor-app/commit/3512fcbb2dea3b3e756c46b61c8c11ce5df7eb8f)), closes [#56](https://github.com/Stonks3141/pet-monitor-app/issues/56)
* Rename v4l2Controls to v4l2_controls ([dc555ac](https://github.com/Stonks3141/pet-monitor-app/commit/dc555ac9d06591bed8108b2ecf0f6aba6ea1da7b))
