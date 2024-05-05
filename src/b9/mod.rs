pub(crate) mod css;

extern crate html_parser;

use std::thread;

use gtk::{gdk_pixbuf, gio, glib::Bytes, prelude::*, Expression};
use html_parser::{Dom, Element, Node, Result};

use self::css::Styleable;

fn parse_html_from_file() -> Result<(Node, Node)> {
    let html: String = std::fs::read_to_string("test/index.html")?;
    let dom = Dom::parse(&html)?;

    let head = find_element_by_name(&dom.children, "head").expect("Couldn't find head.");
    let body = find_element_by_name(&dom.children, "body").expect("Couldn't find body.");

    css::load_css();

    return Ok((head, body));
}

fn find_element_by_name(elements: &Vec<Node>, name: &str) -> Option<Node> {
    for element in elements {
        if element.element()?.name == name {
            return Some(element.to_owned());
        }

        if let Some(child) = find_element_by_name(&element.element()?.children, name) {
            return Some(child);
        }
    }

    None
}

pub fn build_ui() -> Result<gtk::Box> {
    css::load_css();

    let html_view = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .valign(gtk::Align::Start)
        .spacing(6)
        .css_name("body")
        .build();

    html_view.style();
    
    let (head, body) = parse_html_from_file()?;

    for element in body.element().unwrap().children.iter() {
        if let Some(element) = element.element() {
            let contents = element.children.get(0);

            render_html(element, contents, html_view.clone(), false);
        }
    }

    for element in head.element().unwrap().children.iter() {
        if let Some(element) = element.element() {
            let contents = element.children.get(0);

            render_head(element, contents, html_view.clone());
        }
    }

    Ok(html_view)
}

fn render_head(element: &Element, contents: Option<&Node>, html_view: gtk::Box) {
    match element.name.as_str() {
        "title" => {
            // set the current `Tab` name to this
        },
        "link" => {
            if let Some(href) = element.attributes.get("href") {
                if let Some(href) = href.as_ref() {
                    // got the href here
                }
            }
        }
        _ => {}
    }
}

