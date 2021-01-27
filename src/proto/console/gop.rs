//! Graphics output protocol.
//!
//! The UEFI GOP is meant to replace existing [VGA][vga] hardware interfaces.
//! It can be used in the boot environment as well as at runtime,
//! until a high-performance driver is loaded by the OS.
//!
//! The GOP provides access to a hardware frame buffer and allows UEFI apps
//! to draw directly to the graphics output device.
//!
//! The advantage of the GOP over legacy VGA is that it allows multiple GPUs
//! to exist and be used on the system. There is a GOP implementation for every
//! unique GPU in the system which supports UEFI.
//!
//! This protocol _can_ be used after boot services are exited.
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

use crate::proto::Protocol;
use crate::{unsafe_guid, Completion, Result, Status};
use core::marker::PhantomData;
use core::mem;
use core::ptr;
use uefi_sys::{
    EFI_GRAPHICS_OUTPUT_BLT_PIXEL, EFI_GRAPHICS_OUTPUT_MODE_INFORMATION,
    EFI_GRAPHICS_OUTPUT_PROTOCOL, EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE, EFI_PIXEL_BITMASK,
};

/// Provides access to the video hardware's frame buffer.
///
/// The GOP can be used to set the properties of the frame buffer,
/// and also allows the app to access the in-memory buffer.
#[repr(C)]
#[unsafe_guid("9042a9de-23dc-4a38-96fb-7aded080516a")]
#[derive(Protocol)]
pub struct GraphicsOutput<'boot> {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_GRAPHICS_OUTPUT_PROTOCOL,
    _marker: PhantomData<&'boot ()>,
}

impl<'boot> GraphicsOutput<'boot> {
    /// Returns information for an available graphics mode that the graphics
    /// device and the set of active video output devices supports.
    fn query_mode(&self, index: u32) -> Result<Mode> {
        let mut info_sz = 0usize;
        let mut info: *mut EFI_GRAPHICS_OUTPUT_MODE_INFORMATION =
            ptr::null::<EFI_GRAPHICS_OUTPUT_MODE_INFORMATION>() as *mut _;

        Status::from_raw_api(unsafe {
            self.raw.QueryMode.unwrap()(
                self as *const _ as *mut _,
                index,
                &mut info_sz as *mut _ as *mut _,
                &mut info as _,
            )
        })
        .into_with_val(|| {
            let info = unsafe { *info };
            Mode {
                index,
                info_sz,
                info: ModeInfo { raw: info },
            }
        })
    }

    fn mode(&self) -> &'boot ModeData<'boot> {
        unsafe { &*(self.raw.Mode as *const ModeData) }
    }

