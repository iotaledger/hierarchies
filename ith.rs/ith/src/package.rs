// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! ITH package registry
//!
//! This module provides a registry of ITH packages.
//!
//! The registry is a [`RwLock`] that is initialized with the contents of the
//! `ith.move/Move.lock` file.
//!
//! The registry is used to lookup the package ID for the ITH package for a
//! given network.

#![allow(dead_code)]

use std::sync::LazyLock;

use iota_interaction::types::base_types::ObjectID;
use product_common::core_client::CoreClientReadOnly;
use product_common::package_registry::PackageRegistry;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use crate::error::ConfigError;

type PackageRegistryLock = RwLockReadGuard<'static, PackageRegistry>;
type PackageRegistryLockMut = RwLockWriteGuard<'static, PackageRegistry>;

static ITH_PACKAGE_REGISTRY: LazyLock<RwLock<PackageRegistry>> = LazyLock::new(|| {
    let move_lock_content = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ith.move/Move.lock"));
    RwLock::new(PackageRegistry::from_move_lock_content(move_lock_content).expect("Move.lock exists and it's valid"))
});

pub(crate) async fn ith_package_registry() -> PackageRegistryLock {
    ITH_PACKAGE_REGISTRY.read().await
}

pub(crate) fn try_ith_package_registry() -> Result<PackageRegistryLock, TryLockError> {
    ITH_PACKAGE_REGISTRY.try_read()
}

pub(crate) fn blocking_ith_registry() -> PackageRegistryLock {
    ITH_PACKAGE_REGISTRY.blocking_read()
}

pub(crate) async fn ith_package_registry_mut() -> PackageRegistryLockMut {
    ITH_PACKAGE_REGISTRY.write().await
}

pub(crate) fn try_ith_package_registry_mut() -> Result<PackageRegistryLockMut, TryLockError> {
    ITH_PACKAGE_REGISTRY.try_write()
}

pub(crate) fn blocking_ith_registry_mut() -> PackageRegistryLockMut {
    ITH_PACKAGE_REGISTRY.blocking_write()
}

/// Returns the package ID for the ITH package.
pub(crate) async fn ith_package_id<C>(client: &C) -> Result<ObjectID, ConfigError>
where
    C: CoreClientReadOnly,
{
    let network = client.network_name().as_ref();
    ith_package_registry()
        .await
        .package_id(network)
        .ok_or_else(|| ConfigError::PackageNotFound {
            network: network.to_string(),
        })
}
