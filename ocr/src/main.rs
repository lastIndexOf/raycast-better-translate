use std::ffi::c_void;
use std::time::Instant;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSImage,
    NSImageView, NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSData, NSPoint, NSRect, NSSize, NSUInteger};
use core_graphics::display::{CGDisplay, CGDisplayBounds};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{msg_send, sel, sel_impl};
use screenshots::image::codecs::png::PngEncoder;
use screenshots::image::{ColorType, ImageEncoder};
use screenshots::Screen;

extern "C" fn mouse_down(_this: &Object, _sel: Sel, event: id) {
    println!("Mouse button pressed.");
}

extern "C" fn mouse_dragged(_this: &Object, _sel: Sel, event: id) {
    println!("Mouse dragged.");
}

extern "C" fn mouse_up(_this: &Object, _sel: Sel, event: id) {
    println!("Mouse button released.");
}

unsafe fn create_custom_view_class(view_id: u32) -> *mut Object {
    let cls_name = format!("Display{view_id}EventHandlerClass");
    let superclass = Class::get("NSView").unwrap();
    let mut decl = ClassDecl::new(&cls_name, superclass).unwrap();

    decl.add_method(
        sel!(mouseDown:),
        mouse_down as extern "C" fn(&Object, Sel, id),
    );
    decl.add_method(
        sel!(mouseDragged:),
        mouse_dragged as extern "C" fn(&Object, Sel, id),
    );
    decl.add_method(sel!(mouseUp:), mouse_up as extern "C" fn(&Object, Sel, id));

    let my_view_class = decl.register();

    msg_send![my_view_class, new]
}

fn main() -> anyhow::Result<()> {
    let screens = Screen::all().unwrap();

    unsafe {
        // 创建自动释放池
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        // 初始化应用并设置激活策略
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

        // 获取主显示器的尺寸
        // let main_display_id = CGMainDisplayID();

        let displays = CGDisplay::active_displays().unwrap_or_default();

        let mut offset = (0.0, None);
        for display in displays {
            if let Some(screen) = screens.iter().find(|ele| ele.display_info.id == display) {
                let screen_rect = CGDisplayBounds(display);
                let screen_size = NSSize::new(
                    screen_rect.size.width as f64,
                    screen_rect.size.height as f64,
                );

                let offset_y = offset.1.unwrap_or(screen_rect.size.height);
                // 创建窗口和内容视图
                let window = NSWindow::alloc(nil)
                    .initWithContentRect_styleMask_backing_defer_(
                        NSRect::new(
                            NSPoint::new(offset.0, offset_y - screen_rect.size.height),
                            screen_size,
                        ),
                        NSWindowStyleMask::NSBorderlessWindowMask,
                        NSBackingStoreBuffered,
                        NO,
                    )
                    .autorelease();

                window.setContentView_(create_custom_view_class(display));

                let now = Instant::now();
                // 截屏
                let image = screen.capture()?;
                let (width, height) = image.dimensions();
                println!("capture time: {:?}", now.elapsed());

                // 截屏区域
                // let image = screen.capture_area(
                //     0,
                //     0,
                //     screen.display_info.width,
                //     screen.display_info.height,
                // )?;
                // image.save("./target.png")?;
                // let path = NSString::alloc(nil).init_str("./target.png");
                // let image = NSImage::alloc(nil).initByReferencingFile_(path);

                // Encode ImageBuffer to png
                let mut png_data = Vec::new();
                let encoder = PngEncoder::new(&mut png_data);
                encoder.write_image(&image.into_raw(), width, height, ColorType::Rgba8)?;
                let data_ptr = png_data.as_ptr() as *const c_void;
                let data_len = png_data.len() as NSUInteger;
                let ns_data = NSData::dataWithBytes_length_(nil, data_ptr, data_len);
                let image = NSImage::initWithData_(NSImage::alloc(nil), ns_data).autorelease();
                let image_view = NSView::initWithFrame_(
                    NSImageView::alloc(nil),
                    NSWindow::frame(window.contentView()),
                );
                image_view.setImage_(image);
                image_view.setAutoresizingMask_(
                    cocoa::appkit::NSViewWidthSizable | cocoa::appkit::NSViewHeightSizable,
                );

                // 将 NSImageView 添加到窗口的内容视图
                window.contentView().addSubview_(image_view);

                // 设置窗口层级，使其覆盖Dock和菜单栏
                window.setLevel_(cocoa::appkit::NSMainMenuWindowLevel as i64 + 2);
                window.setOpaque_(NO);
                // window.setBackgroundColor_(cocoa::appkit::NSColor::clearColor(nil));
                window.makeKeyAndOrderFront_(nil);

                offset.0 += screen_rect.size.width;
                offset.1 = Some(screen_rect.size.height);
            }
        }

        app.run();

        anyhow::Ok(())
    }
}
