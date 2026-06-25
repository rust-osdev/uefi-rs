// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::mem::memory_map::MemoryType;
use uefi::proto::dma::iommu::{EdkiiIommuAccess, EdkiiIommuAttribute, EdkiiIommuOperation, Iommu};

/// Runs the IOMMU protocol integration tests against the firmware-provided
/// protocol instance. These checks cover allocation, buffer access, mapping
/// operations, attributes, and representative buffer sizes.
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
    test_reject_oversized_mapping(&iommu);
}

/// Tests that the IOMMU protocol can allocate a one-page DMA buffer.
/// It verifies both the recorded page count and the byte size exposed by the
/// safe wrapper.
fn test_allocate_buffer(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    assert_eq!(buffer.pages(), pages);
    assert_eq!(buffer.size(), 4096);
}

/// Tests that an allocated DMA buffer can be accessed as a mutable byte slice.
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

/// Tests basic mapping operations on an allocated IOMMU buffer.
/// It verifies that the buffer can be mapped successfully for device access
/// and that the mapped size matches the requested size.
fn test_map_operations(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let mut buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    let buffer_size = buffer.size();
    let (_device_address, mut mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer,
            buffer_size,
        )
        .expect("Failed to map buffer for BUS_MASTER_READ");
    assert_eq!(mapped_bytes, buffer_size);

    let image_handle = boot::image_handle();
    match iommu.set_attribute(image_handle, &mut mapping, EdkiiIommuAccess::READ) {
        Ok(()) => {}
        Err(e) if e.status() == uefi::Status::UNSUPPORTED => {}
        Err(e) => panic!("set_attribute failed with unexpected error: {:?}", e),
    }

    drop(mapping);

    let (_device_address, mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_WRITE,
            &mut buffer,
            buffer_size,
        )
        .expect("Failed to map buffer for BUS_MASTER_WRITE");
    assert_eq!(mapped_bytes, buffer_size);
    drop(mapping);

    let (_device_address, _mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER,
            &mut buffer,
            buffer_size,
        )
        .expect("Failed to map buffer for BUS_MASTER_COMMON_BUFFER");
    assert_eq!(mapped_bytes, buffer_size);
}

/// Tests 64-bit mapping operations on an allocated IOMMU buffer.
/// This ensures the IOMMU correctly handles requests specifically requiring 64-bit device addresses.
fn test_map_64bit_operations(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let mut buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate IOMMU buffer");

    let buffer_size = buffer.size();
    let (_device_address, mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ64,
            &mut buffer,
            buffer_size,
        )
        .expect("Failed to map buffer for BUS_MASTER_READ64");
    assert_eq!(mapped_bytes, buffer_size);
    drop(mapping);

    let (_device_address, mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_WRITE64,
            &mut buffer,
            buffer_size,
        )
        .expect("Failed to map buffer for BUS_MASTER_WRITE64");
    assert_eq!(mapped_bytes, buffer_size);
    drop(mapping);

    let (_device_address, _mapping, mapped_bytes) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER64,
            &mut buffer,
            buffer_size,
        )
        .expect("Failed to map buffer for BUS_MASTER_COMMON_BUFFER64");
    assert_eq!(mapped_bytes, buffer_size);
}

