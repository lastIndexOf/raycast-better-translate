use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSEvent,
    NSImage, NSImageView, NSRectFill, NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSData, NSPoint, NSRect, NSSize, NSString, NSUInteger};
use core_graphics::display::{CGDisplay, CGDisplayBounds};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use screenshots::image::codecs::png::PngEncoder;
use screenshots::image::{ColorType, ImageEncoder};
use screenshots::Screen;
use std::ffi::{c_char, c_void, CStr};

#[derive(Debug)]
struct State<'a> {
    start_point: Option<(f64, f64)>,
    end_point: Option<(f64, f64)>,
    pressed: bool,
    screen: &'a Screen,
    capture_area: Option<(f64, f64, f64, f64)>,
}

fn main() -> anyhow::Result<()> {
    let screens = Screen::all().unwrap();

    unsafe {
        // TODO 解决多屏幕问题
        // TODO 重构代码
        // TODO 添加键盘事件
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
                    screen: &screen,
                    capture_area: None,
                };

                let shared_state = Box::into_raw(Box::new(state));

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

                        {
                            let state_ptr: *mut c_void = *_this.get_ivar("display_state");
                            let mut state = Box::from_raw(state_ptr as *mut State);

                            state.start_point = Some((mouse_location.x, mouse_location.y));
                            state.pressed = true;

                            let _ = Box::into_raw(state);
                        }

                        let subviews: id = msg_send![_this, subviews];
                        let enumerator: id = msg_send![subviews, objectEnumerator];
                        let mut subview: id;

                        while {
                            subview = msg_send![enumerator, nextObject];
                            subview != nil
                        } {
                            let _: () = msg_send![subview, setNeedsDisplay:true];
                        }
                    }
                }
                extern "C" fn mouse_up(_this: &Object, _sel: Sel, _event: id) {
                    unsafe {
                        {
                            let state_ptr: *mut c_void = *_this.get_ivar("display_state");
                            let mut state = Box::from_raw(state_ptr as *mut State);

                            state.start_point = None;
                            state.end_point = None;
                            state.pressed = false;

                            if let Some((ox, oy, px, py)) = state.capture_area {
                                println!("{:?}", state.capture_area);
                                // 截屏区域
                                let image = state
                                    .screen
                                    .capture_area(ox as i32, oy as i32, px as u32, py as u32)
                                    .expect("capture area error");

                                let target = "/tmp/raycast-better-ocr-target.png";

                                println!("save ocr image to {}", target);

                                image.save(target).expect("save image error");
                            }

                            let _ = Box::into_raw(state);

                            std::process::exit(0);
                        }
                    }
                }
                extern "C" fn mouse_dragged(_this: &Object, _sel: Sel, _event: id) {
                    unsafe {
                        let mouse_location = NSEvent::mouseLocation(nil);

                        {
                            let state_ptr: *mut c_void = *_this.get_ivar("display_state");
                            let mut state = Box::from_raw(state_ptr as *mut State);

                            state.end_point = Some((mouse_location.x, mouse_location.y));

                            let _ = Box::into_raw(state);
                        }

                        let subviews: id = msg_send![_this, subviews];
                        let enumerator: id = msg_send![subviews, objectEnumerator];
                        let mut subview: id;

                        while {
                            subview = msg_send![enumerator, nextObject];
                            subview != nil
                        } {
                            let _: () = msg_send![subview, setNeedsDisplay:true];
                        }
                    }
                }
                extern "C" fn key_down(_this: &Object, _sel: Sel, event: id) {
                    unsafe {
                        println!("@@@@@@");

                        let characters: id = msg_send![event, characters];
                        let characters_ptr: *const c_char = msg_send![characters, UTF8String];
                        let key_string = CStr::from_ptr(characters_ptr)
                            .to_string_lossy()
                            .into_owned();

                        println!("Key {} is pressed", key_string);

                        // Check if the key string contains the escape character
                        if key_string == "\u{1b}" {
                            println!("ESC key is pressed.");
                            // Perform your action here
                        }
                    }
                }

                extern "C" fn draw_rect(_this: &Object, _sel: Sel, _dirty_rect: NSRect) {
                    unsafe {
                        let state_ptr: *mut c_void = *_this.get_ivar("display_state");
                        let mut state = Box::from_raw(state_ptr as *mut State);

                        let capture_area = match state.as_ref() {
                            State {
                                start_point: Some((x1, y1)),
                                end_point: Some((x2, y2)),
                                pressed: true,
                                ..
                            } => {
                                if x1 < x2 && y1 < y2 {
                                    (*x1, *y1, x2 - x1, y2 - y1)
                                } else if x1 < x2 && y1 > y2 {
                                    (*x1, *y2, x2 - x1, y1 - y2)
                                } else if x1 > x2 && y1 < y2 {
                                    (*x2, *y1, x1 - x2, y2 - y1)
                                } else {
                                    (*x2, *y2, x1 - x2, y1 - y2)
                                }
                            }
                            _ => {
                                let _ = Box::into_raw(state);
                                return;
                            }
                        };
                        state.capture_area = Some(capture_area);
                        let (ox, oy, px, py) = capture_area;

                        let blue: id = msg_send![class!(NSColor), colorWithCalibratedRed:0.0 green:0.0 blue:1.0 alpha:0.1];
                        let _: () = msg_send![blue, setFill];
                        let rect_to_draw = NSRect::new(NSPoint::new(ox, oy), NSSize::new(px, py));

                        NSRectFill(rect_to_draw);

                        let _ = Box::into_raw(state);
                    }
                }
                let cls_name = format!("Display{display}CaptureView");
                let superclass = Class::get("NSView").unwrap();
                let mut decl = ClassDecl::new(&cls_name, superclass).unwrap();
                decl.add_method(
                    sel!(drawRect:),
                    draw_rect as extern "C" fn(&Object, Sel, NSRect),
                );

                decl.add_ivar::<*mut c_void>("display_state");

                let my_view_class = decl.register();
                let custom_view: id = msg_send![my_view_class, new];

                let content_view_cls_name = format!("Display{display}ContentView");
                let mut content_view = ClassDecl::new(&content_view_cls_name, superclass).unwrap();
                content_view.add_method(
                    sel!(mouseDown:),
                    mouse_down as extern "C" fn(&Object, Sel, id),
                );
                content_view
                    .add_method(sel!(mouseUp:), mouse_up as extern "C" fn(&Object, Sel, id));
                content_view.add_method(
                    sel!(mouseDragged:),
                    mouse_dragged as extern "C" fn(&Object, Sel, id),
                );
                content_view
                    .add_method(sel!(keyDown:), key_down as extern "C" fn(&Object, Sel, id));
                content_view.add_ivar::<*mut c_void>("display_state");
                let content_view_class = content_view.register();
                let content_view: id = msg_send![content_view_class, new];

                (*custom_view).set_ivar("display_state", shared_state as *mut c_void);
                (*content_view).set_ivar("display_state", shared_state as *mut c_void);
                window.setContentView_(content_view);

                // 截屏
                let image = screen.capture()?;
                let (width, height) = image.dimensions();

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

                // image_view.addSubview_(custom_view);
                // 将 NSImageView 添加到窗口的内容视图
                window.contentView().addSubview_(image_view);
                window.contentView().addSubview_(custom_view);
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
