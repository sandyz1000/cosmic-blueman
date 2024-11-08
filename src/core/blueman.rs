use bluer::{Adapter, Device};
use futures::executor::block_on;
use libcosmic::prelude::*;
use libcosmic::widgets::{Button, Column, Label, List, Text};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
struct BluetoothDevice {
    name: String,
    address: String,
}

// TODO: Fix this
fn main() -> cosmic::Result<()> {
    cosmic::init()?;

    // Create a container to store available devices
    let devices = Rc::new(RefCell::new(Vec::<BluetoothDevice>::new()));

    // Main application window
    let app = cosmic::App::new("Bluetooth Manager", 800, 600)?;

    let adapter = block_on(init_bluetooth_adapter());

    let devices_clone = devices.clone();
    let scan_button = Button::new("Scan Devices", move || {
        // Scan for Bluetooth devices asynchronously
        let devices_list = block_on(scan_for_devices(&adapter));

        // Clear previous devices and populate the new ones
        let mut devices_mut = devices_clone.borrow_mut();
        devices_mut.clear();
        for device in devices_list {
            devices_mut.push(device);
        }
    });

    let devices_clone = devices.clone();
    let device_list = List::new(move || {
        // Access the devices list and display them in the UI
        devices_clone.borrow().iter().map(|device| {
            let connect_button = Button::new("Connect", {
                let address = device.address.clone();
                move || {
                    block_on(connect_to_device(&adapter, &address));
                }
            });
            let disconnect_button = Button::new("Disconnect", {
                let address = device.address.clone();
                move || {
                    block_on(disconnect_device(&adapter, &address));
                }
            });

            // Create a row for each device
            Column::new()
                .push(Label::new(format!(
                    "Device: {} ({})",
                    device.name, device.address
                )))
                .push(connect_button)
                .push(disconnect_button)
        })
    });

    // UI Layout
    let layout = Column::new()
        .push(Text::new("Bluetooth Device Manager"))
        .push(scan_button)
        .push(device_list);

    // Set the layout to the app and start the event loop
    app.set_layout(layout);
    app.run()?;

    Ok(())
}

async fn init_bluetooth_adapter() -> Adapter {
    let session = bluer::Session::new().await.unwrap();
    let adapter = session.default_adapter().await.unwrap();
    adapter.set_powered(true).await.unwrap();
    adapter
}

async fn scan_for_devices(adapter: &Adapter) -> Vec<BluetoothDevice> {
    let mut device_list = Vec::new();

    let device_stream = adapter.discover_devices().await.unwrap();
    futures::pin_mut!(device_stream);

    while let Some(device) = device_stream.next().await {
        if let Ok(device) = device {
            if let Some(name) = device.name().await.unwrap() {
                device_list.push(BluetoothDevice {
                    name,
                    address: device.address().to_string(),
                });
            }
        }
    }

    device_list
}

async fn connect_to_device(adapter: &Adapter, address: &str) {
    let device = adapter.device(address).unwrap();
    if device.is_connected().await.unwrap() {
        println!("Already connected");
    } else {
        println!("Connecting to {}", address);
        device.connect().await.unwrap();
        println!("Connected to {}", address);
    }
}

async fn disconnect_device(adapter: &Adapter, address: &str) {
    let device = adapter.device(address).unwrap();
    if device.is_connected().await.unwrap() {
        println!("Disconnecting from {}", address);
        device.disconnect().await.unwrap();
        println!("Disconnected from {}", address);
    } else {
        println!("Already disconnected");
    }
}
