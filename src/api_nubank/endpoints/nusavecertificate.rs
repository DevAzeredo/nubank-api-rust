use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::api_nubank::{discover::Discovery, cert::certificate_dao, nubank::nubank_dao};


