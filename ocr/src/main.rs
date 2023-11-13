use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSWindow,
    NSWindowStyleMask,
};
use cocoa::base::{nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize};
use core_graphics::display::{CGDisplay, CGDisplayBounds, CGMainDisplayID};

fn main() -> anyhow::Result<()> {
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
            let screen_rect = CGDisplayBounds(display);
            let screen_size = NSSize::new(
                screen_rect.size.width as f64,
                screen_rect.size.height as f64,
                // 336.0, 189.0,
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

            // 设置窗口层级，使其覆盖Dock和菜单栏
            window.setLevel_(cocoa::appkit::NSMainMenuWindowLevel as i64 + 2);
            window.setOpaque_(NO);
            // window.setBackgroundColor_(cocoa::appkit::NSColor::clearColor(nil));
            window.makeKeyAndOrderFront_(nil);

            offset.0 += screen_rect.size.width;
            offset.1 = Some(screen_rect.size.height);
        }

        app.run();

        anyhow::Ok(())
    }
}
