# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## Unreleased
### Changed
- (Breaking) redis flag to redis-db.

## 4.0.0-beta.2 (25. July, 2022)
### Added
- Redis via redis-rs. behind feature flag redis.

### Changed
- (Breaking) migrate to initiate.

### Fixed
- Added cfg for sqlx error type. fixes issue where wont build if sqlx not included.

## 4.0.0-beta.1 (24. July, 2022)
### Added
- (Breaking) AxumDatabasePool as a trait by @11Tuvork28 .
- AxumNullPool to support no databases.

### Changed
- (Breaking) Everything is updated to use AxumDatabasePool by @11Tuvork28 .
- Updated Documentation.

## 4.0.0-beta.0 (19. July, 2022)
### Changed
- changed internal data structure to use dashmap and remove need for RWlock and Mutex lock.

## 3.0.2 (26. June, 2022)
### Fixed
- Session destroy not getting set back to false after session was reset for inner

### Changed
- on Session Destroy hashmap is cleared instead of unloaded and remade.

## 3.0.1 (26. June, 2022)
### Fixed
- Session destroy not getting set back to false after session was reset. Good find @bbigras

## 3.0.0 (22. June, 2022)
### Changed
- (Breaking) Made internal structures and functions crate public only.
- (Breaking) Updated Sqlx to 0.6.0.
- Updated Documentation.

## 2.1.0 (14. June, 2022)
### Changed
- Private cookie as Optional for backwards compatibility.

### Added
- Private/Encypted Cookies for confidentiality, integrity, and authenticity. (#17)

## 2.0.1 (18. May, 2022)
### Fixed
- Documentation issues.

## 2.0.0 (18. May, 2022)
### Changed
- Renamed gdpr_mode to session_mode and added a enumeration for 2 session types.
- Default Session storage type is Always.
- Renamed Accepted to Storable.

## 1.2.0 (17. May, 2022)
### Changed
- GDPR Compliance.
- Data Cookie ID only set if disable_gdpr is true or Accepted is true.
- GDPR is Enabled by default so you must use set_accepted on user data for a session to save or disable gdpr_mode.

### Added
- Config for GDPR Sessions.
- GDPR Memory and Database Session unloading on not accepted.
- Accepted GDPR Cookie.
- Better overall documentation with Doc Tests.

## 1.1.0 (12. May, 2022)
### Added
- Long Term Session Switch. Useful for Remember Me.

## 1.0.1 (4. April, 2022)
### Changed
- Removed need for Tower-cookies and implemented cookie handling.

## 1.0.0 (31. March, 2022)
### Changed
- Updated to `Axum` 0.5.

## 0.2.1 (22. Feburary, 2022)
### Changed
- Replaced Axum with Axum core.

## 0.2.0 (22. Feburary, 2022)
### Added
- Capability to not be persistent.

## 0.1.0 (22. Feburary, 2022)
### Added
- Initial release.
