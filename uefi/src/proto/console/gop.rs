// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graphics output protocol.
//!
//! The UEFI GOP is meant to replace existing [VGA][vga] hardware interfaces.
//!
//! The GOP provides access to a hardware frame buffer and allows UEFI apps
//! to draw directly to the graphics output device.
//!
//! The advantage of the GOP over legacy VGA is that it allows multiple GPUs
//! to exist and be used on the system. There is a GOP implementation for every
//! unique GPU in the system which supports UEFI.
//!
//! [vga]: https://en.wikipedia.org/wiki/Video_Graphics_Array
//!
//! # Definitions
//!
//! All graphics operations use a coordinate system where the top-left of the screen
//! is mapped to the point (0, 0), and `y` increases going down.
//!
//! Rectangles are defined by their top-left corner, and their width and height.
//!
//! The stride is understood as the length in bytes of a scan line / row of a buffer.
//! In theory, a buffer with a width of 640 should have (640 * 4) bytes per row,
//! but in practice there might be some extra padding used for efficiency.
//!
//! Frame buffers represent the graphics card's image buffers, backing the displays.
//!
//! Blits (**bl**ock **t**ransfer) can do high-speed memory copy between
//! the frame buffer and itself, or to and from some other buffers.
//!
//! # Blitting
//!
//! On certain hardware, the frame buffer is in a opaque format,
//! or cannot be accessed by the CPU. In those cases, it is not possible
//! to draw directly to the frame buffer. You must draw to another buffer
//! with a known pixel format, and then submit a blit command to copy that buffer
//! into the back buffer.
//!
//! Blitting can also copy a rectangle from the frame buffer to
//! another rectangle in the frame buffer, or move data out of the frame buffer
//! into a CPU-visible buffer. It can also do very fast color fills.
//!
//! The source and destination rectangles must always be of the same size:
//! no stretching / squashing will be done.
//!
//! # Animations
//!
//! UEFI does not mention if double buffering is used, nor how often
//! the frame buffer gets sent to the screen, but it's safe to assume that
//! the graphics card will re-draw the buffer at around the monitor's refresh rate.
//! You will have to implement your own double buffering if you want to
//! avoid tearing with animations.

use crate::proto::unsafe_protocol;
use crate::util::usize_from_u32;
use crate::{boot, Result, StatusExt};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use uefi_raw::protocol::console::{
    GraphicsOutputBltOperation, GraphicsOutputModeInformation, GraphicsOutputProtocol,
    GraphicsOutputProtocolMode,
};

pub use uefi_raw::protocol::console::PixelBitmask;

/// Provides access to the video hardware's frame buffer.
///
/// The GOP can be used to set the properties of the frame buffer,
/// and also allows the app to access the in-memory buffer.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(GraphicsOutputProtocol::GUID)]
pub struct GraphicsOutput(GraphicsOutputProtocol);

impl GraphicsOutput {
    /// Returns information for an available graphics mode that the graphics
    /// device and the set of active video output devices supports.
    fn query_mode(&self, index: u32) -> Result<Mode> {
        let mut info_sz = 0;
        let mut info_heap_ptr = ptr::null();
        // query_mode allocates a buffer and stores the heap ptr in the provided
        // variable. In this buffer, the queried data can be found.
        unsafe { (self.0.query_mode)(&self.0, index, &mut info_sz, &mut info_heap_ptr) }
            .to_result_with_val(|| {
                // Transform to owned info on the stack.
                let info = unsafe { *info_heap_ptr };

                let info_heap_ptr = info_heap_ptr.cast::<u8>().cast_mut();

                // User has no benefit from propagating this error. If this
                // fails, it is an error of the UEFI implementation.
                unsafe { boot::free_pool(NonNull::new(info_heap_ptr).unwrap()) }
                    .expect("buffer should be deallocatable");

                Mode {
                    index,
                    info_sz,
                    info: ModeInfo(info),
                }
            })
    }

