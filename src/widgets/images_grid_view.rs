use gio::glib::{self, object::Cast};
use gtk::prelude::*;
use gtk::{GridView, ListItem, ScrolledWindow, SignalListItemFactory, SingleSelection, StringList};
use std::rc::Rc;

use crate::widgets::image_card::ImageCard;

pub struct ImagesGridView {
    pub widget: ScrolledWindow,
    pub images_path_list: StringList,
    pub selected_path: Rc<RefCell<String>>,
}

use std::cell::RefCell;

impl ImagesGridView {
    pub fn new<F: Fn(&str) + 'static>(on_image_clicked: F) -> Self {
        let selected_path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let on_image_clicked: Rc<dyn Fn(&str)> = Rc::new(on_image_clicked);
        let factory = SignalListItemFactory::new();
        let images_path_list = StringList::new(&[]);
        let selection_model = SingleSelection::builder()
            .model(&images_path_list)
            .autoselect(false)
            .build();

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

        factory.connect_bind(glib::clone!(
            #[strong]
            selected_path,
            #[strong]
            on_image_clicked,
            move |_, obj: &glib::Object| {
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
                let full_path = string_obj.unwrap().string().to_string();
                let image_card: &ImageCard =
                    unsafe { list_item.data("image_card").unwrap().as_ref() };

                let is_selected = *selected_path.borrow() == full_path;
                if is_selected {
                    image_card.widget.add_css_class("image-frame--selected");
                } else {
                    image_card.widget.remove_css_class("image-frame--selected");
                }

                image_card.set_image(
                    full_path.clone(),
                    Rc::clone(&on_image_clicked),
                    Rc::clone(&selected_path),
                );
            }
        ));

        factory.connect_unbind(|_, obj: &glib::Object| {
            let list_item: &ListItem = obj.downcast_ref::<ListItem>().unwrap();
            let image_card: &ImageCard = unsafe { list_item.data("image_card").unwrap().as_ref() };
            image_card.widget.remove_css_class("image-frame--selected");
        });

        scrolled_window.set_child(Some(&grid_view));

        Self {
            widget: scrolled_window,
            images_path_list,
            selected_path,
        }
    }

    pub fn set_selected(&self, path: &str) {
        *self.selected_path.borrow_mut() = path.to_string();
        let n = self.images_path_list.n_items();
        if n == 0 {
            return;
        }
        let items: Vec<String> = (0..n)
            .filter_map(|i| {
                self.images_path_list
                    .item(i)
                    .and_then(|o| o.downcast::<gtk::StringObject>().ok())
                    .map(|s| s.string().to_string())
            })
            .collect();
        let strs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        self.images_path_list.splice(0, n, strs.as_slice());
    }
}
