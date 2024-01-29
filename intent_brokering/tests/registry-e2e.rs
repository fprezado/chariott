// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use common::get_uuid;
use examples_common::intent_brokering::{
    api::{GrpcIntentBrokering, IntentBrokering, IntentBrokeringExt as _},
    registration::Builder as RegistrationBuilder,
};
use intent_brokering_proto::runtime::{
    intent_registration::Intent, intent_service_registration::ExecutionLocality,
};
use tokio::time::*;
use tokio_stream::StreamExt as _;

mod common;

#[tokio::test]
async fn expired_registrations_are_pruned_after_ttl() -> Result<(), anyhow::Error> {
    // arrange
    let namespace = format!("e2e.registration.{}", get_uuid());

    let builder = RegistrationBuilder::new(
        "e2e",
        "1.0.0",
        "http://localhost/".parse().unwrap(), // DevSkim: ignore DS162092
        &namespace,
        [Intent::Inspect],
        ExecutionLocality::Local,
    );

    let mut intent_broker = setup().await;

    // act
    builder.register_once(&mut None, true).await?;

    let initial_entries = intent_broker.inspect("system.registry", namespace.clone()).await?;
    let ttl =
        Duration::from_secs(env!("INTENT_BROKERING_REGISTRY_TTL_SECS").parse::<u64>().unwrap() + 1);
    sleep(ttl).await;
    let entries = intent_broker.inspect("system.registry", namespace).await?;

    // assert
    assert_eq!(1, initial_entries.len());
    assert_eq!(0, entries.len());

    Ok(())
}

#[tokio::test]
async fn when_provider_registers_notifies_registry_observers() -> anyhow::Result<()> {
    fn namespace_event(namespace: &str) -> Box<str> {
        format!("namespaces/{}", namespace).into()
    }

    // arrange
    let namespace = format!("e2e.registration.{}", get_uuid());

    let builder = RegistrationBuilder::new(
        "registration.provider.e2e",
        "1.0.0",
        // arbitrary url, the provider will never be invoked
        "http://localhost:7090".parse().unwrap(), // DevSkim: ignore DS162092
        &namespace,
        [Intent::Inspect],
        ExecutionLocality::Local,
    );

    let mut subject = setup().await;

    // act
    let stream = subject.listen("system.registry", vec![namespace_event(&namespace)]).await?;

    builder.register_once(&mut None, true).await?;

    // assert
    let stream = stream.timeout(Duration::from_secs(5)).take(1).collect::<Vec<_>>().await;
    let result = stream.into_iter().next().unwrap();
    assert_eq!(namespace_event(&namespace), result.unwrap().unwrap().id);

    Ok(())
}

async fn setup() -> impl IntentBrokering {
    GrpcIntentBrokering::connect().await.unwrap()
}