fn render_html(
    element: &Element,
    contents: Option<&Node>,
    og_html_view: gtk::Box,
    recursive: bool,
) {
    let mut html_view = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .build();

    if !recursive {
        html_view = og_html_view.clone();
    } else {
        og_html_view.append(&html_view);
    }

    match element.name.as_str() {
        "div" => {
            let div_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .css_name("div")
                .css_classes(element.classes.clone())
                .build();

            div_box.style();

            html_view.append(&div_box);

            for child in element.children.iter() {
                match child {
                    Node::Element(el) => {
                        render_html(el, el.children.get(0), div_box.clone(), true);
                    }
                    _ => {}
                }
            }
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            if let Some(text) = contents {
                match text {
                    Node::Text(t) => {
                        let label = gtk::Label::builder()
                            .label(t)
                            .css_name(element.name.as_str())
                            .css_classes(element.classes.clone())
                            .halign(gtk::Align::Start)
                            .wrap(true)
                            .build();

                        css::perform_styling(element, &label);

                        html_view.append(&label);
                    }
                    Node::Element(el) => {
                        render_html(el, el.children.get(0), html_view, true);
                    }
                    _ => {}
                }
            }
        }
        "p" => {
            let label_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .build();

            html_view.append(&label_box);

            for child in element.children.iter() {
                match child {
                    Node::Text(_) => {
                        let label = gtk::Label::builder()
                            .label(child.text().unwrap())
                            .css_name(element.name.as_str())
                            .css_classes(element.classes.clone())
                            .halign(gtk::Align::Start)
                            .wrap(true)
                            .build();

                        css::perform_styling(element, &label);

                        label_box.append(&label);
                    }
                    Node::Element(el) => {
                        if el.name.as_str() == "a" {
                            let uri = el.attributes.get("href").unwrap().clone().unwrap();

                            let link_button = gtk::LinkButton::builder()
                                .label(el.children[0].text().unwrap())
                                .uri(uri)
                                .css_name("a")
                                .css_classes(el.classes.clone())
                                .build();

                            css::perform_styling(element, &link_button);

                            label_box.append(&link_button);
                        } else {
                            render_html(el, el.children.get(0), html_view.clone(), true);
                        }
                    }
                    Node::Comment(_) => {}
                }
            }
        }
        "ul" | "ol" => {
            let list_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .css_name(element.name.as_str())
                .build();

            css::perform_styling(element, &list_box);

            html_view.append(&list_box);

            render_list(element, list_box);
        }
        "hr" => {
            let line = gtk::Separator::builder()
                .orientation(gtk::Orientation::Horizontal)
                .css_name("hr")
                .css_classes(element.classes.clone())
                .build();
            css::perform_styling(element, &line);

            html_view.append(&line);
        }
        "img" => {
            let url = element.attributes.get("src").unwrap().clone().unwrap();

            let handle = thread::spawn(move || {
                let result = reqwest::blocking::get(url).unwrap().bytes().unwrap();
                result
            });

            let img_data = handle.join().unwrap();

            let img_stream = gio::MemoryInputStream::from_bytes(&Bytes::from(&img_data));

            let stream =
                gdk_pixbuf::Pixbuf::from_stream(&img_stream, Some(&gio::Cancellable::new()))
                    .unwrap();

            let wrapper = gtk::Box::builder().build();

            let image = gtk::Picture::builder()
                .css_name("img")
                .alternative_text(element.attributes.get("alt").unwrap().clone().unwrap())
                .css_classes(element.classes.clone())
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .can_shrink(false)
                .build();

            css::perform_styling(element, &image);

            image.set_paintable(Some(&gtk::gdk::Texture::for_pixbuf(&stream)));
            // weird workaround - https://discourse.gnome.org/t/can-shrink-on-picture-creates-empty-occupied-space/20547/2
            wrapper.append(&image);
            html_view.append(&wrapper);
        }
        "input" => {
            let input_type = element
                .attributes
                .get("type")
                .unwrap()
                .clone()
                .unwrap_or_else(|| "text".to_string());

            if input_type == "text" {
                let entry = gtk::Entry::builder()
                    .placeholder_text(
                        element
                            .attributes
                            .get("placeholder")
                            .unwrap()
                            .clone()
                            .unwrap(),
                    )
                    .css_name("input")
                    .css_classes(element.classes.clone())
                    .halign(gtk::Align::Start)
                    .build();

                css::perform_styling(element, &entry);

                html_view.append(&entry);
            }
        }
        "select" => {
            let mut strings = Vec::new();

            for child in element.children.iter() {
                match child {
                    Node::Element(el) => {
                        if el.name.as_str() == "option" {
                            // TODO: keep track of value
                            strings.push(el.children[0].text().unwrap())
                        }
                    }
                    _ => {}
                }
            }

            let dropdown = gtk::DropDown::builder()
                .model(&gtk::StringList::new(&strings[..]))
                .css_name("select")
                .css_classes(element.classes.clone())
                .halign(gtk::Align::Start)
                .build();

            css::perform_styling(element, &dropdown);

            html_view.append(&dropdown);
        }
        "textarea" => {
            let textview = gtk::TextView::builder()
                .editable(true)
                .css_name("textarea")
                .css_classes(element.classes.clone())
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();

            css::perform_styling(element, &textview);

            textview
                .buffer()
                .set_text(element.children[0].text().unwrap());

            html_view.append(&textview);
        }
        _ => {
            println!("INFO: Unknown element: {}", element.name);
        }
    }
}

fn render_list(element: &Element, list_box: gtk::Box) {
    for (i, child) in element.children.iter().enumerate() {
        match child {
            Node::Element(el) => {
                if el.name.as_str() == "li" {
                    let li = gtk::Box::builder().build();

                    let lead = gtk::Label::builder()
                        .label(match element.name.as_str() {
                            "ul" => "\t• ".to_string(),
                            "ol" => format!("\t{}. ", i + 1),
                            _ => panic!("Unknown list type"),
                        })
                        .css_name("li")
                        .css_classes(vec!["lead"])
                        .halign(gtk::Align::Start)
                        .build();

                    let label = gtk::Label::builder()
                        .label(el.children[0].text().unwrap())
                        .css_name("li")
                        .css_classes(el.classes.clone())
                        .halign(gtk::Align::Start)
                        .build();

                    css::perform_styling(element, &label);

                    li.append(&lead);
                    li.append(&label);

                    list_box.append(&li);
                } else {
                    println!("INFO: Expected li inside ul/ol, instead got: {:?}", child);
                }
            }
            _ => {
                println!("INFO: Not an element: {:?}", child);
            }
        }
    }
}
