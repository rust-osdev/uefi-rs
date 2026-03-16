// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::mem::memory_map::MemoryType;
use uefi::proto::dma::iommu::{EdkiiIommuAccess, EdkiiIommuAttribute, EdkiiIommuOperation, Iommu};

pub fn test() {
    info!("Running IOMMU protocol test");

    let handle =
        boot::get_handle_for_protocol::<Iommu>().expect("Failed to get IOMMU protocol handle");
    let iommu =
        boot::open_protocol_exclusive::<Iommu>(handle).expect("Failed to open IOMMU protocol");

    let revision = iommu.revision();
    info!("Revision: {revision:#x}");

    test_allocate_buffer(&iommu);
    test_buffer_read_write(&iommu);
    test_map_operations(&iommu);
    test_map_64bit_operations(&iommu);
    test_multiple_mappings(&iommu);
    test_different_attributes(&iommu);
    test_multiple_buffer_sizes(&iommu);
}

fn test_allocate_buffer(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    assert_eq!(buffer.pages(), pages);
    assert_eq!(buffer.size(), 4096);
}

fn test_buffer_read_write(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let mut buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }

    for (i, byte) in buffer.iter().enumerate() {
        assert_eq!(*byte, (i & 0xFF) as u8, "Buffer mismatch at offset {}", i);
    }

    buffer[0] = 0xAA;
    buffer[100] = 0xBB;
    buffer[4095] = 0xCC;

    assert_eq!(buffer[0], 0xAA);
    assert_eq!(buffer[100], 0xBB);
    assert_eq!(buffer[4095], 0xCC);
}

fn test_map_operations(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    let (_device_address, mapping, mapped_bytes) = iommu
        .map(EdkiiIommuOperation::BUS_MASTER_READ, &buffer, buffer.size())
        .expect("Failed to map buffer for BUS_MASTER_READ");
    assert_eq!(mapped_bytes, buffer.size());

    let image_handle = boot::image_handle();
    match iommu.set_attribute(image_handle, &mapping, EdkiiIommuAccess::READ) {
        Ok(()) => {}
        Err(e) if e.status() == uefi::Status::UNSUPPORTED => {}
        Err(e) => panic!("set_attribute failed with unexpected error: {:?}", e),
    }

    drop(mapping);

    let (_device_address, mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_WRITE,
            &buffer,
            buffer.size(),
        )
        .expect("Failed to map buffer for BUS_MASTER_WRITE");
    assert_eq!(mapped_bytes, buffer.size());
    drop(mapping);

    let (_device_address, _mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER,
            &buffer,
            buffer.size(),
        )
        .expect("Failed to map buffer for BUS_MASTER_COMMON_BUFFER");
    assert_eq!(mapped_bytes, buffer.size());
}

fn test_map_64bit_operations(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    let (_device_address, mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ64,
            &buffer,
            buffer.size(),
        )
        .expect("Failed to map buffer for BUS_MASTER_READ64");
    assert_eq!(mapped_bytes, buffer.size());
    drop(mapping);

    let (_device_address, mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_WRITE64,
            &buffer,
            buffer.size(),
        )
        .expect("Failed to map buffer for BUS_MASTER_WRITE64");
    assert_eq!(mapped_bytes, buffer.size());
    drop(mapping);

    let (_device_address, _mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER64,
            &buffer,
            buffer.size(),
        )
        .expect("Failed to map buffer for BUS_MASTER_COMMON_BUFFER64");
    assert_eq!(mapped_bytes, buffer.size());
}

fn test_multiple_mappings(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let buffer1 = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate buffer 1");

    let buffer2 = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate buffer 2");

    let buffer3 = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate buffer 3");

    let (addr1, mapping1, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer1,
            buffer1.size(),
        )
        .expect("Failed to map buffer 1");

    let (addr2, mapping2, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_WRITE,
            &buffer2,
            buffer2.size(),
        )
        .expect("Failed to map buffer 2");

    let (addr3, mapping3, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER,
            &buffer3,
            buffer3.size(),
        )
        .expect("Failed to map buffer 3");

    assert_ne!(
        addr1, addr2,
        "Mapping 1 and 2 should have different addresses"
    );
    assert_ne!(
        addr2, addr3,
        "Mapping 2 and 3 should have different addresses"
    );
    assert_ne!(
        addr1, addr3,
        "Mapping 1 and 3 should have different addresses"
    );

    drop(mapping1);
    drop(mapping2);
    drop(mapping3);
}

fn test_different_attributes(iommu: &Iommu) {
    let pages = 1;

    let buffer_cached = iommu
        .allocate_buffer(
            MemoryType::BOOT_SERVICES_DATA,
            pages,
            EdkiiIommuAttribute::MEMORY_CACHED,
        )
        .expect("Failed to allocate MEMORY_CACHED buffer");

    let buffer_wc = iommu
        .allocate_buffer(
            MemoryType::BOOT_SERVICES_DATA,
            pages,
            EdkiiIommuAttribute::MEMORY_WRITE_COMBINE,
        )
        .expect("Failed to allocate MEMORY_WRITE_COMBINE buffer");

    let combined_attrs =
        EdkiiIommuAttribute::MEMORY_CACHED | EdkiiIommuAttribute::DUAL_ADDRESS_CYCLE;
    let buffer_combined = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, combined_attrs)
        .expect("Failed to allocate buffer with combined attributes");

    let (_addr, _mapping, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer_cached,
            buffer_cached.size(),
        )
        .expect("Failed to map cached buffer");

    let (_addr, _mapping, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer_wc,
            buffer_wc.size(),
        )
        .expect("Failed to map write-combine buffer");

    let (_addr, _mapping, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer_combined,
            buffer_combined.size(),
        )
        .expect("Failed to map combined attributes buffer");
}

fn test_multiple_buffer_sizes(iommu: &Iommu) {
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let buffer_1pg = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 1, attributes)
        .expect("Failed to allocate 1-page buffer");
    assert_eq!(buffer_1pg.pages(), 1);
    assert_eq!(buffer_1pg.size(), 4096);

    let buffer_4pg = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 4, attributes)
        .expect("Failed to allocate 4-page buffer");
    assert_eq!(buffer_4pg.pages(), 4);
    assert_eq!(buffer_4pg.size(), 16384);

    let buffer_16pg = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 16, attributes)
        .expect("Failed to allocate 16-page buffer");
    assert_eq!(buffer_16pg.pages(), 16);
    assert_eq!(buffer_16pg.size(), 65536);

    let (_, _mapping, mapped) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer_1pg,
            buffer_1pg.size(),
        )
        .expect("Failed to map 1-page buffer");
    assert_eq!(mapped, buffer_1pg.size());

    let (_, _mapping, mapped) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer_4pg,
            buffer_4pg.size(),
        )
        .expect("Failed to map 4-page buffer");
    assert_eq!(mapped, buffer_4pg.size());

    let (_, _mapping, mapped) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &buffer_16pg,
            buffer_16pg.size(),
        )
        .expect("Failed to map 16-page buffer");
    assert_eq!(mapped, buffer_16pg.size());
}
