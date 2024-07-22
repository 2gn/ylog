use chrono::prelude::*;
use html::form;
use icondata::{AiCloseOutlined, AiSendOutlined, LuPencil};
use leptos::*;
use leptos_use::*;
use mobile::{show_toast, ToastOptions};
use std::borrow::BorrowMut;
use std::net::TcpListener;
use std::thread::spawn;
use std::time::Duration;
use std::{fmt::format, fs::File, io::Write};
use svg::view;
use thaw::*;
use tungstenite::{
    accept, accept_hdr,
    handshake::server::{Request, Response},
};
use web_sys::js_sys::Reflect::apply;

fn write_logsheet(data: Vec<QSO>) {
    let file = File::create("logsheet.txt").expect("Unable to create file");
    _append_to_logsheet(&file, "<LOGSHEET TYPE=ylog>\n");
    for qso in data {
        _append_to_logsheet(&file, format_qso(qso).as_str())
    }
    _append_to_logsheet(&file, "\n</LOGSHEET>\n")
}

#[component]
fn TableView() -> impl IntoView {
    let mut callsigns = vec![
        "JA1YXP", "JA1ZGP", "JA1ZGP", "JA1ZGP", "JA1ZGP", "JA1ZGP", "JA1ZGP", "JA1ZGP", "JA1ZGP",
    ];

    view! {
        <Scrollbar style="max-height: 70vh">
            <div style="min-height: 100vh">
                <Table>
                    <thead>
                        <tr>
                            <th>"Time"</th>
                            <th>"Callsign"</th>
                            <th>"Number"</th>
                            <th>"Mode"</th>
                            <th>"Operator"</th>
                            <th>"Modification"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {callsigns.into_iter()
                            .map(|callsign| view! {
                                <tr>
                                    <td>"2023-10-08"</td>
                                    <td>{callsign}</td>
                                    <td>"13M"</td>
                                    <td>"SSB"</td>
                                    <td>"Alex"</td>
                                    <td>
                                        <Space>
                                            <Button icon=LuPencil></Button>
                                            <Button color=ButtonColor::Error icon=AiCloseOutlined circle=true></Button>
                                        </Space>
                                    </td>
                                </tr>
                            })
                            .collect::<Vec<_>>()}
                    </tbody>
                </Table>
            </div>
        </Scrollbar>
    }
}

#[component]
fn CWKeyboard() -> impl IntoView {
    let onclick = move |_| {
        write_logsheet(vec![QSO {
            datetime: Utc::now(),
            band: String::from("50"),
            mode: String::from("SSB"),
            callsign: String::from("JA1YXP"),
            sent_rst: String::from("59"),
            sent_num: String::from("13M"),
            recv_rst: String::from("59"),
            recv_num: String::from("20M"),
            multi: String::from("20"),
            score: 2,
        }]);

        show_toast(ToastOptions {
            message: format!("QSO have been submitted"),
            duration: Duration::from_millis(2000),
        });
    };
    let band = create_rw_signal(String::from("50"));
    let callsign = create_rw_signal(String::from("JA1YXP"));
    let recv_num = create_rw_signal(String::from("13M"));

    let modes = vec![
        SelectOption::new("SSB", String::from("SSB")),
        SelectOption::new("CW", String::from("CW")),
        SelectOption::new("FM", String::from("FM")),
        SelectOption::new("RTTY", String::from("RTTY")),
    ];
    let mode = create_rw_signal(None::<String>);
    view! {
        <div style="min-width: 70vw">
            <Card>
                <Space >
                    <Input value=band/>
                    <Select value=mode options=modes/>
                    <Input value=callsign/>
                    <Input value=recv_num/>
                    <Button on_click=onclick variant=ButtonVariant::Primary>"Submit"</Button>
                </Space>
            </Card>
        </div>
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let percentage = create_rw_signal(0.0f32);
    view! {
        <Space>
            <ProgressCircle percentage/>
            <ProgressCircle percentage color=ProgressColor::Success/>
            <ProgressCircle percentage color=ProgressColor::Warning/>
            <ProgressCircle percentage color=ProgressColor::Error/>
        </Space>
    }
}

#[component]
fn Message() -> impl IntoView {
    let UseWebsocketReturn {
        ready_state,
        message,
        message_bytes,
        send,
        send_bytes,
        open,
        close,
        ..
    } = use_websocket("ws://127.0.0.1/message");
    open();
    let mut msgs: Vec<&str> = vec!["Message server started"];
    let status = move || ready_state.get().to_string();
    let send_message = {};
    view! {
        <Space vertical=true>
            {status}
            <Card>
                <div style="min-height: 73vh; min-width: 20vw">
                    {msgs.into_iter().map(|msg| view! {
                        <div>{msg}</div>
                    }).collect::<Vec<_>>()}
                </div>
            </Card>
            <Card>
                <Space>
                    <Input />
                    <Button icon=AiSendOutlined ></Button>
                </Space>
            </Card>
        </Space>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let dark_mode = create_rw_signal(false);
    let value = create_rw_signal(String::from("sheet"));
    let theme = create_rw_signal(Theme::light());
    let on_change = move |value: bool| {
        if value {
            theme.set(Theme::dark())
        } else {
            theme.set(Theme::light())
        }
    };
    view! {
        <ThemeProvider theme>
            <Space vertical=true>
                <Card>
                    <Space>
                        <div style="margin-left: auto">
                            <Switch value=dark_mode on_change />
                        </div>
                    </Space>
                </Card>
                <Space>
                    <Message />
                    <Tabs value>
                        <Tab key="sheet" label="Sheet">
                            <Space vertical=true>
                                <TableView />
                                <CWKeyboard />
                            </Space>
                        </Tab>
                        <Tab key="dash" label="Dashboard">
                            <Dashboard />
                        </Tab>
                    </Tabs>
                </Space>
            </Space>
        </ThemeProvider>
    }
}

fn main() {
    mount_to_body(App);
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    logging::log!("Setting up the server");
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, mut response: Response| {
                logging::log!("Received a new ws handshake");
                logging::log!("The request's path is: {}", req.uri().path());
                logging::log!("The request's headers are:");
                for (ref header, _value) in req.headers() {
                    logging::log!("* {}", header);
                }

                // Let's add an additional header to our response to the client.
                let headers = response.headers_mut();
                headers.append("MyCustomHeader", ":)".parse().unwrap());
                headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

                Ok(response)
            };
            logging::log!("Received a new ws handshake");
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();
            loop {
                let msg = websocket.read().unwrap();
                if msg.is_text() {
                    let msg = msg.to_text().unwrap();
                    websocket.write(msg.into()).unwrap();
                }
            }
        });
    }
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
