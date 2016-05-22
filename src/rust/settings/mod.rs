use std::thread;
use std::sync::mpsc;

use gtk::prelude::*;
use gtk::{
    Orientation, PositionType,
    Window, WindowType,
    self
};

use mechanics::SettingsUpdate;
use mechanics::PlayerSettings;

macro_rules! add_slider {
    ($container: expr, $channel: expr, $default: expr, $update: ident) => ({
        add_slider!($container, $channel, $default, $update, 0.0, 1000.0, 10.0)
    });
    ($container: expr, $channel: expr, $default: expr, $update: ident,
            $min: expr, $max: expr, $step: expr) => ({
        let slider = gtk::Scale::new_with_range(Orientation::Horizontal, $min, $max, $step);
        let channel = $channel.clone();
        slider.add_mark($default, PositionType::Bottom, Some(stringify!($update)));
        slider.set_value($default);
        $container.add(&slider);
        slider.connect_change_value(move |_, _, value| {
            if let Err(mpsc::SendError(v)) = channel.send(SettingsUpdate::$update(value)) {
                println!("Couldn't send value: {:?}", v);
            }
            Inhibit(false)
        });
    })
}

fn run(channel: mpsc::Sender<SettingsUpdate>) {
    if let Err(e) = gtk::init() {
        println!("Failed to initialize GTK: {:?}", e);
        return;
    }

    let window = Window::new(WindowType::Toplevel);

    window.set_title("b-w-adventures settings");
    window.set_default_size(200, 100);

    let scrolled = gtk::ScrolledWindow::new(None, None);
    let container = gtk::Box::new(Orientation::Vertical, 10);
    window.add(&scrolled);
    scrolled.add(&container);

    let default = PlayerSettings::default();

    add_slider!(container, channel, default.weight, Weight, 0.0, 10.0, 0.1);
    add_slider!(container, channel, default.input_force, InputForce);
    add_slider!(container, channel, default.jump_boost, JumpBoost);
    add_slider!(container, channel, default.wall_boost_x, WallBoostX);
    add_slider!(container, channel, default.wall_boost_y, WallBoostY);
    add_slider!(container, channel, default.gravity_force, GravityForce);
    add_slider!(container, channel, default.drag_constant, DragConstant, 0.0, 1.0, 0.01);
    add_slider!(container, channel, default.tick_constant, TickConstant, 0.0, 20.0, 0.1);
    // add_slider!(container, channel, default.jump_duration, JumpDuration);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}

pub fn exec_threaded(channel: mpsc::Sender<SettingsUpdate>) {
    thread::spawn(||run(channel));
}