    /// Returns information about all available graphics modes.
    pub fn modes(&'_ self) -> impl ExactSizeIterator<Item = Completion<Mode>> + '_ {
        ModeIter {
            gop: self,
            current: 0,
            max: self.mode().max_mode(),
        }
    }

    /// Sets the video device into the specified mode, clearing visible portions
    /// of the output display to black.
    ///
    /// This function will invalidate the current framebuffer.
    pub fn set_mode(&mut self, mode: &Mode) -> Result {
        Status::from_raw_api(unsafe { self.raw.SetMode.unwrap()(&mut self.raw, mode.index) }).into()
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
                    Status::from_raw_api(self.raw.Blt.unwrap()(
                        &mut self.raw,
                        &color as *const _ as *mut _,
                        0,
                        0,
                        0,
                        dest_x as _,
                        dest_y as _,
                        width as _,
                        height as _,
                        0,
                    ))
                    .into()
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
                        BltRegion::Full => Status::from_raw_api(self.raw.Blt.unwrap()(
                            &mut self.raw,
                            buffer.as_mut_ptr() as *mut _ as *mut _,
                            1,
                            src_x as _,
                            src_y as _,
                            0,
                            0,
                            width as _,
                            height as _,
                            0,
                        ))
                        .into(),
                        BltRegion::SubRectangle {
                            coords: (dest_x, dest_y),
                            px_stride,
                        } => Status::from_raw_api(self.raw.Blt.unwrap()(
                            &mut self.raw,
                            buffer.as_mut_ptr() as *mut _ as *mut _,
                            1,
                            src_x as _,
                            src_y as _,
                            dest_x as _,
                            dest_y as _,
                            width as _,
                            height as _,
                            (px_stride * core::mem::size_of::<BltPixel>()) as _,
                        ))
                        .into(),
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
                        BltRegion::Full => Status::from_raw_api(self.raw.Blt.unwrap()(
                            &mut self.raw,
                            buffer.as_ptr() as *mut _,
                            2,
                            0,
                            0,
                            dest_x as _,
                            dest_y as _,
                            width as _,
                            height as _,
                            0,
                        ))
                        .into(),
                        BltRegion::SubRectangle {
                            coords: (src_x, src_y),
                            px_stride,
                        } => Status::from_raw_api(self.raw.Blt.unwrap()(
                            &mut self.raw,
                            buffer.as_ptr() as *mut _,
                            2,
                            src_x as _,
                            src_y as _,
                            dest_x as _,
                            dest_y as _,
                            width as _,
                            height as _,
                            (px_stride * core::mem::size_of::<BltPixel>()) as _,
                        ))
                        .into(),
                    }
                }
                BltOp::VideoToVideo {
                    src: (src_x, src_y),
                    dest: (dest_x, dest_y),
                    dims: (width, height),
                } => {
                    self.check_framebuffer_region((src_x, src_y), (width, height));
                    self.check_framebuffer_region((dest_x, dest_y), (width, height));
                    Status::from_raw_api(self.raw.Blt.unwrap()(
                        &mut self.raw,
                        ptr::null_mut(),
                        3,
                        src_x as _,
                        src_y as _,
                        dest_x as _,
                        dest_y as _,
                        width as _,
                        height as _,
                        0,
                    ))
                    .into()
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
                dims.0.saturating_add(dims.1.saturating_mul(dims.0)) <= buf_length,
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
    pub fn current_mode_info(&self) -> ModeInfo {
        *self.mode().info()
    }

    /// Access the frame buffer directly
    pub fn frame_buffer(&mut self) -> FrameBuffer {
        let mode = self.mode();
        assert!(
            mode.info().pixel_format() != PixelFormat::BltOnly,
            "Cannot access the framebuffer in a Blt-only mode"
        );
        let base = mode.fb_address() as *mut u8;
        let size = mode.fb_size();

        FrameBuffer {
            base,
            size,
            _lifetime: PhantomData,
        }
    }
}

#[repr(C)]
struct ModeData<'info> {
    pub raw: EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE,
    _marker: PhantomData<&'info ()>,
}

impl<'info> ModeData<'info> {
    /// Number of modes which the GOP supports.
    pub fn max_mode(&self) -> u32 {
        self.raw.MaxMode
    }

    /// Current mode.
    pub fn mode(&self) -> u32 {
        self.raw.Mode
    }

    /// Information about the current mode.
    pub fn info(&self) -> &'info ModeInfo {
        unsafe { &*(self.raw.Info as *const ModeInfo) }
    }

    /// Size of the above structure.
    pub fn info_sz(&self) -> usize {
        self.raw.SizeOfInfo as _
    }

    /// Physical address of the frame buffer.
    pub fn fb_address(&self) -> u64 {
        self.raw.FrameBufferBase
    }

    /// Size in bytes. Equal to (pixel size) * height * stride.
    pub fn fb_size(&self) -> usize {
        self.raw.FrameBufferSize as _
    }
}

/// Represents the format of the pixels in a frame buffer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum PixelFormat {
    /// Each pixel is 32-bit long, with 24-bit RGB, and the last byte is reserved.
    RGB = 0,
    /// Each pixel is 32-bit long, with 24-bit BGR, and the last byte is reserved.
    BGR,
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

/// Bitmask used to indicate which bits of a pixel represent a given color.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct PixelBitmask {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_PIXEL_BITMASK,
}

impl PixelBitmask {
    /// The bits indicating the red channel.
    pub fn red(&self) -> u32 {
        self.raw.RedMask
    }
    /// The bits indicating the green channel.
    pub fn green(&self) -> u32 {
        self.raw.GreenMask
    }
    /// The bits indicating the blue channel.
    pub fn blue(&self) -> u32 {
        self.raw.BlueMask
    }
}

/// Represents a graphics mode compatible with a given graphics device.
pub struct Mode {
    index: u32,
    info_sz: usize,
    info: ModeInfo,
}

