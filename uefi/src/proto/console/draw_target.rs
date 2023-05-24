use embedded_graphics_core::prelude::{DrawTarget, OriginDimensions, Size, PixelColor, Pixel, IntoStorage};

use super::gop::GraphicsOutput;

// FIXME: offer conversions from C to current pixel color format?
struct GraphicsDisplay<C: PixelColor> {
    color: C,
    gop: GraphicsOutput
}

impl OriginDimensions for GraphicsOutput {
    fn size(&self) -> embedded_graphics_core::prelude::Size {
        let (width, height) = self.current_mode_info().resolution();

        Size::from((width as u32, height as u32))
    }
}

impl<C: PixelColor> OriginDimensions for GraphicsDisplay<C> {
    fn size(&self) -> Size {
        self.gop.size()
    }
}

impl<C: PixelColor + IntoStorage> DrawTarget for GraphicsDisplay<C> {
    type Color = C;
    type Error = uefi::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>> {
        let stride = self.gop.current_mode_info().stride() as u64;
        for Pixel(point, color) in pixels.into_iter() {
            let bytes = color.into_storage();
            let (x, y) = (point.x as u64, point.y as u64);
            let index: usize = (((y * stride) + x) * 4)
                .try_into()
                .map_err(|_|
                    uefi::Error::from(
                        uefi::Status::UNSUPPORTED
                    )
                )?;

            unsafe {
                self.gop.frame_buffer().write_value(index, bytes);
            }
        }

        Ok(())
    }

    // FIXME: provide a blt technique for fill_solid
    // FIXME: fallback to blt when pixelformat is blt-only.
}
