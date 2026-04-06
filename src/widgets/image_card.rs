use gio::File;
use gtk::prelude::*;
use gtk::{Align, Box, ContentFit, GestureClick, Orientation, Picture};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ImageCard {
    pub widget: Box,
    picture: Picture,
}

impl ImageCard {
    pub fn new() -> Self {
        let frame = Box::builder()
            .orientation(Orientation::Vertical)
            .vexpand(false)
            .hexpand(false)
            .width_request(240)
            .height_request(140)
            .spacing(0)
            .build();
        frame.add_css_class("image-frame");

        let picture = Picture::builder()
            .valign(Align::Center)
            .halign(Align::Center)
            .vexpand(false)
            .hexpand(false)
            .content_fit(ContentFit::Contain)
            .width_request(240)
            .height_request(140)
            .build();
        picture.add_css_class("image-thumb");

        frame.append(&picture);

        Self {
            widget: frame,
            picture: picture,
        }
    }

    pub fn set_image(
        &self,
        path: String,
        on_click: Rc<dyn Fn(&str)>,
        selected_path: Rc<RefCell<String>>,
    ) {
        let image_file = File::for_path(&path);
        self.picture.set_file(Some(&image_file));

        let mut parts: Vec<&str> = path.split('/').collect();
        self.picture.set_tooltip_text(parts.pop());

        let controller = GestureClick::new();
        let frame = self.widget.clone();

        controller.connect_pressed(move |_gesture, _n_press, _x, _y| {
            *selected_path.borrow_mut() = path.clone();
            frame.add_css_class("image-frame--selected");
            on_click(&path);
        });

        self.picture.add_controller(controller);
    }
}
