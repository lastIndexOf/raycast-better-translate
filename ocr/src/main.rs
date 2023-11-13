use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSImage,
    NSImageView, NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use core_graphics::display::{CGDisplay, CGDisplayBounds};
use screenshots::Screen;

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
                // 截屏
                let mut image = screen.capture().unwrap();
                // let (width, height) = image.dimensions();
                // let raw_image = image.into_raw();

                // 截屏区域
                // let image = screen.capture_area(
                //     0,
                //     0,
                //     screen.display_info.width,
                //     screen.display_info.height,
                // )?;

                image.save("./target.png")?;

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

                let path = NSString::alloc(nil).init_str("./target.png");

                // 使用NSImage加载图片
                let image = NSImage::alloc(nil).initByReferencingFile_(path);

                // 创建 NSImageView 并设置图像
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
