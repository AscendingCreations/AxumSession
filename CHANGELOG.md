# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
## Unreleased

## 0.15.0 (1. January, 2025)
### Changed
- (Breaking) Axum 0.8.1

## 0.14.4 (10. October, 2024)
### Fixed
-  Docs should now build.

## 0.14.1 (16. July, 2024)
### Changed
- Readme to include discord badges.
- DashMaps to Version 6.0.1.

## 0.14.0 (12. April, 2024)
### Added
- (Breaking) Split Databases into their own libraries Other than the SessionNullPool and SessionAnyPool

## 0.13.0 (11. March, 2024)
### Added
- Options to enable and disable certain ip and user agent patterns.
- (Breaking) Socket IP and user agent are true by default rest is now false.

## 0.12.4 (5. February, 2024)
### Fixed
- missing config extensions

## 0.12.3 (5. February, 2024)
### Fixed
- missing .memory. on some use_bloom_filters internally.

## 0.12.2 (4. February, 2024)
### Added
- Function with_ip_and_user_agent to disable or enable ip and user agent usage within signed uuid's
- Made signed cookies and headers use ip's and user agent for Message signing to help prevent spoofing. Enabled by default.

## 0.12.1 (1. January, 2023)
- Fixed missing MemoryLifetime in Advanced feature.

## 0.12.0 (1. January, 2023)
### Changed
- (Breaking) split config into groups to help make it more understandab
le on the Docs side.
- (Breaking) Removed encryption of cookies and headers and replaced with signing.
- (Breaking) Database_key is now used to encrypt the Session Data within Database when set.
- (Breaking) Removed Cyclable encryption key since we are signing instead. 
- Added Tracing logs.
- Removed a lot of panics and instead sending an Empty response with 500 internal server error status.

## 0.11.0 (21. December, 2023)
### Changed
- (Breaking) updated Redis_Pool to 0.3.0 which updated Redis to 0.24.0.

## 0.10.1 (27. November, 2023)
### Fixed
- Mongo inserting multiple copies due to use of insert instead of update with upsert option enabled. (@MohenjoDaro)

## 0.10.0 (27. November, 2023)
### Changed
- (Breaking) Surreal is now a single feature instead of selecting all the seperate parts.
- (Breaking) Axum to 0.7

## 0.9.0 (13. November, 2023)
### Added
- __HOST- appending to increase Security to cookies on supported browsers. Off by default. This will Append __HOST- to the front of cookie names.
  You must Set the Domain in order to enable this or cookies will not get Set.
- SessionAnyPool Added Thanks too @smessmer

### Changed
- (Breaking) SessionMode::Storable Renamed to SessionMode::OptIn.
- (Breaking) SessionMode::Always Renamed to SessionMode::Persistent.
- (Breaking) Renamed with_storable_name to with_store_name.

### Updated
- cookie from 0.17.0 to 0.18.0

## 0.8.0 (23. October, 2023)
### Changed
- (Breaking) renamed destroy_session to database_remove_session Also made it (crate) level as this needs to be behind the advanced feature.

### Added
- Advanced feature and functions though Session.

## 0.7.0 (4. October, 2023)
### Changed
- Reduced amount of internal clones to half and other Optimizations.
- Cleaned up way Data gets destroyed or deleted.
- (Breaking) `SessionStore::clear` is async now due to filter needing to be arc/rwlocked.
- (Breaking) Renamed `storable_cookie_name` to `storable_name` changing `with_storable_cookie_name` to `with_storable_name`.
- (Breaking) Renamed `cookie_name` to `session_name` changing `with_cookie_name` to `with_session_name`.
- (Breaking) Renamed `key_cookie_name` to `key_name` changing `with_key_cookie_name` to `with_key_name`.
- (Breaking) `get_session_id` is now not async.

### Fixed
- Removal cookies should now contain a SameSite::None to avoid browser warnings.
- key-store not removing keys due to having it require if the database should auto clean or not.
- Session key store not getting updated correctly due to filter not updating across threads as it should be Arc.
- Dead Locking during an await within database saves. thanks to KrisCarr for finding it!

