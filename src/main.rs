use std::os::fd::OwnedFd;
use std::fs::File;
use std::io::Read;

use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
};
use wayland_client::{
    protocol::wl_seat,
    Connection, QueueHandle,
};

mod dispatch;




struct AppState {
    seat: Option<wl_seat::WlSeat>,
    data_device_manager: Option<ZwlrDataControlManagerV1>,
    data_device: Option<ZwlrDataControlDeviceV1>,
    data_source: Option<ZwlrDataControlSourceV1>,
    pipe_reader: Option<OwnedFd>,
    clipped: Option<Vec<u8>>
}

impl AppState {
    fn new() -> Self {
        AppState {
            seat: None,
            data_device_manager: None,
            data_device: None,
            data_source: None,
            pipe_reader: None,
            clipped: None,
        }
    }
    fn setup_data_device(&mut self, qh: &QueueHandle<AppState>) {
        if let (Some(seat), Some(data_device_manager)) = (&self.seat, &self.data_device_manager) {
            let data_device = data_device_manager.get_data_device(seat, qh, ());
            self.data_device = Some(data_device);
        }
    }
    fn setup_data_source(&mut self, qh: &QueueHandle<AppState>) {
        if let (Some(data_device), Some(data_device_manager)) =
            (&self.data_device, &self.data_device_manager)
        {
            let data_source = data_device_manager.create_data_source(qh, ());
            data_source.offer("text/plain;charset=utf-8".to_string());
            data_device.set_selection(Some(&data_source));
            self.data_source = Some(data_source);
        }
    }
}


fn main() {
    //Cant work without the connection...
    let conn = Connection::connect_to_env().expect("Failed to establish connection to the wayland server.");
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // Create an instance of AppState to manage state
    let mut app_state = AppState::new();

    // Get the registry object and start listening for globals
    let _registry = display.get_registry(&qh, ());
    event_queue.blocking_dispatch(&mut app_state).expect("Failed to dispatch starting events.");

    app_state.setup_data_device(&qh);
    app_state.setup_data_source(&qh);

    loop {
        event_queue.blocking_dispatch(&mut app_state).expect("Failed to dispatch events.");

        // After events completed, check if there was something written into the read end of the
        // selection.receive() pipe. 
        if let Some(reader) = &app_state.pipe_reader {
            match reader.try_clone(){
                Ok(fd) => {
                    let mut file = File::from(fd);
                    let mut buf = vec![];
                    if let Ok(_bytes) = file.read_to_end(&mut buf){
                        app_state.clipped = Some(buf);
                    };
                    app_state.pipe_reader = None;
                } 
                Err(_) => {}
            }
        }

    }
}
