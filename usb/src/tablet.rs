// Copyright (c) 2022 Huawei Technologies Co.,Ltd. All rights reserved.
//
// StratoVirt is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//         http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

use std::sync::{Arc, Mutex, Weak};

use log::{debug, error, info};
use once_cell::sync::Lazy;

use crate::config::*;
use crate::descriptor::{UsbConfigDescriptor, UsbDeviceDescriptor, UsbEndpointDescriptor};
use crate::descriptor::{UsbDescriptorOps, UsbInterfaceDescriptor};
use crate::hid::QUEUE_MASK;
use crate::hid::{HidType, QUEUE_LENGTH};
use crate::usb::{notify_controller, UsbDeviceRequest};
use crate::{
    hid::{
        Hid, DESC_STRINGS, STR_CONFIG_TABLET, STR_MANUFACTURER, STR_PRODUCT_TABLET,
        STR_SERIAL_TABLET,
    },
    usb::{
        usb_endpoint_init, UsbDesc, UsbDescConfig, UsbDescDevice, UsbDescEndpoint, UsbDescIface,
        UsbDescOther, UsbDevice, UsbDeviceOps, UsbEndpoint, UsbPacket, UsbPacketStatus,
    },
    xhci::xhci_controller::XhciDevice,
};
use anyhow::Result;

const INPUT_BUTTON_WHEEL_UP: u32 = 0x08;
const INPUT_BUTTON_WHEEL_DOWN: u32 = 0x10;

/// USB Tablet Descriptor
static DESC_TABLET: Lazy<Arc<UsbDesc>> = Lazy::new(|| {
    let s = DESC_STRINGS.iter().map(|&s| s.to_string()).collect();
    Arc::new(UsbDesc {
        full_dev: Some(DESC_DEVICE_TABLET.clone()),
        high_dev: None,
        super_dev: None,
        strings: s,
    })
});
/// Tablet device descriptor
static DESC_DEVICE_TABLET: Lazy<Arc<UsbDescDevice>> = Lazy::new(|| {
    Arc::new(UsbDescDevice {
        device_desc: UsbDeviceDescriptor {
            bLength: USB_DT_DEVICE_SIZE,
            bDescriptorType: USB_DT_DEVICE,
            idVendor: 0x0627,
            idProduct: 0x0001,
            bcdDevice: 0,
            iManufacturer: STR_MANUFACTURER,
            iProduct: STR_PRODUCT_TABLET,
            iSerialNumber: STR_SERIAL_TABLET,
            bcdUSB: 0x0100,
            bDeviceClass: 0,
            bDeviceSubClass: 0,
            bDeviceProtocol: 0,
            bMaxPacketSize0: 8,
            bNumConfigurations: 1,
        },
        confs: vec![Arc::new(UsbDescConfig {
            config_desc: UsbConfigDescriptor {
                bLength: USB_DT_CONFIG_SIZE,
                bDescriptorType: USB_DT_CONFIGURATION,
                wTotalLength: 0,
                bNumInterfaces: 1,
                bConfigurationValue: 1,
                iConfiguration: STR_CONFIG_TABLET,
                bmAttributes: USB_CONFIGURATION_ATTR_ONE | USB_CONFIGURATION_ATTR_REMOTE_WAKEUP,
                bMaxPower: 50,
            },
            if_groups: Vec::new(),
            ifs: vec![DESC_IFACE_TABLET.clone()],
        })],
    })
});
/// Tablet interface descriptor
static DESC_IFACE_TABLET: Lazy<Arc<UsbDescIface>> = Lazy::new(|| {
    Arc::new(UsbDescIface {
        interface_desc: UsbInterfaceDescriptor {
            bLength: USB_DT_INTERFACE_SIZE,
            bDescriptorType: USB_DT_INTERFACE,
            bInterfaceNumber: 0,
            bAlternateSetting: 0,
            bNumEndpoints: 1,
            bInterfaceClass: USB_CLASS_HID,
            bInterfaceSubClass: 0,
            bInterfaceProtocol: 0,
            iInterface: 0,
        },
        other_desc: vec![Arc::new(UsbDescOther {
            length: 0,
            /// HID descriptor
            data: vec![0x09, 0x21, 0x01, 0x0, 0x0, 0x01, 0x22, 74, 0x0],
        })],
        eps: vec![Arc::new(UsbDescEndpoint {
            endpoint_desc: UsbEndpointDescriptor {
                bLength: USB_DT_ENDPOINT_SIZE,
                bDescriptorType: USB_DT_ENDPOINT,
                bEndpointAddress: USB_DIRECTION_DEVICE_TO_HOST as u8 | 0x1,
                bmAttributes: USB_ENDPOINT_ATTR_INT,
                wMaxPacketSize: 8,
                bInterval: 0xa,
            },
            extra: None,
        })],
    })
});

