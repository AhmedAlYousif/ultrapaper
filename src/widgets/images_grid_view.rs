use gio::glib::{self, object::Cast};
use gtk::prelude::*;
use gtk::{GridView, ListItem, ScrolledWindow, SignalListItemFactory, SingleSelection, StringList};

use crate::widgets::image_card::ImageCard;

pub struct ImagesGridView {
    pub widget: ScrolledWindow,
    pub images_path_list: StringList,
}

impl ImagesGridView {
    pub fn new(on_image_clicked: fn(path: &str)) -> Self {
        let factory = SignalListItemFactory::new();
        let images_path_list = StringList::new(&[]);
        let selection_model = SingleSelection::builder().model(&images_path_list).build();

        let scrolled_window = ScrolledWindow::builder()
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();
        scrolled_window.add_css_class("image-browser-scroll");
        let grid_view = GridView::builder()
            .model(&selection_model)
            .factory(&factory)
            .min_columns(2)
            .max_columns(6)
            .single_click_activate(true)
            .build();
        grid_view.add_css_class("image-browser-grid");

        factory.connect_setup(|_, obj: &glib::Object| {
            let list_item: &ListItem = obj.downcast_ref::<ListItem>().unwrap();
            let image_card = ImageCard::new();
            list_item.set_child(Some(&image_card.widget));
            unsafe {
                list_item.set_data("image_card", image_card);
            }
        });
        factory.connect_bind(move |_, obj: &glib::Object| {
            let list_item: &ListItem = obj.downcast_ref::<ListItem>().unwrap();
            let item_obj = list_item.item();
            if item_obj.is_none() {
                return;
            }
            let item = item_obj.unwrap();

            let string_obj = item.downcast_ref::<gtk::StringObject>();
            if string_obj.is_none() {
                return;
            }
            let full_path = string_obj.unwrap().string();

            let image_card: &ImageCard = unsafe { list_item.data("image_card").unwrap().as_ref() };

            image_card.set_image(full_path.to_string(), on_image_clicked);
        });

        scrolled_window.set_child(Some(&grid_view));

        Self {
            widget: scrolled_window,
            images_path_list,
        }
    }
}
