use std::collections::HashMap;

use actix_web::web;
use actix_web::Resource;

/// Type of API enablement flags.
///
/// Applications can conditionally enable roots based on application defined flags.
/// When the routes are mounted, applications can make use of the `APIFlags` and
/// proveded methods from `RootDescriptor`s to configure the mounted roots at
/// application initialisation time.
pub type APIFlags = HashMap<&'static str, bool>;

/// Interface for types that describe endpoint roots.
///
/// Useful to codify in a sensible way which possible paths are exposed by an API.
///
/// ## Example
/// ```
///# use std::collections::HashMap;
/// use replicante_util_actixweb::RootDescriptor;
///
/// enum APIVersion {
///     V1,
///     V2,
///     V3,
/// }
///
/// impl RootDescriptor for APIVersion {
///     fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool {
///         match self {
///             APIVersion::V1 => match flags.get("v1") {
///                 Some(flag) => *flag,
///                 None => match flags.get("legacy") {
///                     Some(flag) => *flag,
///                     None => true,
///                 },
///             },
///             APIVersion::V2 => match flags.get("v2") {
///                 Some(flag) => *flag,
///                 None => match flags.get("legacy") {
///                     Some(flag) => *flag,
///                     None => true,
///                 },
///             },
///             APIVersion::V3 => true,
///         }
///     }
///
///     fn prefix(&self) -> &'static str {
///         match self {
///             APIVersion::V1 => "/api/v1",
///             APIVersion::V2 => "/some/other/path",
///             APIVersion::V3 => "/v3",
///         }
///     }
/// }
/// ```
pub trait RootDescriptor {
    /// Perform configuration only if the root is enabled.
    fn and_then<C>(&self, flags: &APIFlags, config: C)
    where
        C: FnOnce(&Self),
    {
        if !self.enabled(flags) {
            return;
        }
        config(self);
    }

    /// Check if a root should be enabled according to the given flags.
    fn enabled(&self, flags: &APIFlags) -> bool;

    /// Return the URI prefix for a root.
    fn prefix(&self) -> &'static str;

    /// Create a resource for a path underneath the root.
    fn resource(&self, path: &str) -> Resource {
        match path {
            "" | "/" => web::resource(self.prefix()),
            path if path.starts_with('/') => {
                let path = format!("{}{}", self.prefix(), path);
                web::resource(&path)
            }
            path => {
                let path = format!("{}/{}", self.prefix(), path);
                web::resource(&path)
            }
        }
    }
}
