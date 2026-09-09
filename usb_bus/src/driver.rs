//! USB Bus driver binding implementation.
//!
//! The [`UsbBusDriver`] implements [`patina::driver_binding::DriverBinding`] to
//! manage the USB Bus. It produces USB IO.
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
//use core::{ffi::c_void, ptr{self, NonNull}};
use core::{ptr::NonNull};

//use device_path::EfiDevPathPtr;
//use r_efi::{efi, efi::protocols::usb_io, protocols::device_path::Protocol as EfiDevicePathProtocol};
use r_efi::{efi, protocols::device_path::Protocol as EfiDevicePathProtocol};
//use usb_2_host_controller::{Protocol, UsbPortFeature, UsbPortStatus};

use patina::{
    pi::{
        protocol::status_code,
        status_code::{EFI_PROGRESS_CODE, EFI_IO_BUS_USB, EFI_IOB_PC_INIT},
    },
    uefi::{
        boot_services::BootServices,
        driver_binding::DriverBinding,
    },
};

//use patina::vendor_protocols::hid_io;
//use crate::{control_transfers, descriptors, device::UsbBusDevice, hid_io_impl, interrupt_transfers, usb_hid_defs::*};

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

        if let Some(remaining_device_path) = remaining_device_path {
            let remaining_device_path = remaining_device_path.as_ptr();
            let is_end_device_path = unsafe {
                (*remaining_device_path).r#type == device_path::TYPE_END
                    && (*remaining_device_path).sub_type == device_path::END_ENTIRE_DEVICE_PATH_SUBTYPE
            };
            if !is_end_device_path {
                let device_path_node = unsafe { &*remaining_device_path };
                if device_path_node.r#type != device_path::TYPE_MESSAGING
                    || (device_path_node.sub_type != device_path::MSG_USB_DP
                        && device_path_node.sub_type != device_path::MSG_USB_CLASS_DP
                        && device_path_node.sub_type != device_path::MSG_USB_WWID_DP)
                {
                    return Ok(false);
                }
            }
        }

        if let Err(status) = unsafe {
            boot_services.open_protocol::<usb_2_host_controller::Protocol>(
                controller,
                self.agent,
                controller,
                efi::OPEN_PROTOCOL_BY_DRIVER,
            )
        } {
            if status == efi::Status::ALREADY_STARTED {
                return Ok(true);
            } else {
                return Err(status);
            }
        };

        boot_services.close_protocol(controller, &usb_2_host_controller::PROTOCOL_GUID, self.agent, controller).ok();

        if let Err(status) = unsafe {
            boot_services.open_protocol::<device_path::Protocol>(
                controller,
                self.agent,
                controller,
                efi::OPEN_PROTOCOL_BY_DRIVER,
            )
        } {
            if status == efi::Status::ALREADY_STARTED {
                return Ok(true);
            } else {
                return Err(status);
            }
        };

        boot_services.close_protocol(controller, &device_path::PROTOCOL_GUID, self.agent, controller).ok();

        Ok(true)
    }

    /// Starts USB Bus support for the given controller.
    fn driver_binding_start<U: BootServices + 'static>(
        &mut self,
        boot_services: &'static U,
        controller: efi::Handle,
        remaining_device_path: Option<NonNull<EfiDevicePathProtocol>>,
    ) -> Result<(), efi::Status> {
        log::trace!("USB Bus: driver_binding_start on controller {:?}", controller);

        // The C driver obtains the parent device path before initializing the bus.
        // The Rust implementation currently has no status-code reporting service,
        // so the protocol lookup is retained while its value is intentionally unused.
        let _parent_device_path = unsafe {
            boot_services.open_protocol::<device_path::Protocol>(
                controller,
                self.agent,
                controller,
                efi::OPEN_PROTOCOL_GET_PROTOCOL,
            )
        }?;

        // SAFETY: `p` is the only mutable reference to the `StatusCodeRuntimeProtocol` in this scope.
        let Ok(p) = (unsafe { boot_services.locate_protocol::<status_code::StatusCodeProtocol>(None) }) else {
            log::error!("Performance: Fail to find status code protocol.");
            return Err(efi::Status::NOT_FOUND);
        };

        let _status = p.report_status_code(
            EFI_PROGRESS_CODE,
            EFI_IO_BUS_USB | EFI_IOB_PC_INIT,
            0,
            patina::guid::CALLER_ID.as_efi_guid(),
        )?;
        // report_status_code

        let bus_protocol_exists = unsafe {
            boot_services
                .open_protocol::<crate::usb_bus_defs::EfiUsbBusProtocol>(
                    controller,
                    self.agent,
                    controller,
                    efi::OPEN_PROTOCOL_GET_PROTOCOL,
                )
                .is_ok()
        };

        if bus_protocol_exists {
            if remaining_device_path.is_some_and(|path| unsafe {
                (*path.as_ptr()).r#type == device_path::TYPE_END
                    && (*path.as_ptr()).sub_type == device_path::END_ENTIRE_DEVICE_PATH_SUBTYPE
            }) {
                return Ok(());
            }

            // Wanted-device-path storage and recursive child connection are not
            // implemented in the current Rust bus model yet.
            log::debug!("USB Bus: existing bus requires child connection handling");
            return Ok(());
        }

        // This is the Rust equivalent of the first-start portion of
        // UsbBusBuildProtocol. The full host-controller enumeration is still
        // pending, but installing the bus protocol establishes the controller's
        // started state for subsequent binding calls.
        boot_services.install_protocol_interface(
            Some(controller),
            Box::new(crate::usb_bus_defs::EfiUsbBusProtocol { reserved: 0 }),
        )?;

        Ok(())
    }

    /// Stops USB Bus support for the given controller.
    fn driver_binding_stop<U: BootServices + 'static>(
        &mut self,
        _boot_services: &'static U,
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