    /// Returns a [`ModeIter`].
    #[must_use]
    pub const fn modes(&self) -> ModeIter {
        ModeIter {
            gop: self,
            current: 0,
            max: self.mode().max_mode,
        }
    }

    /// Sets the video device into the specified mode, clearing visible portions
    /// of the output display to black.
    ///
    /// This function will invalidate the current framebuffer.
    pub fn set_mode(&mut self, mode: &Mode) -> Result {
        unsafe { (self.0.set_mode)(&mut self.0, mode.index) }.to_result()
    }

    /// Performs a blt (block transfer) operation on the frame buffer.
    ///
    /// Every operation requires different parameters.
    pub fn blt(&mut self, op: BltOp) -> Result {
        // Demultiplex the operation type.
        unsafe {
            match op {
                BltOp::VideoFill {
                    color,
                    dest: (dest_x, dest_y),
                    dims: (width, height),
                } => {
                    self.check_framebuffer_region((dest_x, dest_y), (width, height));
                    (self.0.blt)(
                        &mut self.0,
                        ptr::from_ref(&color) as *mut _,
                        GraphicsOutputBltOperation::BLT_VIDEO_FILL,
                        0,
                        0,
                        dest_x,
                        dest_y,
                        width,
                        height,
                        0,
                    )
                    .to_result()
                }
                BltOp::VideoToBltBuffer {
                    buffer,
                    src: (src_x, src_y),
                    dest: dest_region,
                    dims: (width, height),
                } => {
                    self.check_framebuffer_region((src_x, src_y), (width, height));
                    self.check_blt_buffer_region(dest_region, (width, height), buffer.len());
                    match dest_region {
                        BltRegion::Full => (self.0.blt)(
                            &mut self.0,
                            buffer.as_mut_ptr().cast(),
                            GraphicsOutputBltOperation::BLT_VIDEO_TO_BLT_BUFFER,
                            src_x,
                            src_y,
                            0,
                            0,
                            width,
                            height,
                            0,
                        )
                        .to_result(),
                        BltRegion::SubRectangle {
                            coords: (dest_x, dest_y),
                            px_stride,
                        } => (self.0.blt)(
                            &mut self.0,
                            buffer.as_mut_ptr().cast(),
                            GraphicsOutputBltOperation::BLT_VIDEO_TO_BLT_BUFFER,
                            src_x,
                            src_y,
                            dest_x,
                            dest_y,
                            width,
                            height,
                            px_stride * size_of::<BltPixel>(),
                        )
                        .to_result(),
                    }
                }
                BltOp::BufferToVideo {
                    buffer,
                    src: src_region,
                    dest: (dest_x, dest_y),
                    dims: (width, height),
                } => {
                    self.check_blt_buffer_region(src_region, (width, height), buffer.len());
                    self.check_framebuffer_region((dest_x, dest_y), (width, height));
                    match src_region {
                        BltRegion::Full => (self.0.blt)(
                            &mut self.0,
                            buffer.as_ptr() as *mut _,
                            GraphicsOutputBltOperation::BLT_BUFFER_TO_VIDEO,
                            0,
                            0,
                            dest_x,
                            dest_y,
                            width,
                            height,
                            0,
                        )
                        .to_result(),
                        BltRegion::SubRectangle {
                            coords: (src_x, src_y),
                            px_stride,
                        } => (self.0.blt)(
                            &mut self.0,
                            buffer.as_ptr() as *mut _,
                            GraphicsOutputBltOperation::BLT_BUFFER_TO_VIDEO,
                            src_x,
                            src_y,
                            dest_x,
                            dest_y,
                            width,
                            height,
                            px_stride * size_of::<BltPixel>(),
                        )
                        .to_result(),
                    }
                }
                BltOp::VideoToVideo {
                    src: (src_x, src_y),
                    dest: (dest_x, dest_y),
                    dims: (width, height),
                } => {
                    self.check_framebuffer_region((src_x, src_y), (width, height));
                    self.check_framebuffer_region((dest_x, dest_y), (width, height));
                    (self.0.blt)(
                        &mut self.0,
                        ptr::null_mut(),
                        GraphicsOutputBltOperation::BLT_VIDEO_TO_VIDEO,
                        src_x,
                        src_y,
                        dest_x,
                        dest_y,
                        width,
                        height,
                        0,
                    )
                    .to_result()
                }
            }
        }
    }

