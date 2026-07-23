use std::any::Any;

use crate::widget::ActionFlags;

#[allow(unused)]
pub trait MessageReceiver {
    fn handle_message(&mut self, message: &mut Message, broker: &mut MessageBroker) -> ActionFlags {
        ActionFlags::None
    }
}

#[derive(Debug)]
pub struct Message {
    data: Box<dyn Any>,
    propergate: bool,
}

impl Message {
    #[inline]
    pub fn new(data: impl Any) -> Self {
        Self {
            data: Box::new(data),
            propergate: true,
        }
    }

    #[inline]
    pub fn propergate(&self) -> bool {
        self.propergate
    }

    #[inline]
    pub fn stop_propergation(&mut self) {
        self.propergate = false;
    }

    #[inline]
    pub fn data<T: Any>(&self) -> Option<&T> {
        self.data.as_ref().downcast_ref::<T>()
    }

    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        self.data.as_ref().is::<T>()
    }

    #[inline]
    pub fn is_consumed(&self) -> bool {
        self.data.as_ref().is::<Consumed>()
    }

    #[inline]
    pub fn consume<T: Any>(&mut self) -> Option<Box<T>> {
        if !self.data.is::<T>() {
            return None;
        }

        let mut temp: Box<dyn Any> = Box::new(Consumed);

        std::mem::swap(&mut temp, &mut self.data);
        self.stop_propergation();

        temp.downcast::<T>().ok()
    }
}

#[derive(Debug)]
struct Consumed;


#[derive(Debug, Default)]
pub struct MessageBroker {
    messages: Vec<Message>,
}

impl MessageBroker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn has_pending(&self) -> bool {
        !self.messages.is_empty()
    }

    #[inline]
    pub fn dispatch(&mut self, data: impl Any) {
        self.dispatch_message(Message::new(data));
    }

    #[inline]
    pub fn dispatch_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn deliver(&mut self, receivers: &mut [&mut dyn MessageReceiver]) -> ActionFlags {
        let mut flags = ActionFlags::None;

        if self.messages.is_empty() {
            return flags;
        }

        let mut messages = Vec::new();
        while !self.messages.is_empty() {
            std::mem::swap(&mut messages, &mut self.messages);

            for message in &mut messages {
                for receiver in receivers.iter_mut() {
                    if !message.propergate() {
                        break;
                    }
                    flags |= receiver.handle_message(message, self);
                }
            }

            messages.clear();
        }

        flags
    }
}
