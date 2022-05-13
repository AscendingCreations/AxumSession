# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

# Unreleased

- **breaking:** GDPR Compliance.
- **breaking:** Data Cookie ID only set if disable_gdpr is true or Accepted is true.
- **added:** Config for GDPR Sessions.
- **added:** GDPR Memory and Database Session unloading on not accepted.
- **added:** Accepted GDPR Cookie.
- **breaking:** GDPR is Enabled by default so you must use set_accepted on user data for a session to save or disable gdpr_mode.

# 1.1.0 (12. May, 2022)

- **added:** Long Term Session Switch. Useful for Remember Me.

# 1.0.1 (4. April, 2022)

- **fixed:** Removed need for Tower-cookies and implemented cookie handling.

# 1.0.0 (31. March, 2022)

- **breaking:** Updated to `Axum` 0.5.

# 0.2.1 (22. Feburary, 2022)

- **Improved:** Replaced Axum with Axum core.

# 0.2.0 (22. Feburary, 2022)

- **added:** Capability to not be persistent.

# 0.1.0 (22. Feburary, 2022)

- Initial release.