impl Mode {
    /// The size of the info structure in bytes.
    ///
    /// Newer versions of the spec might add extra information, in a backwards compatible way.
    pub fn info_size(&self) -> usize {
        self.info_sz
    }

    /// Returns a reference to the mode info structure.
    pub fn info(&self) -> &ModeInfo {
        &self.info
    }
}

/// Information about a graphics output mode.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ModeInfo {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_GRAPHICS_OUTPUT_MODE_INFORMATION,
}

impl ModeInfo {
    /// Returns the (horizontal, vertical) resolution.
    ///
    /// On desktop monitors, this usually means (width, height).
    pub fn resolution(&self) -> (usize, usize) {
        (
            self.raw.HorizontalResolution as usize,
            self.raw.VerticalResolution as usize,
        )
    }

    /// Returns the format of the frame buffer.
    pub fn pixel_format(&self) -> PixelFormat {
        unsafe { core::mem::transmute(self.raw.PixelFormat) }
    }

    /// Returns the bitmask of the custom pixel format, if available.
    pub fn pixel_bitmask(&self) -> Option<PixelBitmask> {
        match self.pixel_format() {
            PixelFormat::Bitmask => Some(PixelBitmask {
                raw: self.raw.PixelInformation,
            }),
            _ => None,
        }
    }

    /// Returns the number of pixels per scanline.
    ///
    /// Due to performance reasons, the stride might not be equal to the width,
    /// instead the stride might be bigger for better alignment.
    pub fn stride(&self) -> usize {
        self.raw.PixelsPerScanLine as _
    }
}

/// Iterator for graphics modes.
struct ModeIter<'gop> {
    gop: &'gop GraphicsOutput<'gop>,
    current: u32,
    max: u32,
}

impl<'gop> Iterator for ModeIter<'gop> {
    type Item = Completion<Mode>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current;
        if index < self.max {
            self.current += 1;

            self.gop.query_mode(index).ok().or_else(|| self.next())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.max - self.current) as usize;
        (size, Some(size))
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
    pub raw: EFI_GRAPHICS_OUTPUT_BLT_PIXEL,
}

impl BltPixel {
    /// Create a new pixel from RGB values.
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            raw: EFI_GRAPHICS_OUTPUT_BLT_PIXEL {
                Red: red,
                Green: green,
                Blue: blue,
                Reserved: 0,
            },
        }
    }
}

impl From<u32> for BltPixel {
    fn from(color: u32) -> Self {
        Self::new(
            (color & 0x00_00_FF) as u8,
            ((color & 0x00_FF_00) >> 8) as u8,
            ((color & 0xFF_00_00) >> 16) as u8,
        )
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
pub struct FrameBuffer<'gop> {
    base: *mut u8,
    size: usize,
    _lifetime: PhantomData<&'gop mut u8>,
}

impl<'gop> FrameBuffer<'gop> {
    /// Access the raw framebuffer pointer
    ///
    /// To use this pointer safely and correctly, you must...
    /// - Honor the pixel format and stride specified by the mode info
    /// - Keep memory accesses in bound
    /// - Use volatile reads and writes
    /// - Make sure that the pointer does not outlive the FrameBuffer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.base
    }

    /// Query the framebuffer size in bytes
    pub fn size(&self) -> usize {
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
        self.base.add(index).write_volatile(value)
    }

    /// Read the i-th byte of the frame buffer
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn read_byte(&self, index: usize) -> u8 {
        debug_assert!(index < self.size, "Frame buffer accessed out of bounds");
        self.base.add(index).read_volatile()
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
    /// - It is your reponsibility to make sure that the value type makes sense
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn write_value<T>(&mut self, index: usize, value: T) {
        debug_assert!(
            index.saturating_add(mem::size_of::<T>()) <= self.size,
            "Frame buffer accessed out of bounds"
        );
        (self.base.add(index) as *mut T).write_volatile(value)
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
    /// - It is your reponsibility to make sure that the value type makes sense
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn read_value<T>(&self, index: usize) -> T {
        debug_assert!(
            index.saturating_add(mem::size_of::<T>()) <= self.size,
            "Frame buffer accessed out of bounds"
        );
        (self.base.add(index) as *const T).read_volatile()
    }
}
