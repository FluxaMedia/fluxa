use std::ffi::c_void;
use std::ptr::null;

type CFTypeRef = *const c_void;

#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
struct CFRange {
    location: isize,
    length: isize,
}

const UTF8: u32 = 0x0800_0100;
const PREMULTIPLIED_LAST: u32 = 1;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: CFTypeRef);
    fn CFStringCreateWithBytes(
        alloc: CFTypeRef,
        bytes: *const u8,
        len: isize,
        encoding: u32,
        external: bool,
    ) -> CFTypeRef;
    fn CFAttributedStringCreateMutable(alloc: CFTypeRef, max: isize) -> CFTypeRef;
    fn CFAttributedStringReplaceString(string: CFTypeRef, range: CFRange, replacement: CFTypeRef);
    fn CFAttributedStringSetAttribute(
        string: CFTypeRef,
        range: CFRange,
        name: CFTypeRef,
        value: CFTypeRef,
    );
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGColorSpaceCreateDeviceRGB() -> CFTypeRef;
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits: usize,
        row_bytes: usize,
        space: CFTypeRef,
        info: u32,
    ) -> CFTypeRef;
    fn CGContextSetTextPosition(context: CFTypeRef, x: f64, y: f64);
}

#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    static kCTFontAttributeName: CFTypeRef;
    fn CTFontCreateWithName(name: CFTypeRef, size: f64, matrix: *const c_void) -> CFTypeRef;
    fn CTLineCreateWithAttributedString(string: CFTypeRef) -> CFTypeRef;
    fn CTLineGetTypographicBounds(
        line: CFTypeRef,
        ascent: *mut f64,
        descent: *mut f64,
        leading: *mut f64,
    ) -> f64;
    fn CTLineDraw(line: CFTypeRef, context: CFTypeRef);
}

unsafe fn string(value: &str) -> CFTypeRef {
    unsafe { CFStringCreateWithBytes(null(), value.as_ptr(), value.len() as isize, UTF8, false) }
}

pub fn rasterize(cluster: &str, size: u32) -> Option<(u32, u32, Vec<u8>)> {
    let side = size as usize;
    let mut pixels = vec![0u8; side * side * 4];
    unsafe {
        let space = CGColorSpaceCreateDeviceRGB();
        let context = CGBitmapContextCreate(
            pixels.as_mut_ptr().cast(),
            side,
            side,
            8,
            side * 4,
            space,
            PREMULTIPLIED_LAST,
        );
        CFRelease(space);
        if context.is_null() {
            return None;
        }
        let font_name = string("AppleColorEmoji");
        let font = CTFontCreateWithName(font_name, f64::from(size) * 0.86, null());
        CFRelease(font_name);
        let text = string(cluster);
        let attributed = CFAttributedStringCreateMutable(null(), 0);
        CFAttributedStringReplaceString(
            attributed,
            CFRange {
                location: 0,
                length: 0,
            },
            text,
        );
        let length = CFRange {
            location: 0,
            length: cluster.encode_utf16().count() as isize,
        };
        CFAttributedStringSetAttribute(attributed, length, kCTFontAttributeName, font);
        let line = CTLineCreateWithAttributedString(attributed);
        let (mut ascent, mut descent, mut leading) = (0.0, 0.0, 0.0);
        let width = CTLineGetTypographicBounds(line, &mut ascent, &mut descent, &mut leading);
        if width > 0.0 {
            let side = f64::from(size);
            let position = CGPoint {
                x: (side - width) / 2.0,
                y: (side - (ascent + descent)) / 2.0 + descent,
            };
            CGContextSetTextPosition(context, position.x, position.y);
            CTLineDraw(line, context);
        }
        CFRelease(line);
        CFRelease(attributed);
        CFRelease(text);
        CFRelease(font);
        CFRelease(context);
        if width <= 0.0 {
            return None;
        }
    }
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha != 0 && alpha != 255 {
            for channel in &mut pixel[..3] {
                *channel = (u32::from(*channel) * 255 / alpha).min(255) as u8;
            }
        }
    }
    Some((size, size, pixels))
}
