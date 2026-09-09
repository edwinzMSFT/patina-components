//! Rust definitions corresponding to `MdeModulePkg/Bus/Usb/UsbBusDxe/UsbBus.h`.
//!
//! The executable bus implementation is still being migrated. These types keep
//! the private bus model and its timing constants explicit while preserving the
//! pointer-oriented relationships used by the UEFI driver.

#![allow(dead_code)]

use core::ffi::c_void;

use r_efi::{base::Boolean, efi};
use r_efi::efi::protocols::{device_path, usb_io};

pub const USB_MAX_LANG_ID: usize = 16;
pub const USB_MAX_INTERFACE: usize = 16;
pub const USB_MAX_DEVICES: usize = 128;

pub const USB_BUS_1_MILLISECOND: u64 = 1_000;
pub const USB_ROOTHUB_POLL_INTERVAL: u64 = 100 * 10_000;
pub const USB_HUB_POLL_INTERVAL: u64 = 64;
pub const USB_ENUM_POLL_MINIMUM_ATTEMPTS: u8 = 6;
pub const USB_ENUM_POLL_MAXIMUM_ATTEMPTS: u8 = 200;
pub const USB_WAIT_PORT_STABLE_STALL: u64 = 100 * USB_BUS_1_MILLISECOND;
pub const USB_WAIT_PORT_STS_CHANGE_STALL: u64 = 100;
pub const USB_SET_DEVICE_ADDRESS_STALL: u64 = 2 * USB_BUS_1_MILLISECOND;
pub const USB_RETRY_MAX_PACK_SIZE_STALL: u64 = 100 * USB_BUS_1_MILLISECOND;
pub const USB_SET_PORT_POWER_STALL: u64 = 2 * USB_BUS_1_MILLISECOND;
pub const USB_SET_PORT_RESET_STALL: u64 = 20 * USB_BUS_1_MILLISECOND;
pub const USB_SET_ROOT_PORT_RESET_STALL: u64 = 50 * USB_BUS_1_MILLISECOND;
pub const USB_SET_PORT_RECOVERY_STALL: u64 = 10 * USB_BUS_1_MILLISECOND;
pub const USB_CLR_ROOT_PORT_RESET_STALL: u64 = 20 * USB_BUS_1_MILLISECOND;
pub const USB_SET_ROOT_PORT_ENABLE_STALL: u64 = 20 * USB_BUS_1_MILLISECOND;
pub const USB_GENERAL_DEVICE_REQUEST_TIMEOUT: u32 = 500;
pub const USB_CLEAR_FEATURE_REQUEST_TIMEOUT: u32 = 10;

pub const USB_BUS_TPL: usize = 16;
pub const USB_US_LAND_ID: u16 = 0x0409;

pub const USB_INTERFACE_SIGNATURE: u64 = signature(b"USBI");
pub const USB_BUS_SIGNATURE: u64 = signature(b"USBB");
pub const DEVICE_PATH_LIST_ITEM_SIGNATURE: u64 = signature(b"dpli");

const fn signature(bytes: &[u8; 4]) -> u64 {
    u32::from_le_bytes(*bytes) as u64
}

pub const fn usb_bit(bit: u32) -> usize {
    1usize << bit
}

pub const fn usb_bit_is_set(data: usize, bit: usize) -> bool {
    data & bit == bit
}

/// Opaque descriptor types implemented by the descriptor parser.
pub enum UsbDeviceDesc {}
pub enum UsbConfigDesc {}
pub enum UsbInterfaceDesc {}
pub enum UsbInterfaceSetting {}
pub enum UsbEndpointDesc {}
pub enum UsbHubApi {}

#[repr(C)]
pub struct EfiUsbBusProtocol {
    pub reserved: u64,
}

#[repr(C)]
pub struct UsbDevice {
    pub bus: *mut UsbBus,
    pub speed: u8,
    pub address: u8,
    pub max_packet0: u32,
    pub dev_desc: *mut UsbDeviceDesc,
    pub active_config: *mut UsbConfigDesc,
    pub lang_id: [u16; USB_MAX_LANG_ID],
    pub total_lang_id: u16,
    pub num_of_interface: u8,
    pub interfaces: [*mut UsbInterface; USB_MAX_INTERFACE],
    pub translator: *mut c_void,
    pub parent_addr: u8,
    pub parent_if: *mut UsbInterface,
    pub parent_port: u8,
    pub tier: u8,
    pub connected: Boolean,
    pub disconnect_fail: Boolean,
}

#[repr(C)]
pub struct UsbInterface {
    pub signature: usize,
    pub device: *mut UsbDevice,
    pub if_desc: *mut UsbInterfaceDesc,
    pub if_setting: *mut UsbInterfaceSetting,
    pub handle: efi::Handle,
    pub usb_io: usb_io::Protocol,
    pub device_path: *mut device_path::Protocol,
    pub is_managed: Boolean,
    pub is_hub: Boolean,
    pub hub_api: *mut UsbHubApi,
    pub num_of_port: u8,
    pub hub_notify: efi::Event,
    pub hub_ep: *mut UsbEndpointDesc,
    pub change_map: *mut u8,
    pub max_speed: u8,
    pub poll_count: u8,
}

#[repr(C)]
pub struct UsbBus {
    pub signature: usize,
    pub bus_id: EfiUsbBusProtocol,
    pub host_handle: efi::Handle,
    pub device_path: *mut device_path::Protocol,
    pub usb2_hc: *mut c_void,
    pub max_devices: u32,
    pub devices: [*mut UsbDevice; 256],
    pub wanted_usb_io_dp_list: *mut c_void,
}

#[repr(C)]
pub struct DevicePathListItem {
    pub signature: usize,
    pub link: *mut c_void,
    pub device_path: *mut device_path::Protocol,
}

#[repr(C)]
pub struct UsbClassFormatDevicePath {
    pub usb_class: [u8; 32],
    pub end: device_path::Protocol,
}
