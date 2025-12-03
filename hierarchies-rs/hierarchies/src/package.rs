// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Hierarchies package registry
//!
//! This module provides a registry of Hierarchies packages.
//!
//! The registry is a [`RwLock`] that is initialized with the contents of the
//! `hierarchies-move/Move.lock` file.
//!
//! The registry is used to lookup the package ID for the Hierarchies package for a
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

#[allow(deprecated)] // TODO : Remove after MoveHistoryManager is released with product-core
static HIERARCHIES_PACKAGE_REGISTRY: LazyLock<RwLock<PackageRegistry>> = LazyLock::new(|| {
    let package_history_json = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../hierarchies-move/Move.history.json"
    ));
    RwLock::new(
        PackageRegistry::from_package_history_json_str(package_history_json)
            .expect("Move.history.json exists and it's valid"),
    )
});

pub(crate) async fn hierarchies_package_registry() -> PackageRegistryLock {
    HIERARCHIES_PACKAGE_REGISTRY.read().await
}

pub(crate) fn try_hierarchies_package_registry() -> Result<PackageRegistryLock, TryLockError> {
    HIERARCHIES_PACKAGE_REGISTRY.try_read()
}

pub(crate) fn blocking_hierarchies_registry() -> PackageRegistryLock {
    HIERARCHIES_PACKAGE_REGISTRY.blocking_read()
}

pub(crate) async fn hierarchies_package_registry_mut() -> PackageRegistryLockMut {
    HIERARCHIES_PACKAGE_REGISTRY.write().await
}

pub(crate) fn try_hierarchies_package_registry_mut() -> Result<PackageRegistryLockMut, TryLockError> {
    HIERARCHIES_PACKAGE_REGISTRY.try_write()
}

pub(crate) fn blocking_hierarchies_registry_mut() -> PackageRegistryLockMut {
    HIERARCHIES_PACKAGE_REGISTRY.blocking_write()
}

/// Returns the package ID for the Hierarchies package.
pub(crate) async fn hierarchies_package_id<C>(client: &C) -> Result<ObjectID, ConfigError>
where
    C: CoreClientReadOnly,
{
    let network = client.network_name().as_ref();
    hierarchies_package_registry()
        .await
        .package_id(network)
        .ok_or_else(|| ConfigError::PackageNotFound {
            network: network.to_string(),
        })
}
