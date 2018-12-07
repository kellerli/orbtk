use enums::ParentType;
use event::{Key, KeyEventHandler, MouseEventHandler};
use properties::{Focused, Label, Offset, Point, TextSelection, WaterMark};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use theme::Selector;
use widget::{
    Container, Context, Cursor, ScrollViewer, SharedProperty, Stack, State, Template,
    WaterMarkTextBlock, Widget, WidgetKey
};

/// The `TextBoxState` handles the text processing of the `TextBox` widget.
#[derive(Default)]
pub struct TextBoxState {
    text: RefCell<String>,
    focused: Cell<bool>,
    updated: Cell<bool>,
    selection_start: Cell<usize>,
    selection_length: Cell<usize>,
}

impl Into<Rc<State>> for TextBoxState {
    fn into(self) -> Rc<State> {
        Rc::new(self)
    }
}

impl TextBoxState {
    fn click(&self, point: Point) {
        println!("Clicked text box point: ({}, {})", point.x, point.y);
    }

    fn update_selection_start(&self, selection: i32) {
        self.selection_start
            .set(selection.max(0).min(self.text.borrow().len() as i32) as usize);
    }

    fn update_text(&self, key: Key) -> bool {
        if !self.focused.get() {
            return false;
        }

        match <Option<u8>>::from(key) {
            Some(byte) => {
                (*self.text.borrow_mut()).insert(self.selection_start.get(), byte as char);
                self.update_selection_start(self.selection_start.get() as i32 + 1);
            }
            None => match key {
                Key::Left => {
                    self.update_selection_start(self.selection_start.get() as i32 - 1);
                    self.selection_length.set(0);
                }
                Key::Right => {
                    self.update_selection_start(self.selection_start.get() as i32 + 1);
                    self.selection_length.set(0);
                }
                Key::Backspace => {
                    if self.text.borrow().len() > 0 {
                        if self.selection_start.get() > 0 {
                            for _ in 0..(self.selection_length.get() + 1) {
                                (*self.text.borrow_mut()).remove(self.selection_start.get() - 1);
                            }
                            self.update_selection_start(self.selection_start.get() as i32 - 1);
                        }
                    }
                }
                _ => {}
            },
        }

        self.updated.set(true);

        true
    }
}

impl State for TextBoxState {
    fn update(&self, context: &mut Context) {
        if let Ok(focused) = context.widget.borrow_property::<Focused>() {
            self.focused.set(focused.0);
        }

        if let Ok(label) = context.widget.borrow_mut_property::<Label>() {
            if label.0 != *self.text.borrow() {
                if self.updated.get() {
                    label.0 = self.text.borrow().clone();
                } else {
                    let text_length = self.text.borrow().len();
                    let origin_text_length = label.0.len();
                    let delta = text_length as i32 - origin_text_length as i32;

                    *self.text.borrow_mut() = label.0.clone();

                    // adjust cursor position after labe is changed from outside
                    if text_length < origin_text_length {
                        self.update_selection_start(self.selection_start.get() as i32 - delta);
                    } else {
                        self.update_selection_start(self.selection_start.get() as i32 + delta);
                    }
                }

                self.updated.set(false);
            }
        }

        if let Ok(selection) = context.widget.borrow_mut_property::<TextSelection>() {
            selection.start_index = self.selection_start.get();
            selection.length = self.selection_length.get();
        }
    }

    fn update_post_layout(&self, context: &mut Context) {
        if let Ok(offset) = context.widget.borrow_mut_property_by_key::<Offset>("Cursor") {
            println!("Curor Offset: {}", offset.0);
        }
    }
}

/// The `TextBox` represents a single line text input widget.
///
/// # Shared Properties
///
/// * `Label` - String used to display the text of the text box.
/// * `Watermark` - String used to display a placeholder text if `Label` string is empty.
/// * `Selector` - CSS selector with  element name `textbox`, used to request the theme of the widget.
/// * `TextSelection` - Represents the current selection of the text used by the cursor.
/// * `Focused` - Defines if the widget is focues and handles the current text input.
///
/// # Others
///
/// * `TextBoxState` - Handles the inner state of the widget.
/// * `KeyEventHandler` - Process the text input of the control if it is focuesd.
pub struct TextBox;

impl Widget for TextBox {
    fn create() -> Template {
        let label = SharedProperty::new(Label::default());
        let water_mark = SharedProperty::new(WaterMark::default());
        let selector = SharedProperty::new(Selector::from("textbox"));
        let selection = SharedProperty::new(TextSelection::default());
        let offset = SharedProperty::new(Offset::default());
        let focused = SharedProperty::new(Focused(false));
        let state = Rc::new(TextBoxState::default());
        let click_state = state.clone();
        let text_block_key = WidgetKey::from("TextBlock");
        let cursor_key = WidgetKey::from("Cursor");
        
        Template::default()
            .as_parent_type(ParentType::Single)
            .with_property(Focused(false))
            .with_child(
                Container::create()
                    .with_child(
                        Stack::create()
                            .with_child(
                                ScrollViewer::create()
                                    .with_child(
                                        WaterMarkTextBlock::create()
                                            .with_shared_property(label.clone())
                                            .with_shared_property(selector.clone())
                                            .with_shared_property(water_mark.clone())
                                            .with_key(text_block_key.clone())
                                    )
                                    .with_shared_property(offset.clone()),
                            )
                            .with_child(
                                Cursor::create()
                                    .with_shared_property(label.clone())
                                    .with_shared_property(selection.clone())
                                    .with_shared_property(offset.clone())
                                    .with_shared_property(focused.clone())
                                    .with_key(cursor_key.clone()),
                            )
                            .with_event_handler(MouseEventHandler::default().on_mouse_down(
                                Rc::new(move |pos: Point| -> bool {
                                    click_state.click(pos);
                                    false
                                }),
                            )),
                    )
                    .with_shared_property(selector.clone()),
            )
            .with_state(state.clone())
            .with_debug_name("TextBox")
            .with_shared_property(label)
            .with_shared_property(selector)
            .with_shared_property(water_mark)
            .with_shared_property(selection)
            .with_shared_property(offset)
            .with_shared_property(focused)
            .with_child_key(text_block_key)
            .with_child_key(cursor_key)
            .with_event_handler(
                KeyEventHandler::default()
                    .on_key_down(Rc::new(move |key: Key| -> bool { state.update_text(key) })),
            )
    }
}
