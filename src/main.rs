use chrono::prelude::*;
use std::{fs::File, io::Write};

#[macro_use]
extern crate rocket;

#[post("/putqso")]
async fn putqso() {}

#[get("/webui")]
async fn webui() -> &'static str {
    "TODO"
}

fn write_logsheet(data: Vec<QSO>) {
    let file = File::create("logsheet.txt").expect("Unable to create file");
    _append_to_logsheet(&file, "<LOGSHEET TYPE=ylog>\n");
    for qso in data {
        _append_to_logsheet(&file, format_qso(qso).as_str())
    }
    _append_to_logsheet(&file, "\n</LOGSHEET>\n")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_write_logsheet() {
        let my_qso = super::QSO {
            datetime: super::Utc::now(),
            band: String::from("50"),
            mode: String::from("SSB"),
            callsign: String::from("JA1YXP"),
            sent_rst: String::from("59"),
            sent_num: String::from("13M"),
            recv_rst: String::from("59"),
            recv_num: String::from("20M"),
            multi: String::from("20"),
            score: 2,
        };
        super::write_logsheet(vec![my_qso]);
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/putqso", routes![putqso])
        .mount("/webui", routes![webui])
}

fn _append_to_logsheet(mut file: &File, data: &str) {
    file.write(data.as_bytes()).expect("Failed to write data");
}

fn format_qso(qso: QSO) -> String {
    let date = qso.datetime.format("%Y-%m-%d").to_string();
    let time = qso.datetime.format("%H:%M").to_string();
    format!(
        "{} {} {} {} {} {} {} {} {} {} {}",
        date,
        time,
        qso.band,
        qso.mode,
        qso.callsign,
        qso.sent_rst,
        qso.sent_num,
        qso.recv_rst,
        qso.recv_num,
        qso.multi,
        qso.score,
    )
}

struct QSO {
    datetime: DateTime<Utc>,
    band: String,
    mode: String,
    callsign: String,
    sent_rst: String,
    sent_num: String,
    recv_rst: String,
    recv_num: String,
    multi: String,
    score: i32,
}
