let on_click_export_png_ = undefined;

export function on_click_export_png() {
    if (on_click_export_png_ !== undefined) {
        on_click_export_png_();
    }
}

export function set_on_click_export_png(func) {
    on_click_export_png_ = func;
}