    /// Memory-safety check for accessing a region of the framebuffer
    fn check_framebuffer_region(&self, coords: (usize, usize), dims: (usize, usize)) {
        let (width, height) = self.current_mode_info().resolution();
        assert!(
            coords.0.saturating_add(dims.0) <= width,
            "Horizontal framebuffer coordinate out of bounds"
        );
        assert!(
            coords.1.saturating_add(dims.1) <= height,
            "Vertical framebuffer coordinate out of bounds"
        );
    }

    /// Memory-safety check for accessing a region of a user-provided buffer
    fn check_blt_buffer_region(&self, region: BltRegion, dims: (usize, usize), buf_length: usize) {
        match region {
            BltRegion::Full => assert!(
                dims.1.saturating_mul(dims.0) <= buf_length,
                "BltBuffer access out of bounds"
            ),
            BltRegion::SubRectangle {
                coords: (x, y),
                px_stride,
            } => {
                assert!(
                    x.saturating_add(dims.0) <= px_stride,
                    "Horizontal BltBuffer coordinate out of bounds"
                );
                assert!(
                    y.saturating_add(dims.1).saturating_mul(px_stride) <= buf_length,
                    "Vertical BltBuffer coordinate out of bounds"
                );
            }
        }
    }

    /// Returns the frame buffer information for the current mode.
    #[must_use]
    pub const fn current_mode_info(&self) -> ModeInfo {
        unsafe { *self.mode().info.cast_const().cast::<ModeInfo>() }
    }

    /// Access the frame buffer directly
    pub fn frame_buffer(&mut self) -> FrameBuffer {
        assert!(
            self.current_mode_info().pixel_format() != PixelFormat::BltOnly,
            "Cannot access the framebuffer in a Blt-only mode"
        );
        let base = self.mode().frame_buffer_base as *mut u8;
        let size = self.mode().frame_buffer_size;

        FrameBuffer {
            base,
            size,
            _lifetime: PhantomData,
        }
    }

    const fn mode(&self) -> &GraphicsOutputProtocolMode {
        unsafe { &*self.0.mode.cast_const() }
    }
}

/// Represents the format of the pixels in a frame buffer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum PixelFormat {
    /// Each pixel is 32-bit long, with 24-bit RGB, and the last byte is reserved.
    Rgb = 0,
    /// Each pixel is 32-bit long, with 24-bit BGR, and the last byte is reserved.
    Bgr,
    /// Custom pixel format, check the associated bitmask.
    Bitmask,
    /// The graphics mode does not support drawing directly to the frame buffer.
    ///
    /// This means you will have to use the `blt` function which will
    /// convert the graphics data to the device's internal pixel format.
    BltOnly,
    // SAFETY: UEFI also defines a PixelFormatMax variant, and states that all
    //         valid enum values are guaranteed to be smaller. Since that is the
    //         case, adding a new enum variant would be a breaking change, so it
    //         is safe to model this C enum as a Rust enum.
}

/// Represents a graphics mode compatible with a given graphics device.
#[derive(Copy, Clone, Debug)]
pub struct Mode {
    index: u32,
    info_sz: usize,
    info: ModeInfo,
}

impl Mode {
    /// The size of the info structure in bytes.
    ///
    /// Newer versions of the spec might add extra information, in a backwards compatible way.
    #[must_use]
    pub const fn info_size(&self) -> usize {
        self.info_sz
    }

