use std:: {
    fs::File,
    io::Write,
    os::fd::AsFd,
};
use nix::{sys::stat, unistd::pipe};
use wayland_client::{
    event_created_child,
    protocol::{
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat}
    },
    Connection, QueueHandle,
    Dispatch, Proxy,
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
    zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1},
};
use crate::AppState;


impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == WlSeat::interface().name {
                    state.seat = Some(registry.bind::<WlSeat, _, AppState>(name, version, qh, ()));
                } else if interface == ZwlrDataControlManagerV1::interface().name {
                    state.data_device_manager = Some(registry
                        .bind::<ZwlrDataControlManagerV1, _, AppState>(name, version, qh, ()));
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlSeat, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _data_device_manager: &ZwlrDataControlManagerV1,
        _event: <ZwlrDataControlManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {}
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlOfferV1,
        event: <ZwlrDataControlOfferV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {
        match event {
            zwlr_data_control_offer_v1::Event::Offer { mime_type } => {
                state.mime_types.insert(mime_type);
            },
            _ => {}
        }
    }
}
impl Dispatch<ZwlrDataControlSourceV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlSourceV1,
        event: <ZwlrDataControlSourceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {
        match event {
            zwlr_data_control_source_v1::Event::Send { mime_type, fd } => {
                let clipboard_content = state.clipped.as_ref();
                println!("{}", mime_type);
                let mut file = File::from(fd);
                match file.write_all(&clipboard_content){
                    Ok(_) => {},
                    Err(_) => {},
                };
            }
            _ => {}
        }
    }
}
impl Dispatch<ZwlrDataControlDeviceV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _device: &ZwlrDataControlDeviceV1,
        event: <ZwlrDataControlDeviceV1 as Proxy>::Event,
        _data: &(),
        conn: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {
        match event {
            zwlr_data_control_device_v1::Event::Selection { id } => {
                let Some(selection) = id else { return };
                let Some(data_device) = state.data_device.as_ref() else { return; };
                let Ok((reader, writer)) = pipe() else { return; };

                if let Some(mime_type) = state.get_best_mimetype() {
                    selection.receive(mime_type, writer.as_fd());
                    match conn.roundtrip(){
                        Ok(_) => {
                            state.pipe_reader = Some(reader);
                            data_device.set_selection(state.data_source.as_ref());
                            data_device.set_primary_selection(state.data_source.as_ref());
                        },
                        Err(_) => {}
                    }
                }
            }
            zwlr_data_control_device_v1::Event::DataOffer { .. } => {}
            zwlr_data_control_device_v1::Event::Finished => {}
            _ => {}
        }

    }
    event_created_child!(AppState, ZwlrDataControlDeviceV1, [zwlr_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ZwlrDataControlOfferV1, ())]);
}
