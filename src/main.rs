// #![windows_subsystem = "windows"]
#![allow(dead_code)]
use clipboard_ext::clipboard::ClipboardContext;
use clipboard_ext::prelude::ClipboardProvider;
use gtk::{
    prelude::{ComboBoxExtManual, WidgetExt},
    traits::{BoxExt, ButtonExt, ComboBoxExt, EntryExt, GridExt, GtkWindowExt, OrientableExt},
    Align, EntryBuffer, Inhibit,
};
use mc_server_scanner::packets::ServerQueryResponse;
use native_dialog::{MessageDialog, MessageType};
use relm4::*;
use std::net::IpAddr;
use tracing::{debug, error};
use utils::{
    async_handler::{AsyncHandler, AsyncHandlerMsg},
    local_ip::get_local_ip,
    port_forwarding::redirect_minecraft_to_a_port,
};

use portforwarder_rs::port_forwarder::{Forwarder, PortMappingProtocol};

pub mod utils;
struct AppComponents {
    async_handler: RelmMsgHandler<AsyncHandler, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        AppComponents {
            async_handler: RelmMsgHandler::new(parent_model, parent_sender),
        }
    }
    fn connect_parent(&mut self, _parent_widgets: &<AppModel as Model>::Widgets) {}
}

enum AppMsg {
    RescanServers,
    ServerScanResults(Vec<(ServerQueryResponse, u16)>),
    SelectedPort(Option<u16>),
    PortForward,
    Close,
}

#[tracker::track]
struct AppModel {
    public_ip: IpAddr,
    private_ip: IpAddr,
    external_port: u16,
    internal_ports: Vec<(String, u16)>,
    selected_minecraft_port: Option<u16>,
    scanning: bool,
    forwarder: MyForwarder,
}

pub struct MyForwarder(Forwarder);

impl PartialEq for MyForwarder {
    fn eq(&self, other: &Self) -> bool {
        self.0.network_interface == other.0.network_interface
    }
}