    /// Returns a reference to the mode info structure.
    #[must_use]
    pub const fn info(&self) -> &ModeInfo {
        &self.info
    }
}

/// Information about a graphics output mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ModeInfo(GraphicsOutputModeInformation);

impl ModeInfo {
    /// Returns the (horizontal, vertical) resolution.
    ///
    /// On desktop monitors, this usually means (width, height).
    #[must_use]
    pub const fn resolution(&self) -> (usize, usize) {
        (
            usize_from_u32(self.0.horizontal_resolution),
            usize_from_u32(self.0.vertical_resolution),
        )
    }

    /// Returns the format of the frame buffer.
    #[must_use]
    pub const fn pixel_format(&self) -> PixelFormat {
        match self.0.pixel_format.0 {
            0 => PixelFormat::Rgb,
            1 => PixelFormat::Bgr,
            2 => PixelFormat::Bitmask,
            3 => PixelFormat::BltOnly,
            _ => panic!("invalid pixel format"),
        }
    }

    /// Returns the bitmask of the custom pixel format, if available.
    #[must_use]
    pub const fn pixel_bitmask(&self) -> Option<PixelBitmask> {
        match self.pixel_format() {
            PixelFormat::Bitmask => Some(self.0.pixel_information),
            _ => None,
        }
    }

    /// Returns the number of pixels per scanline.
    ///
    /// Due to performance reasons, the stride might not be equal to the width,
    /// instead the stride might be bigger for better alignment.
    #[must_use]
    pub const fn stride(&self) -> usize {
        usize_from_u32(self.0.pixels_per_scan_line)
    }
}

/// Iterator for [`Mode`]s of the [`GraphicsOutput`] protocol.
pub struct ModeIter<'gop> {
    gop: &'gop GraphicsOutput,
    current: u32,
    max: u32,
}

impl Iterator for ModeIter<'_> {
    type Item = Mode;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current;
        if index < self.max {
            let m = self.gop.query_mode(index);
            self.current += 1;

            m.ok().or_else(|| self.next())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.max - self.current) as usize;
        (size, Some(size))
    }
}

impl Debug for ModeIter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModeIter")
            .field("current", &self.current)
            .field("max", &self.max)
            .finish()
    }
}

impl ExactSizeIterator for ModeIter<'_> {}

/// Format of pixel data used for blitting.
///
/// This is a BGR 24-bit format with an 8-bit padding, to keep each pixel 32-bit in size.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BltPixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    _reserved: u8,
}

impl BltPixel {
    /// Create a new pixel from RGB values.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            _reserved: 0,
        }
    }
}

impl From<u32> for BltPixel {
    fn from(color: u32) -> Self {
        Self {
            blue: (color & 0x00_00_FF) as u8,
            green: ((color & 0x00_FF_00) >> 8) as u8,
            red: ((color & 0xFF_00_00) >> 16) as u8,
            _reserved: 0,
        }
    }
}

/// Region of the `BltBuffer` which we are operating on
///
/// Some `Blt` operations can operate on either the full `BltBuffer` or a
/// sub-rectangle of it, but require the stride to be known in the latter case.
#[derive(Clone, Copy, Debug)]
pub enum BltRegion {
    /// Operate on the full `BltBuffer`
    Full,

    /// Operate on a sub-rectangle of the `BltBuffer`
    SubRectangle {
        /// Coordinate of the rectangle in the `BltBuffer`
        coords: (usize, usize),

        /// Stride (length of each row of the `BltBuffer`) in **pixels**
        px_stride: usize,
    },
}

