// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;
use futures::executor;
use identity::core::ToJson;
use identity::iota::Client as IotaClient;
use identity::iota::DocumentDiff;
use identity::iota::IotaDID;
use identity::iota::IotaDocument;
use identity::iota::MessageId;
use js_sys::Promise;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::tangle::Config;
use crate::tangle::WasmNetwork;
use crate::utils::err;

type Shared<T> = Rc<RefCell<T>>;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Client {
  client: Shared<IotaClient>,
}

#[wasm_bindgen]
impl Client {
  fn from_client(client: IotaClient) -> Client {
    Self {
      client: Rc::new(RefCell::new(client)),
    }
  }

  /// Creates a new `Client` with default settings.
  #[wasm_bindgen(constructor)]
  pub fn new() -> Result<Client, JsValue> {
    Self::from_config(&mut Config::new())
  }

  /// Creates a new `Client` with settings from the given `Config`.
  #[wasm_bindgen(js_name = fromConfig)]
  pub fn from_config(config: &mut Config) -> Result<Client, JsValue> {
    let future = config.take_builder()?.build();
    let output = executor::block_on(future).map_err(err);

    output.map(Self::from_client)
  }

  /// Creates a new `Client` with default settings for the given `Network`.
  #[wasm_bindgen(js_name = fromNetwork)]
  pub fn from_network(network: WasmNetwork) -> Result<Client, JsValue> {
    let future = IotaClient::from_network(network.into());
    let output = executor::block_on(future).map_err(err);

    output.map(Self::from_client)
  }

  /// Returns the `Client` Tangle network.
  #[wasm_bindgen]
  pub fn network(&self) -> WasmNetwork {
    self.client.borrow().network().into()
  }

  /// Returns the default node URL of the `Client` network.
  #[wasm_bindgen(getter = defaultNodeURL)]
  pub fn default_node_url(&self) -> String {
    self.client.borrow().default_node_url().to_string()
  }

  /// Returns the web explorer URL of the `Client` network.
  #[wasm_bindgen(getter = explorerURL)]
  pub fn explorer_url(&self) -> String {
    self.client.borrow().explorer_url().to_string()
  }

  /// Returns the web explorer URL of the given `transaction`.
  #[wasm_bindgen(js_name = transactionURL)]
  pub fn transaction_url(&self, message_id: &str) -> String {
    self.client.borrow().transaction_url(message_id).to_string()
  }

  /// Publishes an `IotaDocument` to the Tangle.
  #[wasm_bindgen(js_name = publishDocument)]
  pub fn publish_document(&self, document: &JsValue) -> Result<Promise, JsValue> {
    let document: IotaDocument = document.into_serde().map_err(err)?;
    let client: Shared<IotaClient> = self.client.clone();

    let promise: Promise = future_to_promise(async move {
      client
        .borrow()
        .publish_document(&document)
        .await
        .map_err(err)
        .map(|message| message.to_string().into())
    });

    Ok(promise)
  }

  /// Publishes a `DocumentDiff` to the Tangle.
  #[wasm_bindgen(js_name = publishDiff)]
  pub fn publish_diff(&self, message_id: &str, value: &JsValue) -> Result<Promise, JsValue> {
    let diff: DocumentDiff = value.into_serde().map_err(err)?;
    let message: MessageId = MessageId::from_str(message_id).map_err(err)?;
    let client: Shared<IotaClient> = self.client.clone();

    let promise: Promise = future_to_promise(async move {
      client
        .borrow()
        .publish_diff(&message, &diff)
        .await
        .map_err(err)
        .map(|message| message.to_string().into())
    });

    Ok(promise)
  }

  #[wasm_bindgen(js_name = readDocument)]
  pub fn read_document(&self, did: &str) -> Result<Promise, JsValue> {
    let client: Shared<IotaClient> = self.client.clone();
    let did: IotaDID = did.parse().map_err(err)?;

    let promise: Promise = future_to_promise(async move {
      client
        .borrow()
        .read_document(&did)
        .await
        .map_err(err)
        .and_then(|document| document.to_json().map_err(err))
        .map(Into::into)
    });

    Ok(promise)
  }
}
