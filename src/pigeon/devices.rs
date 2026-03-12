use thingspace_sdk::models::{AccountDeviceListResponse, Device, NiddCallback};
use worker::{
  DurableObject, Env, Request, Response, Result, SqlStorage, State, console_error, console_log,
  durable_object,
  wasm_bindgen::{self, UnwrapThrowExt},
};

#[durable_object]
pub struct Pigeon {
  sql: SqlStorage,
  #[allow(unused)]
  state: State,
  env: Env, // access `Env` across requests, use inside `fetch`
}

impl DurableObject for Pigeon {
  fn new(state: State, env: Env) -> Pigeon {
    let sql = state.storage().sql();
    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS devices (
          device_id INTEGER NOT NULL PRIMARY KEY,
          messages_id TEXT NOT NULL,
          imei TEXT,
          iccid TEXT,
          mdn TEXT,
          imsi TEXT,
          msisdn TEXT,
          min TEXT,
          account_name TEXT,
          billing_cycle_end_date TEXT,
          group_name TEXT,
          last_activation_by TEXT,
          last_activation_date TEXT,
          carrier_name TEXT,
          service_plan TEXT,
          state TEXT,
          last_connection_date TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_devices_imei ON devices(imei);
        CREATE INDEX IF NOT EXISTS idx_devices_iccid ON devices(iccid);

        CREATE TABLE IF NOT EXISTS extended_attributes (
          device_id INTEGER NOT NULL PRIMARY KEY,
          primary_place_of_use_first_name TEXT,
          primary_place_of_use_last_name TEXT,
          primary_place_of_use_address_line_1 TEXT,
          primary_place_of_use_city TEXT,
          primary_place_of_use_state TEXT,
          primary_place_of_use_country TEXT,
          primary_place_of_use_zip_code TEXT,
          primary_place_of_use_zip_code_4 TEXT,
          account_number TEXT,
          sku_number TEXT,
          pre_imei TEXT,
          sim_ota_date TEXT,
          roaming_status TEXT,
          last_roaming_status_update TEXT,
          FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
        );",
        None,
      )
      .expect("create table");

    Pigeon { sql, state, env }
  }

  async fn fetch(&self, req: Request) -> Result<Response> {
    match req.path().as_str() {
      "/vzw/nidd" => match req.clone()?.json::<NiddCallback>().await {
        Ok(b) => {
          console_log!("{b:?}");
          #[derive(serde::Deserialize)]
          struct Row {
            messages_id: String,
          }

          let namespace = self.env.durable_object("NIDDMESSAGES")?;
          let mut query = String::with_capacity(58);
          query.push_str("SELECT messages_id FROM devices WHERE ");
          query.push_str(&b.device_ids[0].kind.to_lowercase());
          query.push_str(" = ? LIMIT 1;");

          let table_id = self
            .sql
            .exec(&query, vec![b.device_ids[0].id.to_owned().into()])?
            .one::<Row>()?;

          let stub = namespace
            .id_from_string(&table_id.messages_id)?
            .get_stub()?;
          match stub.fetch_with_request(req).await {
            Ok(_) => Response::empty(),
            Err(e) => {
              console_error!("{e}");
              Response::error(e.to_string(), 500)
            }
          }
        }
        Err(e) => {
          console_error!("{e}");
          // console_debug!("{:?}", req.text().await);
          Response::error(e.to_string(), 500)
        }
      },
      "/vzw/update_tables" => match init_device_table(self, req).await {
        Ok(_) => Response::empty(),
        Err(e) => {
          console_error!("{e}");
          Response::error(e.to_string(), 500)
        }
      },
      _ => Response::error("Not found", 404),
    }
  }
}

async fn init_device_table(pigeon: &Pigeon, mut req: Request) -> Result<Response> {
  let res = req.json::<AccountDeviceListResponse>().await;
  match res {
    Ok(adl) => {
      for dev in adl.devices {
        insert_device_row(pigeon, dev)?;
      }
      Response::empty()
    }
    Err(e) => {
      console_error!("{e}");
      Response::error(e.to_string(), 500)
    }
  }
}

