use thingspace_sdk::models::{NiddCallback, NiddResponse};
use worker::{
  DurableObject, Env, Request, Response, Result, SqlStorage, State, console_debug, console_error,
  console_log, durable_object,
  wasm_bindgen::{self, UnwrapThrowExt},
};

#[durable_object]
pub struct Pigeon {
  sql: SqlStorage,
  #[allow(unused)]
  state: State,
  #[allow(unused)]
  env: Env, // access `Env` across requests, use inside `fetch`
}

// NiddCallback { request_id: "8782ed77-824c-4c4d-9118-104a97f74ebd", device_ids: [DeviceID { id: "350457799502610", kind: "IMEI" }], nidd_response: NiddMTDeliveryResponse { account_name: "0742644905-00001", acknowledge_time: None, first_attempt_delivery_time: None, reason: Some("Buffered, device not reachable"), device_ids: [DeviceID { id: "350457799502610", kind: "IMEI" }, DeviceID { id: "4062914013", kind: "MDN" }, DeviceID { id: "14062914013", kind: "MSISDN" }, DeviceID { id: "89148000008531108276", kind: "ICCID" }] }, status: Some("Queued"), callback_count: 1, max_callback_threshold: 2 }
// NiddCallback { request_id: "8ef55029-871c-4f7c-8c50-9d32d7e77d67", device_ids: [DeviceID { id: "89148000008531108276", kind: "ICCID" }], nidd_response: NiddMONotificationResponse { account_name: "0742644905-00001", message: "SGVsbG8sIFdvcmxkIQ\\u003d\\u003d", device_ids: [DeviceID { id: "350457799502610", kind: "IMEI" }, DeviceID { id: "311270028205048", kind: "IMSI" }, DeviceID { id: "4062914013", kind: "MDN" }, DeviceID { id: "4062912483", kind: "MIN" }, DeviceID { id: "14062914013", kind: "MSISDN" }, DeviceID { id: "89148000008531108276", kind: "ICCID" }] }, status: None, callback_count: 1, max_callback_threshold: 2 }
// NiddCallback { request_id: "35885d8e-6606-4d1e-b9f2-e74806b11b88", device_ids: [DeviceID { id: "350457799502610", kind: "IMEI" }], nidd_response: NiddMTDeliveryResponse { account_name: "0742644905-00001", acknowledge_time: Some(DateTime { date: YMD { year: 2025, month: 12, day: 11 }, time: Time { hour: 17, minute: 28, second: 53, millisecond: 108, tz_offset_hours: 0, tz_offset_minutes: 0 } }), first_attempt_delivery_time: None, reason: None, device_ids: [DeviceID { id: "350457799502610", kind: "IMEI" }, DeviceID { id: "4062914013", kind: "MDN" }, DeviceID { id: "14062914013", kind: "MSISDN" }, DeviceID { id: "89148000008531108276", kind: "ICCID" }] }, status: Some("Delivered"), callback_count: 1, max_callback_threshold: 2 }