impl Eq for MyForwarder {}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::RescanServers => {
                debug!("Scanning for servers...");
                self.set_scanning(true);
                components
                    .async_handler
                    .sender()
                    .blocking_send(AsyncHandlerMsg::RescanServers)
                    .expect("Receiver dropped");
            }
            AppMsg::ServerScanResults(data) => {
                self.set_scanning(false);
                debug!("Found {} servers!", data.len());
                self.set_internal_ports(
                    data.into_iter()
                        .map(|a| (a.0.motd, a.1))
                        .collect::<Vec<_>>(),
                );
            }
            AppMsg::SelectedPort(port) => {
                self.set_selected_minecraft_port(port);
                self.reset();
            }
            AppMsg::PortForward => {
                if let Some(selected_minecraft_port) = self.get_selected_minecraft_port() {
                    let selected_minecraft_port = *selected_minecraft_port;
                    let redirect = {
                        let ports = *self.get_external_port();
                        let MyForwarder(fwd) = self.get_mut_forwarder();
                        redirect_minecraft_to_a_port(fwd, selected_minecraft_port, ports)
                    };
                    if let Err(a) = redirect {
                        show_error(&format!("Error occured while opening the ports.\n{}", a));
                        error!("{:?}", a);
                    } else {
                        let mut ctx = ClipboardContext::new().unwrap();
                        ctx.set_contents(format!(
                            "{}:{}",
                            self.get_public_ip(),
                            self.get_external_port()
                        ))
                        .unwrap();
                        MessageDialog::new()
                                .set_type(MessageType::Info)
                                .set_title("Port opened!")
                                .set_text(&format!("Minecraft should now be redirected from port {} to {}.\nPort will be open so long until the app is closed.\nPeople can join using {}:{}, it has also been copied to your clipboard.", selected_minecraft_port, self.get_external_port(), self.get_public_ip(), self.get_external_port()))
                                .show_alert()
                                .unwrap();
                    }
                } else {
                    show_error("No minecraft server selected.");
                }
            }
            AppMsg::Close => {
                let ports = self
                    .forwarder
                    .0
                    .open_ports
                    .iter()
                    .map(|a| (a.num, a.proto))
                    .collect::<Vec<(u16, PortMappingProtocol)>>();
                for port in ports.iter() {
                    if let Err(e) = self.forwarder.0.remove_port(port.0, port.1) {
                        error!("Failed to close port {}/{} due to {}", port.0, port.1, e);
                    }
                }
                debug!("All ports closed, closing the application.");
                return false;
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            connect_close_request(sender) => move |_|{
                send!(sender, AppMsg::Close);
                Inhibit(false)
            },
            set_child = Some(&gtk::Box){
                set_margin_all: 5,
                set_orientation: gtk::Orientation::Vertical,
                append = &gtk::Grid{
                    set_column_homogeneous: true,
                    //? Titles
                    attach(0, 0, 1, 1) = &gtk::Label{
                        set_halign: Align::Start,
                        set_label: "Public IP"
                    },
                    attach(0, 1, 1, 1) = &gtk::Label{
                        set_halign: Align::Start,
                        set_label: "Private IP"
                    },
                    attach(0, 2, 1, 1) = &gtk::Label{
                        set_halign: Align::Start,
                        set_label: "External port"
                    },
                    attach(0, 3, 1, 1) = &gtk::Label{
                        set_halign: Align::Start,
                        set_label: "Internal port"
                    },
                    //? Values
                    attach(1, 0, 1, 1) = &gtk::Label{
                        set_halign: Align::Start,
                        set_label: watch!(&model.get_public_ip().to_string())
                    },
                    attach(1, 1, 1, 1) = &gtk::Label{
                        set_halign: Align::Start,
                        set_label: watch!(&model.get_private_ip().to_string())
                    },
                    attach(1, 2, 1, 1) = &gtk::Entry{
                        set_buffer: watch!(&EntryBuffer::new(Some(&format!("{}", model.get_external_port())))),
                    },
                    attach(1, 3, 1, 1): internal_ports = &gtk::ComboBoxText{
                        set_hexpand: true,
                        connect_changed(sender) => move |a|{
                            let port = a.active_id().map(|id| id.to_string().parse::<u16>().unwrap());
                            send!(sender, AppMsg::SelectedPort(port));
                        }
                    },
                    //? Buttons
                    attach(0, 4, 1, 1) = &gtk::Button{
                        set_label: "Rescan minecraft instances",
                        set_visible: watch!(!model.scanning),
                        connect_clicked(sender) => move |_|{
                            send!(sender, AppMsg::RescanServers);
                        }
                    },
                    attach(0, 4, 1, 1) = &gtk::Spinner{
                        set_spinning: true,
                        set_visible: watch!(model.scanning),
                    },
                    attach(1, 4, 1, 1) = &gtk::Button{
                        set_label: "Start forwarding",
                        connect_clicked(sender) => move |_|{
                            send!(sender, AppMsg::PortForward);
                        }
                    },
                }
            }
        }
    }

    fn pre_view() {
        if model.changed(AppModel::internal_ports()) {
            self.internal_ports.remove_all();
            for f in model.internal_ports.iter() {
                self.internal_ports
                    .append(Some(&format!("{}", f.1)), f.0.as_str());
            }
            if !model.get_internal_ports().is_empty() {
                self.internal_ports.set_active(Some(0));
            } else {
                self.internal_ports.set_active(None);
            }
        }
    }
}

fn show_error(text: &str) {
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Error")
        .set_text(text)
        .show_alert()
        .unwrap();
}

fn main() {
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support();
    tracing_subscriber::fmt::init();
    let pub_addr = {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(8)
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        runtime.block_on(public_ip::addr())
    };
    if let Some(pub_addr) = pub_addr {
        if let Some(local_addr) = get_local_ip() {
            let forwarder = if let IpAddr::V4(a) = local_addr {
                if let Ok(a) = portforwarder_rs::port_forwarder::create_forwarder(a) {
                    a
                } else {
                    error!("Unable to create a forwarder!");
                    std::process::exit(0);
                }
            } else {
                error!("IPv6 is currently not supported!");
                std::process::exit(0);
            };
            let model = AppModel {
                public_ip: pub_addr,
                private_ip: local_addr,
                external_port: 25565,
                internal_ports: vec![],
                selected_minecraft_port: None,
                scanning: false,
                forwarder: MyForwarder(forwarder),
                tracker: 0,
            };
            let relm = RelmApp::new(model);
            relm.run();
        } else {
            show_error(
                "Your local IP doesn't seem like it's an IPv4.\nIPv6 isn't supported in IGD yet.",
            );
        }
    } else {
        show_error("There was an error retrieving your public IP.");
    }
}
