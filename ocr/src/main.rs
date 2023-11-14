use std::ffi::c_void;
use std::time::Instant;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSColor,
    NSEvent, NSImage, NSImageView, NSRectFill, NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSData, NSPoint, NSRect, NSSize, NSUInteger};
use core_graphics::context::CGContextRef;
use core_graphics::display::{CGDisplay, CGDisplayBounds};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::sys::CGContext;
use objc::declare::ClassDecl;
use objc::rc::StrongPtr;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use screenshots::image::codecs::png::PngEncoder;
use screenshots::image::{ColorType, ImageEncoder};
use screenshots::Screen;

struct State {
    start_point: Option<(f64, f64)>,
    end_point: Option<(f64, f64)>,
    pressed: bool,
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
                let state = State {
                    start_point: None,
                    end_point: None,
                    pressed: false,
                };

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

                extern "C" fn mouse_down(_this: &Object, _sel: Sel, _event: id) {
                    unsafe {
                        let mouse_location = NSEvent::mouseLocation(nil);
                        println!(
                            "Mouse button pressed at x: {}, y: {}",
                            mouse_location.x, mouse_location.y
                        );
                    }
                }
                extern "C" fn mouse_up(_this: &Object, _sel: Sel, _event: id) {
                    unsafe {
                        let mouse_location = NSEvent::mouseLocation(nil);
                        println!(
                            "Mouse button pressed at x: {}, y: {}",
                            mouse_location.x, mouse_location.y
                        );
                    }
                }

                extern "C" fn draw_rect(this: &Object, _sel: Sel, _rect: NSRect) {
                    unsafe {
                        let blue: id = msg_send![class!(NSColor), colorWithCalibratedRed:0.0 green:0.0 blue:1.0 alpha:0.1];
                        let _: () = msg_send![blue, setFill];
                        let rect = msg_send![NSMakeRect, 0.0, 0., 300., 300.];
                        // NSRectFill(NSRect::new(
                        //     NSPoint::new(0., 0.),
                        //     NSSize::new(300.0, 300.0),
                        // ));
                        println!("({}, {})", _rect.size.width, _rect.size.height);
                        NSRectFill(_rect);
                    }
                }
                let cls_name = format!("Display{display}EventHandlerView");
                let superclass = Class::get("NSView").unwrap();
                let mut decl = ClassDecl::new(&cls_name, superclass).unwrap();
                decl.add_method(
                    sel!(mouseDown:),
                    mouse_down as extern "C" fn(&Object, Sel, id),
                );
                decl.add_method(sel!(mouseUp:), mouse_up as extern "C" fn(&Object, Sel, id));
                decl.add_method(
                    sel!(drawRect:),
                    draw_rect as extern "C" fn(&Object, Sel, NSRect),
                );
                let my_view_class = decl.register();
                let custom_view: id = msg_send![my_view_class, new];

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

                image_view.addSubview_(custom_view);
                // 将 NSImageView 添加到窗口的内容视图
                window.contentView().addSubview_(image_view);
                // 设置窗口层级，使其覆盖Dock和菜单栏
                window.setLevel_(cocoa::appkit::NSMainMenuWindowLevel as i64 + 2);
                window.setOpaque_(NO);
                // 直接让窗口透明，截图动态图片
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