/// Blit operation to perform.
#[derive(Debug)]
pub enum BltOp<'buf> {
    /// Fills a rectangle of video display with a pixel color.
    VideoFill {
        /// The color to fill with.
        color: BltPixel,
        /// The X / Y coordinates of the destination rectangle.
        dest: (usize, usize),
        /// The width / height of the rectangle.
        dims: (usize, usize),
    },
    /// Reads data from the video display to the buffer.
    VideoToBltBuffer {
        /// Buffer into which to copy data.
        buffer: &'buf mut [BltPixel],
        /// Coordinates of the source rectangle, in the frame buffer.
        src: (usize, usize),
        /// Location of the destination rectangle in the user-provided buffer
        dest: BltRegion,
        /// Width / height of the rectangles.
        dims: (usize, usize),
    },
    /// Write data from the buffer to the video rectangle.
    /// Delta must be the stride (count of bytes in a row) of the buffer.
    BufferToVideo {
        /// Buffer from which to copy data.
        buffer: &'buf [BltPixel],
        /// Location of the source rectangle in the user-provided buffer.
        src: BltRegion,
        /// Coordinates of the destination rectangle, in the frame buffer.
        dest: (usize, usize),
        /// Width / height of the rectangles.
        dims: (usize, usize),
    },
    /// Copy from the source rectangle in video memory to
    /// the destination rectangle, also in video memory.
    VideoToVideo {
        /// Coordinates of the source rectangle, in the frame buffer.
        src: (usize, usize),
        /// Coordinates of the destination rectangle, also in the frame buffer.
        dest: (usize, usize),
        /// Width / height of the rectangles.
        dims: (usize, usize),
    },
}

/// Direct access to a memory-mapped frame buffer
#[derive(Debug)]
pub struct FrameBuffer<'gop> {
    base: *mut u8,
    size: usize,
    _lifetime: PhantomData<&'gop mut u8>,
}

impl FrameBuffer<'_> {
    /// Access the raw framebuffer pointer
    ///
    /// To use this pointer safely and correctly, you must...
    /// - Honor the pixel format and stride specified by the mode info
    /// - Keep memory accesses in bound
    /// - Use volatile reads and writes
    /// - Make sure that the pointer does not outlive the FrameBuffer
    ///
    /// On some implementations this framebuffer pointer can be used after
    /// exiting boot services, but that is not guaranteed by the UEFI Specification.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.base
    }

    /// Query the framebuffer size in bytes
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Modify the i-th byte of the frame buffer
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn write_byte(&mut self, index: usize, value: u8) {
        debug_assert!(index < self.size, "Frame buffer accessed out of bounds");
        unsafe { self.base.add(index).write_volatile(value) }
    }

    /// Read the i-th byte of the frame buffer
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    #[must_use]
    pub unsafe fn read_byte(&self, index: usize) -> u8 {
        debug_assert!(index < self.size, "Frame buffer accessed out of bounds");
        unsafe { self.base.add(index).read_volatile() }
    }

    /// Write a value in the frame buffer, starting at the i-th byte
    ///
    /// We only recommend using this method with [u8; N] arrays. Once Rust has
    /// const generics, it will be deprecated and replaced with a write_bytes()
    /// method that only accepts [u8; N] input.
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - It is your responsibility to make sure that the value type makes sense
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn write_value<T>(&mut self, index: usize, value: T) {
        debug_assert!(
            index.saturating_add(size_of::<T>()) <= self.size,
            "Frame buffer accessed out of bounds"
        );
        unsafe {
            let ptr = self.base.add(index).cast::<T>();
            ptr.write_volatile(value)
        }
    }

    /// Read a value from the frame buffer, starting at the i-th byte
    ///
    /// We only recommend using this method with [u8; N] arrays. Once Rust has
    /// const generics, it will be deprecated and replaced with a read_bytes()
    /// method that only accepts [u8; N] input.
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - It is your responsibility to make sure that the value type makes sense
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    #[must_use]
    pub unsafe fn read_value<T>(&self, index: usize) -> T {
        debug_assert!(
            index.saturating_add(size_of::<T>()) <= self.size,
            "Frame buffer accessed out of bounds"
        );
        unsafe { (self.base.add(index) as *const T).read_volatile() }
    }
}
