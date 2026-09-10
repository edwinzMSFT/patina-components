//! USB descriptor model translated from `UsbDesc.h`.

#![allow(dead_code)]

use r_efi::industry::usb::{ConfigDescriptor, DeviceDescriptor, EndpointDescriptor, InterfaceDescriptor};

/// Maximum number of alternate settings retained for one USB interface.
pub const USB_MAX_INTERFACE_SETTING: usize = 256;

pub const USB_DESC_TYPE_DEVICE: u8 = 0x01;
pub const USB_DESC_TYPE_CONFIG: u8 = 0x02;
pub const USB_DESC_TYPE_STRING: u8 = 0x03;
pub const USB_DESC_TYPE_INTERFACE: u8 = 0x04;
pub const USB_DESC_TYPE_ENDPOINT: u8 = 0x05;

pub const USB_REQ_TYPE_STANDARD: usize = 0x00;
pub const USB_TARGET_DEVICE: usize = 0x00;
pub const USB_REQ_GET_DESCRIPTOR: usize = 0x06;
pub const USB_REQ_SET_ADDRESS: usize = 0x05;
pub const USB_REQ_SET_CONFIG: usize = 0x09;
pub const USB_REQ_CLEAR_FEATURE: usize = 0x01;

/// USB data direction bit used by the standard request-type encoding.
pub const USB_DATA_IN: u32 = 0x01;

/// Builds the `bmRequestType` byte from direction, request type, and target.
pub const fn usb_request_type(direction_is_in: bool, request_type: usize, target: usize) -> u8 {
    (((if direction_is_in { USB_DATA_IN } else { 0 }) << 7) as usize | request_type | target) as u8
}

/// Common two-byte USB descriptor header.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DescriptorHeader {
    pub length: u8,
    pub descriptor_type: u8,
}

/// Endpoint descriptor plus the data toggle used by transfers.
#[repr(C)]
pub struct UsbEndpointDesc {
    pub descriptor: EndpointDescriptor,
    pub toggle: u8,
}

/// Interface alternate setting and its endpoint descriptors.
#[repr(C)]
pub struct UsbInterfaceSetting {
    pub descriptor: InterfaceDescriptor,
    pub endpoints: *mut *mut UsbEndpointDesc,
}

/// All alternate settings belonging to one interface.
#[repr(C)]
pub struct UsbInterfaceDesc {
    pub settings: [*mut UsbInterfaceSetting; USB_MAX_INTERFACE_SETTING],
    pub num_of_setting: usize,
    pub active_index: usize,
}

/// Configuration descriptor and its interface descriptors.
#[repr(C)]
pub struct UsbConfigDesc {
    pub descriptor: ConfigDescriptor,
    pub interfaces: *mut *mut UsbInterfaceDesc,
}

/// Device descriptor and its configuration descriptors.
#[repr(C)]
pub struct UsbDeviceDesc {
    pub descriptor: DeviceDescriptor,
    pub configs: *mut *mut UsbConfigDesc,
}
