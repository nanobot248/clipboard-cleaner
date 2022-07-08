use std::sync::Arc;
use gtk::{Application, TextBuffer, TextView};
use gtk::prelude::{TextBufferExt, TextViewExt};

pub struct ContentTextbox {
    app: Arc<Application>,
    textbox: Arc<TextView>,
}

impl ContentTextbox {
    pub fn new(app: Arc<Application>, textbox: Arc<TextView>) -> Arc<ContentTextbox> {
        let result = Arc::new(ContentTextbox {
            app: app.clone(),
            textbox: textbox.clone(),
        });

        textbox.set_editable(false);

        return result.clone();
    }

    pub fn has_content(&self) -> bool {
        let buffer = self.textbox.buffer();
        if let Some(buffer) = buffer {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
            if let Some(text) = text {
                return text.len() > 0;
            }
        }
        return false;
    }

    pub fn content(&self) -> Option<String> {
        let buffer = self.textbox.buffer();
        if let Some(buffer) = buffer {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
            if let Some(text) = text {
                return Some(text.to_string());
            }
        }
        return None;
    }

    pub fn set_content(&self, content: &str) {
        let buffer = self.textbox.buffer();
        if let Some(buffer) = buffer {
            buffer.set_text(content);
        } else {
            let text_buffer = TextBuffer::builder().text(content);
            self.textbox.set_buffer(Some(&text_buffer.build()));
        }
    }
}