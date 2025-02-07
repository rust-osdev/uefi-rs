// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot::{self, ScopedProtocol, SearchType};
use uefi::prelude::*;
use uefi::proto::driver::{ComponentName, ComponentName2, LanguageError, LanguageIter};
use uefi::{CStr16, Result};

#[allow(deprecated)]
use uefi::proto::driver::ComponentName1;

/// Generic interface for testing `ComponentName1`, `ComponentName2`, and
/// `ComponentName`.
trait ComponentNameInterface: Sized {
    fn open(handle: Handle) -> Result<Self>;
    fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError>;
    fn driver_name(&self, language: &str) -> Result<&CStr16>;
    fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16>;
}

#[allow(deprecated)]
impl ComponentNameInterface for ScopedProtocol<ComponentName1> {
    fn open(handle: Handle) -> Result<Self> {
        boot::open_protocol_exclusive::<ComponentName1>(handle)
    }

    fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError> {
        (**self).supported_languages()
    }

    fn driver_name(&self, language: &str) -> Result<&CStr16> {
        (**self).driver_name(language)
    }

    fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16> {
        (**self).controller_name(controller_handle, child_handle, language)
    }
}

impl ComponentNameInterface for ScopedProtocol<ComponentName2> {
    fn open(handle: Handle) -> Result<Self> {
        boot::open_protocol_exclusive::<ComponentName2>(handle)
    }

    fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError> {
        (**self).supported_languages()
    }

    fn driver_name(&self, language: &str) -> Result<&CStr16> {
        (**self).driver_name(language)
    }

    fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16> {
        (**self).controller_name(controller_handle, child_handle, language)
    }
}

impl ComponentNameInterface for ComponentName {
    fn open(handle: Handle) -> Result<Self> {
        Self::open(handle)
    }

    fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError> {
        self.supported_languages()
    }

    fn driver_name(&self, language: &str) -> Result<&CStr16> {
        self.driver_name(language)
    }

    fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16> {
        self.controller_name(controller_handle, child_handle, language)
    }
}

fn test_component_name<C: ComponentNameInterface>(english: &str) {
    let all_handles = boot::locate_handle_buffer(SearchType::AllHandles).unwrap();

    let fat_driver_name = cstr16!("FAT File System Driver");
    let fat_controller_name = cstr16!("FAT File System");

    // Find the FAT driver by name.
    let component_name: C = all_handles
        .iter()
        .find_map(|handle| {
            let component_name = C::open(*handle).ok()?;

            assert!(component_name
                .supported_languages()
                .ok()?
                .any(|lang| lang == english));

            let driver_name = component_name.driver_name(english).ok()?;
            if driver_name == fat_driver_name {
                Some(component_name)
            } else {
                None
            }
        })
        .expect("failed to find FAT driver");

    // Now check that the FAT controller can be found by name.
    all_handles
        .iter()
        .find(|handle| {
            let controller_name = if let Ok(controller_name) =
                component_name.controller_name(**handle, None, english)
            {
                controller_name
            } else {
                return false;
            };

            controller_name == fat_controller_name
        })
        .expect("failed to find FAT controller");
}

pub fn test() {
    info!("Running component name test");

    #[allow(deprecated)]
    test_component_name::<ScopedProtocol<ComponentName1>>("eng");
    test_component_name::<ScopedProtocol<ComponentName2>>("en");
    test_component_name::<ComponentName>("en");
}