/// USB tablet device.
pub struct UsbTablet {
    id: String,
    usb_device: UsbDevice,
    hid: Hid,
    /// USB controller used to notify controller to transfer data.
    ctrl: Option<Weak<Mutex<XhciDevice>>>,
}

impl UsbTablet {
    pub fn new(id: String) -> Self {
        Self {
            id,
            usb_device: UsbDevice::new(),
            hid: Hid::new(HidType::Tablet),
            ctrl: None,
        }
    }

    pub fn realize(mut self) -> Result<Arc<Mutex<Self>>> {
        self.usb_device.product_desc = String::from("StratoVirt USB Tablet");
        self.usb_device.strings = Vec::new();
        let tablet = Arc::new(Mutex::new(self));
        let cloned_tablet = tablet.clone();
        usb_endpoint_init(&(tablet as Arc<Mutex<dyn UsbDeviceOps>>));
        let mut locked_tablet = cloned_tablet.lock().unwrap();
        locked_tablet.usb_device.usb_desc = Some(DESC_TABLET.clone());
        locked_tablet.usb_device.init_descriptor()?;
        drop(locked_tablet);
        Ok(cloned_tablet)
    }
}

// Used for VNC to send pointer event.
pub fn pointer_event(tablet: &Arc<Mutex<UsbTablet>>, button: u32, x: i32, y: i32) -> Result<()> {
    let mut locked_tablet = tablet.lock().unwrap();
    if locked_tablet.hid.num >= QUEUE_LENGTH {
        debug!("Pointer queue is full!");
        // Return ok to ignore the request.
        return Ok(());
    }
    let index = ((locked_tablet.hid.head + locked_tablet.hid.num) & QUEUE_MASK) as usize;
    let mut evt = &mut locked_tablet.hid.pointer.queue[index];
    if button == INPUT_BUTTON_WHEEL_UP {
        evt.pos_z += 1;
    } else if button == INPUT_BUTTON_WHEEL_DOWN {
        evt.pos_z -= 1;
    }
    evt.button_state = button;
    evt.pos_x = x;
    evt.pos_y = y;
    locked_tablet.hid.num += 1;
    drop(locked_tablet);
    let clone_tablet = tablet.clone();
    notify_controller(&(clone_tablet as Arc<Mutex<dyn UsbDeviceOps>>))
}

impl UsbDeviceOps for UsbTablet {
    fn reset(&mut self) {
        info!("Tablet device reset");
        self.usb_device.remote_wakeup &= !USB_DEVICE_REMOTE_WAKEUP;
        self.usb_device.addr = 0;
        self.hid.reset();
    }

    fn handle_control(&mut self, packet: &mut UsbPacket, device_req: &UsbDeviceRequest) {
        debug!("handle_control request {:?}", device_req);
        match self
            .usb_device
            .handle_control_for_descriptor(packet, device_req)
        {
            Ok(handled) => {
                if handled {
                    debug!("Tablet control handled by descriptor, return directly.");
                    return;
                }
            }
            Err(e) => {
                error!("Tablet descriptor error {}", e);
                packet.status = UsbPacketStatus::Stall;
                return;
            }
        }
        self.hid
            .handle_control_packet(packet, device_req, &mut self.usb_device.data_buf);
    }

    fn handle_data(&mut self, p: &mut UsbPacket) {
        self.hid.handle_data_packet(p);
    }

    fn device_id(&self) -> String {
        self.id.clone()
    }

    fn get_usb_device(&self) -> &UsbDevice {
        &self.usb_device
    }

    fn get_mut_usb_device(&mut self) -> &mut UsbDevice {
        &mut self.usb_device
    }

    fn set_controller(&mut self, ctrl: Weak<Mutex<XhciDevice>>) {
        self.ctrl = Some(ctrl);
    }

    fn get_controller(&self) -> Option<Weak<Mutex<XhciDevice>>> {
        self.ctrl.clone()
    }

    fn get_wakeup_endpoint(&self) -> Arc<Mutex<UsbEndpoint>> {
        self.usb_device.get_endpoint(true, 1)
    }
}
