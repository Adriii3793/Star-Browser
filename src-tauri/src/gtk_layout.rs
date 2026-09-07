use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

const LAYER_NAME: &str = "star-layer";
const BASE_NAME: &str = "star-base";
const MASK_NAME: &str = "star-corner-mask";

#[derive(Clone, Copy, PartialEq, Eq)]
struct Insets {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Insets {
    const FILL: Self = Self {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Corners {
    pub radius: i32,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

impl Corners {
    pub const SQUARE: Self = Self {
        radius: 0,
        bottom_left: false,
        bottom_right: false,
    };

    fn rounds_anything(self) -> bool {
        self.radius > 0 && (self.bottom_left || self.bottom_right)
    }

    fn floating(self) -> bool {
        self.radius > 0
    }
}

const RESIZE_GRIP: i32 = 8;
const RESIZE_GRIP_CORNER: i32 = 14;

fn page_input_region(
    (x, y, width, height): (i32, i32, i32, i32),
    layer: (i32, i32),
    corners: Corners,
) -> Option<gtk::cairo::Region> {
    if !corners.floating() || width <= 0 || height <= 0 {
        return None;
    }
    
    let gap = |grip: i32, distance: i32| (grip - distance).clamp(0, grip);
    let left = gap(RESIZE_GRIP, x);
    let right = gap(RESIZE_GRIP, layer.0 - (x + width));
    let bottom = gap(RESIZE_GRIP, layer.1 - (y + height));

    if left == 0 && right == 0 && bottom == 0 {
        return None;
    }

    let region = gtk::cairo::Region::create_rectangle(&gtk::cairo::RectangleInt::new(
        0, 0, width, height,
    ));
    let cut = |region: &gtk::cairo::Region, rect: gtk::cairo::RectangleInt| {
        let _ = region.subtract_rectangle(&rect);
    };
    if left > 0 {
        cut(&region, gtk::cairo::RectangleInt::new(0, 0, left, height));
    }
    if right > 0 {
        cut(
            &region,
            gtk::cairo::RectangleInt::new(width - right, 0, right, height),
        );
    }
    if bottom > 0 {
        cut(
            &region,
            gtk::cairo::RectangleInt::new(0, height - bottom, width, bottom),
        );
    }
    
    let corner = RESIZE_GRIP_CORNER.min(width).min(height);
    let bottom_corner = gap(RESIZE_GRIP_CORNER, layer.1 - (y + height));
    if bottom_corner > 0 {
        if gap(RESIZE_GRIP_CORNER, x) > 0 {
            cut(
                &region,
                gtk::cairo::RectangleInt::new(0, height - corner, corner, corner),
            );
        }
        if gap(RESIZE_GRIP_CORNER, layer.0 - (x + width)) > 0 {
            cut(
                &region,
                gtk::cairo::RectangleInt::new(width - corner, height - corner, corner, corner),
            );
        }
    }
    Some(region)
}

#[cfg(test)]
mod input_region_tests {
    use super::*;

    fn rounded() -> Corners {
        Corners {
            radius: 12,
            bottom_left: true,
            bottom_right: true,
        }
    }

    fn squared() -> Corners {
        Corners {
            radius: 0,
            bottom_left: false,
            bottom_right: false,
        }
    }

    const PAGE: (i32, i32, i32, i32) = (1, 86, 998, 713);
    const LAYER: (i32, i32) = (1000, 800);

    #[test]
    fn a_maximized_window_leaves_the_page_every_pixel() {
        assert!(page_input_region(PAGE, LAYER, squared()).is_none());
    }

    #[test]
    fn the_window_edges_stop_being_the_pages_and_its_middle_is_untouched() {
        let region = page_input_region(PAGE, LAYER, rounded()).unwrap();
        assert!(!region.contains_point(0, 300));
        assert!(!region.contains_point(6, 300));
        assert!(!region.contains_point(997, 300));
        assert!(!region.contains_point(500, 712));
        assert!(region.contains_point(500, 300));
        assert!(region.contains_point(20, 300));
        assert!(region.contains_point(500, 0));
    }

    #[test]
    fn the_corner_grips_are_deeper_than_the_edges() {
        let region = page_input_region(PAGE, LAYER, rounded()).unwrap();
        assert!(!region.contains_point(10, 703));
        assert!(!region.contains_point(987, 703));
        assert!(region.contains_point(10, 400));
    }

    #[test]
    fn a_side_the_page_does_not_reach_keeps_its_clicks() {
        let with_panel = (1, 86, 638, 713);
        let region = page_input_region(with_panel, LAYER, rounded()).unwrap();
        assert!(region.contains_point(637, 300));
        assert!(!region.contains_point(0, 300));
    }
}

struct CornerMasks {
    left: gtk::DrawingArea,
    right: gtk::DrawingArea,
}

#[derive(Clone, Copy)]
enum MaskSide {
    Left,
    Right,
}

thread_local! {
    static MASKS: RefCell<Option<CornerMasks>> = const { RefCell::new(None) };
    static WINDOW_CORNERS: RefCell<Corners> = const { RefCell::new(Corners::SQUARE) };
}

fn corner_mask(side: MaskSide) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_widget_name(MASK_NAME);
    area.set_app_paintable(true);
    area.connect_draw(move |area, cr| {
        let width = area.allocated_width() as f64;
        let height = area.allocated_height() as f64;
        let (cx, cy) = match side {
            MaskSide::Left => (width, 0.0),
            MaskSide::Right => (0.0, 0.0),
        };
        cr.set_operator(gtk::cairo::Operator::Clear);
        cr.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
        cr.rectangle(0.0, 0.0, width, height);
        cr.new_sub_path();
        cr.arc(cx, cy, width.min(height), 0.0, std::f64::consts::PI * 2.0);
        let _ = cr.fill();
        gtk::glib::Propagation::Stop
    });
    area
}

fn sync_corner_masks(layer: &gtk::Layout) {
    if std::env::var_os("STAR_DBG_NO_MASKS").is_some() {
        return;
    }
    let corners = WINDOW_CORNERS.with(|c| *c.borrow());
    let allocation = layer.allocation();
    let (width, height) = (allocation.width(), allocation.height());
    let radius = corners
        .radius
        .min(width / 2)
        .min(height / 2)
        .max(0);
    let wanted = corners.rounds_anything() && radius > 0;

    MASKS.with(|cell| {
        let mut cell = cell.borrow_mut();
        if !wanted {
            if let Some(masks) = cell.as_ref() {
                masks.left.hide();
                masks.right.hide();
            }
            return;
        }

        let masks = cell.get_or_insert_with(|| {
            let left = corner_mask(MaskSide::Left);
            let right = corner_mask(MaskSide::Right);
            layer.put(&left, 0, 0);
            layer.put(&right, 0, 0);
            CornerMasks { left, right }
        });

        for (area, wants, x) in [
            (&masks.left, corners.bottom_left, 0),
            (&masks.right, corners.bottom_right, width - radius),
        ] {
            if !wants {
                area.hide();
                continue;
            }
            layer.move_(area, x, height - radius);
            area.set_size_request(radius, radius);
            area.show();
            if let Some(window) = area.window() {
                window.raise();
                window.input_shape_combine_region(&gtk::cairo::Region::create(), 0, 0);
            }
            area.queue_draw();
        }
    });
}

#[derive(Clone, Copy)]
struct Placed {
    insets: Option<Insets>,
    rect: (i32, i32, i32, i32),
    corners: Option<Corners>,
    allocated: bool,
}

thread_local! {
    static PLACEMENTS: RefCell<HashMap<usize, Placed>> = RefCell::new(HashMap::new());
    static MAP_REPAIRS: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
}

fn key_of(widget: &gtk::Widget) -> usize {
    widget.as_ptr() as usize
}

fn is_webview(widget: &gtk::Widget) -> bool {
    widget.type_().name().starts_with("WebKit")
}

pub fn install(vbox: &gtk::Box) {
    ensure_layer(vbox).show();
}

fn ensure_layer(vbox: &gtk::Box) -> gtk::Layout {
    if let Some(layer) = find_layer(vbox) {
        return layer;
    }

    let layer = gtk::Layout::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    layer.set_widget_name(LAYER_NAME);

    let existing = vbox.children();
    vbox.pack_start(&layer, true, true, 0);

    for child in existing {
        if !is_webview(&child) {
            continue;
        }
        child.set_widget_name(BASE_NAME);
        let _ = adopt(&layer, &child);
        remember(&child, Some(Insets::FILL));
    }

    layer.connect_size_allocate(|layer, allocation| {
        relayout(layer, allocation.width(), allocation.height());
    });

    vbox.connect_add(|_, child| {
        if child.widget_name().as_str() == LAYER_NAME || !is_webview(child) {
            return;
        }
        child.set_child_visible(false);
    });

    layer
}

fn find_layer(vbox: &gtk::Box) -> Option<gtk::Layout> {
    vbox.children().into_iter().find_map(|child| {
        (child.widget_name().as_str() == LAYER_NAME)
            .then(|| child.downcast::<gtk::Layout>().ok())
            .flatten()
    })
}

fn remember(widget: &gtk::Widget, insets: Option<Insets>) {
    PLACEMENTS.with(|placements| {
        let mut placements = placements.borrow_mut();
        let entry = placements.entry(key_of(widget)).or_insert(Placed {
            insets,
            rect: (0, 0, 0, 0),
            corners: None,
            allocated: false,
        });
        entry.insets = insets;
    });
}

fn adopt(layer: &gtk::Layout, widget: &gtk::Widget) -> bool {
    let layer_widget: &gtk::Widget = layer.upcast_ref();
    if let Some(parent) = widget.parent() {
        if key_of(&parent) == key_of(layer_widget) {
            widget.set_child_visible(true);
            return false;
        }
        let was_realized = widget.is_realized();
        if let Some(container) = parent.downcast_ref::<gtk::Container>() {
            container.remove(widget);
        }
        layer.put(widget, 0, 0);
        widget.set_child_visible(true);
        return was_realized;
    }
    layer.put(widget, 0, 0);
    widget.set_child_visible(true);
    false
}

fn layer_for(widget: &gtk::Widget) -> Option<gtk::Layout> {
    let parent = widget.parent()?;
    if parent.widget_name().as_str() == LAYER_NAME {
        return parent.downcast::<gtk::Layout>().ok();
    }
    let vbox = parent.downcast::<gtk::Box>().ok()?;
    Some(ensure_layer(&vbox))
}

fn apply(
    layer: &gtk::Layout,
    widget: &gtk::Widget,
    rect: (i32, i32, i32, i32),
    corners: Option<Corners>,
) -> bool {
    let (x, y, width, height) = rect;
    if let Some(corners) = corners.filter(|_| std::env::var_os("STAR_DBG_NO_INPUT_SHAPE").is_none()) {
        if let Some(window) = widget.window() {
            let allocation = layer.allocation();
            let region = page_input_region(
                rect,
                (allocation.width(), allocation.height()),
                corners,
            )
            .unwrap_or_else(|| {
                gtk::cairo::Region::create_rectangle(&gtk::cairo::RectangleInt::new(
                    0, 0, width, height,
                ))
            });
            window.input_shape_combine_region(&region, 0, 0);
        }
    }
    layer.move_(widget, x, y);
    if widget.size_request() != (width, height) {
        widget.set_size_request(width, height);
    }
    let allocated = widget.get_visible();
    if allocated {
        let allocation = gtk::Allocation::new(x, y, width, height);
        if widget.allocation() != allocation {
            widget.size_allocate(&allocation);
        }
    }
    layer.queue_draw();
    allocated
}

fn arm_map_repair(widget: &gtk::Widget) {
    let already = MAP_REPAIRS.with(|armed| !armed.borrow_mut().insert(key_of(widget)));
    if already {
        return;
    }
    widget.connect_map(|widget| {
        let widget: gtk::Widget = widget.clone().upcast();
        let key = key_of(&widget);
        let pending = PLACEMENTS.with(|placements| {
            placements
                .borrow()
                .get(&key)
                .filter(|placed| !placed.allocated)
                .map(|placed| (placed.rect, placed.corners))
        });
        let Some((rect, corners)) = pending else {
            return;
        };
        let Some(layer) = layer_for(&widget) else {
            return;
        };
        let allocated = apply(&layer, &widget, rect, corners);
        PLACEMENTS.with(|placements| {
            if let Some(placed) = placements.borrow_mut().get_mut(&key) {
                placed.allocated = allocated;
            }
        });
    });
}

fn relayout(layer: &gtk::Layout, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }
    let children = layer.children();
    sync_corner_masks(layer);

    PLACEMENTS.with(|placements| {
        let mut placements = placements.borrow_mut();

        let live: HashSet<usize> = children.iter().map(key_of).collect();
        placements.retain(|key, _| live.contains(key));
        MAP_REPAIRS.with(|armed| armed.borrow_mut().retain(|key| live.contains(key)));

        for child in &children {
            if child.widget_name().as_str() == MASK_NAME {
                continue;
            }
            let key = key_of(child);
            if child.widget_name().as_str() == BASE_NAME {
                placements.entry(key).or_insert(Placed {
                    insets: Some(Insets::FILL),
                    rect: (0, 0, 0, 0),
                    corners: None,
                    allocated: false,
                });
            }
            let Some(placed) = placements.get_mut(&key) else {
                continue;
            };
            let Some(insets) = placed.insets else {
                continue;
            };
            let rect = (
                insets.left,
                insets.top,
                (width - insets.left - insets.right).max(1),
                (height - insets.top - insets.bottom).max(1),
            );
            if rect == placed.rect && placed.allocated {
                continue;
            }
            placed.rect = rect;
            placed.allocated = apply(layer, child, rect, placed.corners);
        }
    });
}

pub fn place(
    webview: &tauri::Webview,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    follow_window: bool,
    restore_load: bool,
    corners: Option<Corners>,
) {
    let x = x.round() as i32;
    let y = y.round() as i32;
    let width = (width.round() as i32).max(1);
    let height = (height.round() as i32).max(1);

    let handle = webview.clone();

    let _ = webview.with_webview(move |platform| {
        let widget: gtk::Widget = platform.inner().upcast();
        let Some(layer) = layer_for(&widget) else {
            return;
        };

        if adopt(&layer, &widget) && restore_load {
            let handle = handle.clone();
            gtk::glib::idle_add_local_once(move || {
                let _ = handle.reload();
            });
        }
        arm_map_repair(&widget);

        let allocation = layer.allocation();
        let measurable = allocation.width() > 1 && allocation.height() > 1;
        let insets = (follow_window && measurable).then(|| Insets {
            left: x,
            top: y,
            right: allocation.width() - (x + width),
            bottom: allocation.height() - (y + height),
        });

        let rect = (x, y, width, height);
        let key = key_of(&widget);
        let (changed, was_allocated) = PLACEMENTS.with(|placements| {
            match placements.borrow().get(&key) {
                Some(previous) => (
                    previous.rect != rect
                        || previous.insets != insets
                        || previous.corners != corners
                        || !previous.allocated,
                    previous.allocated,
                ),
                None => (true, false),
            }
        });
        let allocated = if changed {
            apply(&layer, &widget, rect, corners)
        } else {
            was_allocated
        };
        PLACEMENTS.with(|placements| {
            placements.borrow_mut().insert(
                key,
                Placed {
                    insets,
                    rect,
                    corners,
                    allocated,
                },
            );
        });
        if let Some(corners) = corners {
            let moved = WINDOW_CORNERS.with(|current| current.replace(corners) != corners);
            if moved || changed {
                sync_corner_masks(&layer);
            }
        }
    });
}

pub fn raise(webview: &tauri::Webview) {
    let _ = webview.with_webview(|platform| {
        let widget: gtk::Widget = platform.inner().upcast();
        if let Some(window) = widget.window() {
            window.raise();
        }
        if let Some(layer) = layer_for(&widget) {
            sync_corner_masks(&layer);
        }
    });
}

pub fn forget(webview: &tauri::Webview) {
    let _ = webview.with_webview(|platform| {
        let widget: gtk::Widget = platform.inner().upcast();
        PLACEMENTS.with(|placements| {
            placements.borrow_mut().remove(&key_of(&widget));
        });
        MAP_REPAIRS.with(|armed| {
            armed.borrow_mut().remove(&key_of(&widget));
        });
    });
}

static TILED: AtomicBool = AtomicBool::new(false);

fn tiled_flags() -> gtk::gdk::WindowState {
    gtk::gdk::WindowState::TILED
        | gtk::gdk::WindowState::LEFT_TILED
        | gtk::gdk::WindowState::RIGHT_TILED
        | gtk::gdk::WindowState::TOP_TILED
        | gtk::gdk::WindowState::BOTTOM_TILED
}

pub fn is_tiled() -> bool {
    TILED.load(Ordering::Relaxed)
}

pub fn track_window_state(window: &gtk::ApplicationWindow) {
    let widget: &gtk::Widget = window.upcast_ref();
    if let Some(gdk_window) = widget.window() {
        TILED.store(gdk_window.state().intersects(tiled_flags()), Ordering::Relaxed);
    }
    window.connect_window_state_event(|_, event| {
        TILED.store(
            event.new_window_state().intersects(tiled_flags()),
            Ordering::Relaxed,
        );
        gtk::glib::Propagation::Proceed
    });
}

pub fn on_key_press<F>(webview: &tauri::Webview, handler: F)
where
    F: Fn(u32, bool, bool, bool) -> bool + Send + 'static,
{
    let _ = webview.with_webview(move |platform| {
        let widget: gtk::Widget = platform.inner().upcast();
        widget.connect_key_press_event(move |_, event| {
            let state = event.state();
            let consumed = handler(
                *event.keyval(),
                state.contains(gtk::gdk::ModifierType::CONTROL_MASK),
                state.contains(gtk::gdk::ModifierType::SHIFT_MASK),
                state.contains(gtk::gdk::ModifierType::MOD1_MASK),
            );
            if consumed {
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
    });
}