### Added
- `get_store` and `get_mut_store` to Session.
- Rest Mode. Pulls Data from Request Headers and places Data back into Response Headers. No cookies are used.
- `get_session_name`, `get_key_name` and `get_storable_name` to SessionConfig.
- `clear_check_on_load` to config. This will allow you to bypass the Clear check before the Request.
- is_parallel Requests counter to prevent unloading of data till all requests have finished.

## 0.6.1 (22. September, 2023)
### Changed
- Reduced Data sent to persistent database and gathered from persistent database.

## 0.6.0 (18. September, 2023)
### Added
- MongoDB support by @MohenjoDaro

### Changed
- (breaking) redis to redis_pool. https://github.com/AscendingCreations/RedisPool
- Updated Surreal to 1.0.0
- Added Redis ClusterClient support via feature redis_clusterdb.
- (breaking) Memory and Database purge runner no longer uses memory lifetime. it instead uses its own purge_update and purge_database_update times.

### Fixed
- Filter now removes keys if client doesnt Exist for database and keys get cleared.
- Sessions now no longer unload data if they are in memory and not expired but database is expired.

## 0.5.0 (6. September, 2023)
### Changed
- (breaking) SessionStore::initiate() is removed. initiate has been merged into SessionStore::new(). 
### Fixed
- Filter Seeding Errors due to no tables initiated.
- Filter even on else now uses the config to set FilterBuilder.

## 0.4.0 (3. September, 2023)
### Changed
- (Breaking) session.destroy() Deletes Session and cookie on Response phase rather than just sessiondata on Request phase.
- (Breaking) Removed indxdb and fdb 5.1 - 6.0 from surreal due to outdate or incompatibilities.
- (Breaking) SurrealDB updated to 1.0.0-beta.10 Thank you (@Atila-M-Schrieber).

## 0.3.5 (7. August, 2023)
### Fixed
- rename git repo.
- Add ignores to comments so they dont run due to async code errors.

## 0.3.4 (20. July, 2023)
### Fixed
- Removed not needed default features.

## 0.3.3 (17. July, 2023)
### Fixed
- greater than to lesser than in delete_by_expiry for postgresql, sqlite and surrealdb. Thank you (@alexichepura).
- Removed uneeded Clone from ID String Gathering.

## 0.3.2 (17. July, 2023)
### Fixed
- Readme for crates.io and github.

## 0.3.1 (7. July, 2023)
### Added
- Made ReadOnlySession Visible.

## 0.3.0 (7. July, 2023)
### Added
- Key Storage via fastbloom. for optimized key usage comparison.
- key-store feature to tie key storage behind.

### Changed
- (breaking) Surrealdb connection API has been updated to recommends methods.
- (breaking) Updated Sqlx to 0.7.0

## 0.2.3 (15. May, 2023)
### Added 
- Per Session SessionID Encryption.

## 0.2.2 (4. May, 2023)
### Fixed 
- RUSTSEC-2020-0071 from chrono. (damccull)

## 0.2.1 (12. April, 2023)
### Fixed 
- Database Documentation.

## 0.2.0 (11. April, 2023)
### Fixed 
- Surreal Session compile Error.

### Changed
- Made pub(crate) visible in docs... Docs.rs had a error still....
- Redis is now 0.23.0
- surrealdb is now 1.0.0-beta.9+20230402

## 0.1.8 (11. April, 2023)
### Changed
- Made pub(crate) visible in docs... Docs.rs had a error..

## 0.1.7 (10. April, 2023)
### Changed
- Made pub(crate) visible in docs... Docs.rs had a error..

## 0.1.6 (10. April, 2023)
### Changed
- Made pub(crate) visible in docs.

## 0.1.5 (30. March, 2023)
### Changed
- Fixed Readme.

## 0.1.4 (30. March, 2023)
### Changed
- Made SessionID Public and Retrievable @mwcz.

## 0.1.3 (27. March, 2023)
### Changed
- Fixed SqlLite delete all @cold-brewed.

## 0.1.2 (16. March, 2023)
### Changed
- Fixed Readme Layout.

## 0.1.1 (16. March, 2023)
### Changed
- Added functions to SessionData and SessionStore for Backend usage.

### Added
- ReadOnlySession.

## 0.1.0 (13. March, 2023)
### Added
- Initial rename and reversioning.
