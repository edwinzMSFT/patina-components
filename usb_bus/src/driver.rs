//! USB Bus driver binding implementation.
//!
//! The [`UsbBusDriver`] implements [`patina::driver_binding::DriverBinding`] to
//! manage USB Bus devices. It consumes USB IO and produces HidIo.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

// possibly upstream these to r_efi
#[path = "../../protocols/device_path.rs"]
mod device_path;

#[path = "../../protocols/usb_2_host_controller.rs"]
mod usb_2_host_controller;

use alloc::boxed::Box;
use core::{ffi::c_void, ptr::NonNull};

use device_path::EfiDevPathPtr;
use usb_2_host_controller::{Protocol, UsbPortFeature, UsbPortStatus};
use r_efi::{efi, efi::protocols::usb_io, protocols::device_path::Protocol as EfiDevicePathProtocol};

use patina::{boot_services::BootServices, driver_binding::DriverBinding};

//use patina::vendor_protocols::hid_io;

//use crate::{control_transfers, descriptors, device::UsbBusDevice, hid_io_impl, interrupt_transfers, usb_hid_defs::*};
use patina::boot_services::event::EventTimerType;

/// USB Bus driver that implements [`DriverBinding`].
pub struct UsbBusDriver {
    agent: efi::Handle,
}

impl UsbBusDriver {
    /// Creates a new USB Bus driver bound to the given agent handle.
    pub fn new(agent: efi::Handle) -> Self {
        Self { agent }
    }
}

// efi::Handle is an opaque *mut c_void that is never actually dereferenced as a pointer.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl DriverBinding for UsbBusDriver {
    /// Tests if the given controller has USB IO with HID interface class.
    #[coverage(off)]
    fn driver_binding_supported<U: BootServices + 'static>(
        &self,
        boot_services: &'static U,
        controller: efi::Handle,
        remaining_device_path: Option<NonNull<EfiDevicePathProtocol>>,
    ) -> Result<bool, efi::Status> {
        // SAFETY: usb_2_host_controller::Protocol layout matches the USB 2.0 Host Controller GUID.
        let usb_2_host_controller = match unsafe {
            boot_services.open_protocol::<usb_2_host_controller::Protocol>(
                controller,
                self.agent,
                controller,
                efi::OPEN_PROTOCOL_BY_DRIVER,
            )
        } {
            Ok(usb_2_host_controller) => usb_2_host_controller,
            Err(_) => return Ok(false),
        };

        let result = true;

        boot_services.close_protocol(controller, &usb_2_host_controller::PROTOCOL_GUID, self.agent, controller).ok();

        Ok(result)
    }

    /// Starts USB Bus support for the given controller.
    fn driver_binding_start<U: BootServices + 'static>(
        &mut self,
        boot_services: &'static U,
        controller: efi::Handle,
        _remaining_device_path: Option<NonNull<EfiDevicePathProtocol>>,
    ) -> Result<(), efi::Status> {
        log::trace!("USB Bus: driver_binding_start on controller {:?}", controller);

        Ok(())
    }

    /// Stops USB Bus support for the given controller.
    fn driver_binding_stop<U: BootServices + 'static>(
        &mut self,
        boot_services: &'static U,
        controller: efi::Handle,
        _number_of_children: usize,
        _child_handle_buffer: Option<NonNull<efi::Handle>>,
    ) -> Result<(), efi::Status> {
        log::trace!("USB Bus: driver_binding_stop on controller {:?}", controller);

        Ok(())
    }
}
/*
impl UsbBusDriver {
    fn close_usb_io(&self, boot_services: &impl BootServices, controller: efi::Handle) {
        if let Err(status) = boot_services.close_protocol(controller, &usb_io::PROTOCOL_GUID, self.agent, controller) {
            log::error!("USB Bus: error closing USB IO protocol: {status:x?}");
        }
    }
}
*/