impl DurableObject for Pigeon {
  fn new(state: State, env: Env) -> Pigeon {
    let sql = state.storage().sql();
    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS device (
          imei INTEGER NOT NULL PRIMARY KEY,
          mdn TEXT,
          imsi TEXT,
          iccid TEXT,
          msisdn TEXT,
          min TEXT
        );",
        None,
      )
      .expect("create table");

    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS nidd_mo_notification (
          request_id TEXT NOT NULL PRIMARY KEY,
          device_imei INTEGER NOT NULL,
          message TEXT NOT NULL,
          FOREIGN KEY (device_imei) REFERENCES device(imei) ON DELETE CASCADE
        );",
        None,
      )
      .expect("create table");

    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS nidd_mt_delivery (
          request_id TEXT NOT NULL PRIMARY KEY,
          device_imei INTEGER NOT NULL,
          acknowledge_time TEXT,
          first_attempt_delivery_time TEXT,
          reason TEXT,
          status TEXT NOT NULL,
          FOREIGN KEY (device_imei) REFERENCES device(imei) ON DELETE CASCADE
        );",
        None,
      )
      .expect("create table");

    Pigeon { sql, state, env }
  }

  async fn fetch(&self, mut req: Request) -> Result<Response> {
    let body = req.json::<NiddCallback>().await;

    match body {
      Ok(b) => {
        console_log!("{b:?}");

        match b.nidd_response {
          NiddResponse::NiddMONotificationResponse {
            account_name: _,
            message,
            device_ids,
          } => {
            let imei = insert_device_row(self, &device_ids)?;

            self.sql.exec(
              "INSERT INTO nidd_mo_notification (device_imei, request_id, message)
              VALUES (?, ?, ?);",
              vec![imei.into(), b.request_id.into(), message.into()],
            )?;
          }
          NiddResponse::NiddMTDeliveryResponse {
            account_name: _,
            acknowledge_time,
            first_attempt_delivery_time,
            reason,
            device_ids,
          } => {
            // let atime: Option<String> = None;
            // if let Some(time) = acknowledge_time {
            //   atime = time.fmt("%Y-%m-%dT%H:%M:%S%z", f).to_string();
            // }
            let imei = insert_device_row(self, &device_ids)?;

            self.sql.exec(
              "INSERT INTO nidd_mt_delivery (device_imei, request_id, acknowledge_time, first_attempt_delivery_time, reason, status)
              VALUES (?, ?, ?, ?, ?, ?);",
              vec![imei.into(), b.request_id.into(), acknowledge_time.into(), first_attempt_delivery_time.into(), reason.into(), b.status.into()],
            )?;
          }
        }
      }
      Err(e) => {
        console_error!("{e}");
        console_debug!("{:?}", req.text().await);
      }
    }
    Response::empty()
  }

  // async fn websocket_message(
  //   &self,
  //   ws: worker::WebSocket,
  //   message: worker::WebSocketIncomingMessage,
  // ) -> Result<()> {
  //   console_error!("websocket_message() handler not implemented");
  //   std::unimplemented!("websocket_message() handler")
  // }

  // async fn websocket_close(
  //   &self,
  //   ws: worker::WebSocket,
  //   code: usize,
  //   reason: String,
  //   was_clean: bool,
  // ) -> Result<()> {
  //   console_error!("websocket_close() handler not implemented");
  //   std::unimplemented!("websocket_close() handler")
  // }

  // async fn websocket_error(&self, ws: worker::WebSocket, error: worker::Error) -> Result<()> {
  //   console_error!("websocket_error() handler not implemented");
  //   std::unimplemented!("websocket_error() handler")
  // }
}

fn insert_device_row(
  pigeon: &Pigeon,
  device_ids: &[thingspace_sdk::models::DeviceID],
) -> Result<i64> {
  let mut imei: i64 = 0;
  let mut imsi = String::with_capacity(15);
  let mut mdn = String::with_capacity(10);
  let mut min = String::with_capacity(10);
  let mut msisdn = String::with_capacity(11);
  let mut iccid = String::with_capacity(20);

  // DeviceID { id: "350457799502610", kind: "IMEI" },
  // DeviceID { id: "4062914013", kind: "MDN" },
  // DeviceID { id: "14062914013", kind: "MSISDN" },
  // DeviceID { id: "89148000008531108276", kind: "ICCID" }
  // console_log!(
  //   " imei: {imei}\n imsi: {imsi}\n mdn: {mdn}\n min: {min}\n msisdn: {msisdn}\n iccid: {iccid}"
  // );

  for did in device_ids.iter() {
    match did.kind.as_str() {
      "IMEI" => imei = did.id.parse().unwrap_throw(),
      "IMSI" => imsi = did.id.to_owned(),
      "MDN" => mdn = did.id.to_owned(),
      "MIN" => min = did.id.to_owned(),
      "MSISDN" => msisdn = did.id.to_owned(),
      "ICCID" => iccid = did.id.to_owned(),
      _ => continue,
    }
  }

  pigeon.sql.exec(
    "INSERT INTO device (imei, mdn, imsi, iccid, msisdn, min)
              VALUES (?, ?, ?, ?, ?, ?)
              ON CONFLICT(imei) DO NOTHING;",
    vec![
      imei.into(),
      mdn.into(),
      imsi.into(),
      iccid.into(),
      msisdn.into(),
      min.into(),
    ],
  )?;

  Ok(imei)
}
