use thingspace_sdk::models::{NiddCallback, NiddResponse};
use worker::{
  DurableObject, Env, Request, Response, Result, SqlStorage, State, console_debug, console_error,
  durable_object, wasm_bindgen,
};

#[durable_object]
pub struct NiddMessages {
  sql: SqlStorage,
  #[allow(unused)]
  state: State,
  #[allow(unused)]
  env: Env, // access `Env` across requests, use inside `fetch`
}

impl DurableObject for NiddMessages {
  fn new(state: State, env: Env) -> NiddMessages {
    let sql = state.storage().sql();
    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS nidd_mo_notifications (
          request_id TEXT NOT NULL PRIMARY KEY,
          device_imei TEXT NOT NULL,
          message TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS nidd_mt_deliveries (
          request_id TEXT NOT NULL PRIMARY KEY,
          device_imei TEXT NOT NULL,
          acknowledge_time TEXT,
          first_attempt_delivery_time TEXT,
          reason TEXT,
          status TEXT NOT NULL
        );",
        None,
      )
      .expect("create table");

    NiddMessages { sql, state, env }
  }

  async fn fetch(&self, mut req: Request) -> Result<Response> {
    match req.json::<NiddCallback>().await {
      Ok(b) => match b.nidd_response {
        NiddResponse::NiddMONotificationResponse {
          account_name: _,
          message,
          device_ids,
        } => {
          let mut imei = String::with_capacity(15);
          for did in device_ids.iter() {
            match did.kind.to_lowercase().as_str() {
              "imei" => {
                imei = did.id.to_owned();
                break;
              }
              _ => continue,
            }
          }
          self.sql.exec(
            "INSERT INTO nidd_mo_notifications (request_id, device_imei, message)
              VALUES (?, ?, ?);",
            vec![b.request_id.into(), imei.into(), message.into()],
          )?;
          Response::empty()
        }
        NiddResponse::NiddMTDeliveryResponse {
          account_name: _,
          acknowledge_time,
          first_attempt_delivery_time,
          reason,
          device_ids,
        } => {
          let mut imei = String::with_capacity(15);
          for did in device_ids.iter() {
            match did.kind.to_lowercase().as_str() {
              "imei" => {
                imei = did.id.to_owned();
                break;
              }
              _ => continue,
            }
          }
          self.sql.exec(
              "INSERT INTO nidd_mt_deliveries (request_id, device_imei, acknowledge_time, first_attempt_delivery_time, reason, status)
              VALUES (?, ?, ?, ?, ?, ?);",
              vec![b.request_id.into(), imei.into(), acknowledge_time.into(), first_attempt_delivery_time.into(), reason.into(), b.status.into()],
            )?;
          Response::empty()
        }
      },
      Err(e) => {
        console_error!("{e}");
        console_debug!("{:?}", req.text().await);
        Response::error(e.to_string(), 500)
      }
    }
  }
}
