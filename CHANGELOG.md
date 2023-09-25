# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
## Unreleased
### Changed
- Reduced amount of internal clones to half.
- Cleaned up way Data gets destroyed or deleted.
- (Breaking) clean is async now due to filter needing to be arc/rwlocked.

### Fixed
- Removal cookies should now contain a SameSite::None to avoid browser warnings.
- key-store not removing keys due to having it require if the database should auto clean or not.
- Session key store not getting updated correctly due to filter not updating across threads as it should be Arc.

### Added
- get_store and get_mut_store to Session.

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
