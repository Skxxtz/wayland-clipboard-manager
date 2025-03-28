use crate::AppState;
use nix::unistd::pipe;
use std::{fs::File, io::Write, os::fd::AsFd};
use wayland_client::{
    event_created_child,
    protocol::{
        wl_registry::{self, WlRegistry},
        wl_seat::WlSeat,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
    zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1},
};

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
                    state.data_device_manager =
                        Some(registry.bind::<ZwlrDataControlManagerV1, _, AppState>(
                            name,
                            version,
                            qh,
                            (),
                        ));
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
    ) {
    }
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _data_device_manager: &ZwlrDataControlManagerV1,
        _event: <ZwlrDataControlManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {
    }
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
                println!("{:?}", mime_type);
                state.mime_type = parse_mime(mime_type);
            }
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
                println!("{:?} - {}", state.mime_type, mime_type);
                if let Some(mime) = &state.mime_type {
                    if mime == mime_type.as_str() {
                        let mut file = File::from(fd);
                        match file.write_all(&clipboard_content) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }
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
                let Some(data_device) = state.data_device.as_ref() else {
                    return;
                };
                let Ok((reader, writer)) = pipe() else {
                    return;
                };

                if let Some(mime_type) = &state.mime_type {
                    selection.receive(mime_type.to_string(), writer.as_fd());
                    match conn.roundtrip() {
                        Ok(_) => {
                            state.pipe_reader = Some(reader);
                            data_device.set_selection(state.data_source.as_ref());
                            data_device.set_primary_selection(state.data_source.as_ref());
                        }
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

fn parse_mime(mime: String) -> Option<String> {
    let text_mimes = vec![
        "text/plain",
        "text/plain;charset=utf-8",
        "TEXT",
        "STRING",
        "UTF8_STRING",
        "COMPOUND_TEXT",
    ];
    if mime.as_str() == "text/html" {
        return Some(mime);
    } else if mime.contains("text/") || text_mimes.contains(&mime.as_str()) {
        return Some("text/plain;charset=utf-8".to_string());
    } else {
        return Some(mime);
    };
}

#[cfg(test)]
mod tests {
    use super::parse_mime;

    #[test]
    fn test_parse_mime() {
        // Text MIME types should return "text/plain;charset=utf-8"
        assert_eq!(parse_mime("text/plain".to_string()), Some("text/plain;charset=utf-8".to_string()));
        assert_eq!(parse_mime("TEXT".to_string()), Some("text/plain;charset=utf-8".to_string()));
        assert_eq!(parse_mime("UTF8_STRING".to_string()), Some("text/plain;charset=utf-8".to_string()));
        assert_eq!(parse_mime("text/markdown".to_string()), Some("text/plain;charset=utf-8".to_string()));

        // HTML should return as-is
        assert_eq!(parse_mime("text/html".to_string()), Some("text/html".to_string()));

        // Other MIME types should return as-is
        assert_eq!(parse_mime("image/png".to_string()), Some("image/png".to_string()));
        assert_eq!(parse_mime("application/json".to_string()), Some("application/json".to_string()));
    }
}