/// Tests that mapping multiple distinct buffers yields unique device addresses.
/// This verifies the IOMMU allocator's ability to handle concurrent mappings
/// without address space collisions.
fn test_multiple_mappings(iommu: &Iommu) {
    let pages = 1;
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let mut buffer1 = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate buffer 1");

    let mut buffer2 = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate buffer 2");

    let mut buffer3 = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, attributes)
        .expect("Failed to allocate buffer 3");

    let buffer1_size = buffer1.size();
    let buffer2_size = buffer2.size();
    let buffer3_size = buffer3.size();

    let (addr1, mapping1, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer1,
            buffer1_size,
        )
        .expect("Failed to map buffer 1");

    let (addr2, mapping2, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_WRITE,
            &mut buffer2,
            buffer2_size,
        )
        .expect("Failed to map buffer 2");

    let (addr3, mapping3, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER,
            &mut buffer3,
            buffer3_size,
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

/// Tests allocation and mapping with several IOMMU memory attributes.
/// This verifies that cached, write-combine, and combined attributes can pass
/// through the safe API to firmware.
fn test_different_attributes(iommu: &Iommu) {
    let pages = 1;

    let mut buffer_cached = iommu
        .allocate_buffer(
            MemoryType::BOOT_SERVICES_DATA,
            pages,
            EdkiiIommuAttribute::MEMORY_CACHED,
        )
        .expect("Failed to allocate MEMORY_CACHED buffer");

    let mut buffer_wc = iommu
        .allocate_buffer(
            MemoryType::BOOT_SERVICES_DATA,
            pages,
            EdkiiIommuAttribute::MEMORY_WRITE_COMBINE,
        )
        .expect("Failed to allocate MEMORY_WRITE_COMBINE buffer");

    let combined_attrs =
        EdkiiIommuAttribute::MEMORY_CACHED | EdkiiIommuAttribute::DUAL_ADDRESS_CYCLE;
    let mut buffer_combined = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, pages, combined_attrs)
        .expect("Failed to allocate buffer with combined attributes");

    let buffer_cached_size = buffer_cached.size();
    let buffer_wc_size = buffer_wc.size();
    let buffer_combined_size = buffer_combined.size();

    let (_addr, _mapping, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer_cached,
            buffer_cached_size,
        )
        .expect("Failed to map cached buffer");

    let (_addr, _mapping, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer_wc,
            buffer_wc_size,
        )
        .expect("Failed to map write-combine buffer");

    let (_addr, _mapping, _) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer_combined,
            buffer_combined_size,
        )
        .expect("Failed to map combined attributes buffer");
}

/// Tests allocation and mapping for representative one-page and multi-page
/// buffers. It verifies that wrapper size calculations stay consistent across
/// different page counts.
fn test_multiple_buffer_sizes(iommu: &Iommu) {
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;

    let mut buffer_1pg = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 1, attributes)
        .expect("Failed to allocate 1-page buffer");
    assert_eq!(buffer_1pg.pages(), 1);
    assert_eq!(buffer_1pg.size(), 4096);

    let mut buffer_4pg = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 4, attributes)
        .expect("Failed to allocate 4-page buffer");
    assert_eq!(buffer_4pg.pages(), 4);
    assert_eq!(buffer_4pg.size(), 16384);

    let mut buffer_16pg = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 16, attributes)
        .expect("Failed to allocate 16-page buffer");
    assert_eq!(buffer_16pg.pages(), 16);
    assert_eq!(buffer_16pg.size(), 65536);

    let buffer_1pg_size = buffer_1pg.size();
    let buffer_4pg_size = buffer_4pg.size();
    let buffer_16pg_size = buffer_16pg.size();

    let (_, _mapping, mapped) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer_1pg,
            buffer_1pg_size,
        )
        .expect("Failed to map 1-page buffer");
    assert_eq!(mapped, buffer_1pg_size);

    let (_, _mapping, mapped) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer_4pg,
            buffer_4pg_size,
        )
        .expect("Failed to map 4-page buffer");
    assert_eq!(mapped, buffer_4pg_size);

    let (_, _mapping, mapped) = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer_16pg,
            buffer_16pg_size,
        )
        .expect("Failed to map 16-page buffer");
    assert_eq!(mapped, buffer_16pg_size);
}

/// Tests that the safe mapping API rejects lengths larger than the DMA buffer.
fn test_reject_oversized_mapping(iommu: &Iommu) {
    let attributes = EdkiiIommuAttribute::MEMORY_CACHED;
    let mut buffer = iommu
        .allocate_buffer(MemoryType::BOOT_SERVICES_DATA, 1, attributes)
        .expect("Failed to allocate IOMMU buffer");
    let oversized_len = buffer.size() + 1;

    let err = iommu
        .map(
            EdkiiIommuOperation::BUS_MASTER_READ,
            &mut buffer,
            oversized_len,
        )
        .expect_err("IOMMU map accepted an oversized buffer length");

    assert_eq!(err.status(), uefi::Status::BAD_BUFFER_SIZE);
}