fn insert_device_row(pigeon: &Pigeon, device: Device) -> Result<()> {
  let mut imei = String::with_capacity(15);
  let mut imsi = String::with_capacity(15);
  let mut mdn = String::with_capacity(10);
  let mut min = String::with_capacity(10);
  let mut msisdn = String::with_capacity(11);
  let mut iccid = String::with_capacity(20);

  for did in device.device_ids.iter() {
    match did.kind.to_lowercase().as_str() {
      "imei" => imei = did.id.to_owned(),
      "imsi" => imsi = did.id.to_owned(),
      "mdn" => mdn = did.id.to_owned(),
      "min" => min = did.id.to_owned(),
      "msisdn" => msisdn = did.id.to_owned(),
      "iccid" => iccid = did.id.to_owned(),
      _ => continue,
    }
  }

  let mut primary_place_of_use_first_name: Option<String> = None;
  let mut primary_place_of_use_last_name: Option<String> = None;
  let mut primary_place_of_use_address_line_1: Option<String> = None;
  let mut primary_place_of_use_city: Option<String> = None;
  let mut primary_place_of_use_state: Option<String> = None;
  let mut primary_place_of_use_country: Option<String> = None;
  let mut primary_place_of_use_zip_code: Option<String> = None;
  let mut primary_place_of_use_zip_code_4: Option<String> = None;
  let mut account_number: Option<String> = None;
  let mut sku_number: Option<String> = None;
  let mut pre_imei: Option<String> = None;
  let mut sim_ota_date: Option<String> = None;
  let mut roaming_status: Option<String> = None;
  let mut last_roaming_status_update: Option<String> = None;
  let mut device_id = 0;

  for ea in device.extended_attributes.iter() {
    match ea.key.as_str() {
      "PrimaryPlaceOfUseFirstName" => primary_place_of_use_first_name = ea.value.to_owned(),
      "PrimaryPlaceOfUseLastName" => primary_place_of_use_last_name = ea.value.to_owned(),
      "PrimaryPlaceOfUseAddressLine1" => primary_place_of_use_address_line_1 = ea.value.to_owned(),
      "PrimaryPlaceOfUseCity" => primary_place_of_use_city = ea.value.to_owned(),
      "PrimaryPlaceOfUseState" => primary_place_of_use_state = ea.value.to_owned(),
      "PrimaryPlaceOfUseCountry" => primary_place_of_use_country = ea.value.to_owned(),
      "PrimaryPlaceOfUseZipCode" => primary_place_of_use_zip_code = ea.value.to_owned(),
      "PrimaryPlaceOfUseZipCode4" => primary_place_of_use_zip_code_4 = ea.value.to_owned(),
      "AccountNumber" => account_number = ea.value.to_owned(),
      "SkuNumber" => sku_number = ea.value.to_owned(),
      "PreIMEI" => pre_imei = ea.value.to_owned(),
      "SIMOTADate" => sim_ota_date = ea.value.to_owned(),
      "RoamingStatus" => roaming_status = ea.value.to_owned(),
      "LastRoamingStatusUpdate" => last_roaming_status_update = ea.value.to_owned(),
      "DeviceId" => {
        device_id = ea
          .value
          .to_owned()
          .expect_throw("Device Extended Attributes missing device_id")
          .parse()
          .unwrap_throw();
      }
      _ => continue,
    }
  }

  let namespace = pigeon.env.durable_object("NIDDMESSAGES")?;
  let messages_id = namespace.unique_id()?.to_string();

  pigeon.sql.exec(
    "INSERT INTO devices (
      device_id,
      messages_id,
      imei,
      iccid,
      mdn,
      imsi,
      msisdn,
      min,
      account_name,
      billing_cycle_end_date,
      group_name,
      last_activation_by,
      last_activation_date,
      carrier_name,
      service_plan,
      state,
      last_connection_date
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT DO UPDATE SET
    imei = excluded.imei,
    iccid = excluded.iccid,
    mdn = excluded.mdn,
    imsi = excluded.imsi,
    msisdn = excluded.msisdn,
    min = excluded.min,
    account_name = excluded.account_name,
    billing_cycle_end_date = excluded.billing_cycle_end_date,
    group_name = excluded.group_name,
    last_activation_by = excluded.last_activation_by,
    last_activation_date = excluded.last_activation_date,
    carrier_name = excluded.carrier_name,
    service_plan = excluded.service_plan,
    state = excluded.state,
    last_connection_date = excluded.last_connection_date;",
    vec![
      device_id.into(),
      messages_id.into(),
      imei.into(),
      iccid.into(),
      mdn.into(),
      imsi.into(),
      msisdn.into(),
      min.into(),
      device.account_name.into(),
      device.billing_cycle_end_date.into(),
      device.group_names[0].to_owned().into(),
      device.last_activation_by.into(),
      device.last_activation_date.into(),
      device.carrier_informations[0]
        .to_owned()
        .carrier_name
        .into(),
      device.carrier_informations[0]
        .to_owned()
        .service_plan
        .into(),
      device.carrier_informations[0].to_owned().state.into(),
      device.last_connection_date.into(),
    ],
  )?;

  pigeon.sql.exec(
    "INSERT INTO extended_attributes (
      device_id,
      primary_place_of_use_first_name,
      primary_place_of_use_last_name,
      primary_place_of_use_address_line_1,
      primary_place_of_use_city,
      primary_place_of_use_state,
      primary_place_of_use_country,
      primary_place_of_use_zip_code,
      primary_place_of_use_zip_code_4,
      account_number,
      sku_number,
      pre_imei,
      sim_ota_date,
      roaming_status,
      last_roaming_status_update
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT DO UPDATE SET
    primary_place_of_use_first_name = excluded.primary_place_of_use_first_name,
    primary_place_of_use_last_name = excluded.primary_place_of_use_last_name,
    primary_place_of_use_address_line_1 = excluded.primary_place_of_use_address_line_1,
    primary_place_of_use_city = excluded.primary_place_of_use_city,
    primary_place_of_use_state = excluded.primary_place_of_use_state,
    primary_place_of_use_country = excluded.primary_place_of_use_country,
    primary_place_of_use_zip_code = excluded.primary_place_of_use_zip_code,
    primary_place_of_use_zip_code_4 = excluded.primary_place_of_use_zip_code_4,
    account_number = excluded.account_number,
    sku_number = excluded.sku_number,
    pre_imei = excluded.pre_imei,
    sim_ota_date = excluded.sim_ota_date,
    roaming_status = excluded.roaming_status,
    last_roaming_status_update = excluded.last_roaming_status_update;",
    vec![
      device_id.into(),
      primary_place_of_use_first_name.into(),
      primary_place_of_use_last_name.into(),
      primary_place_of_use_address_line_1.into(),
      primary_place_of_use_city.into(),
      primary_place_of_use_state.into(),
      primary_place_of_use_country.into(),
      primary_place_of_use_zip_code.into(),
      primary_place_of_use_zip_code_4.into(),
      account_number.into(),
      sku_number.into(),
      pre_imei.into(),
      sim_ota_date.into(),
      roaming_status.into(),
      last_roaming_status_update.into(),
    ],
  )?;

  Ok(())
}
