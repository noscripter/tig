use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

pub type TuiFrame<'a> = Frame<'a>;

pub enum Transition<S> {
    None,
    Quit,
    Back,
    Push(Box<dyn View<S>>),
    Replace(Box<dyn View<S>>),
}

pub trait View<S> {
    fn title(&self) -> String;
    fn render(&mut self, f: &mut TuiFrame<'_>, area: Rect, state: &S);
    fn on_event(&mut self, ev: &Event, state: &mut S) -> Transition<S>;
}

pub struct Router<S> {
    stack: Vec<Box<dyn View<S>>>,
}

impl<S> Router<S> {
    pub fn new(root: Box<dyn View<S>>) -> Self {
        Self { stack: vec![root] }
    }

    pub fn current(&self) -> Option<&Box<dyn View<S>>> { self.stack.last() }
    pub fn current_mut(&mut self) -> Option<&mut Box<dyn View<S>>> { self.stack.last_mut() }

    pub fn render(&mut self, f: &mut TuiFrame<'_>, area: Rect, state: &S) {
        if let Some(view) = self.current_mut() {
            view.render(f, area, state);
        }
    }

    pub fn handle_event(&mut self, ev: &Event, state: &mut S) -> bool {
        let transition = match self.current_mut() {
            Some(view) => view.on_event(ev, state),
            None => Transition::Quit,
        };
        match transition {
            Transition::None => false,
            Transition::Quit => true,
            Transition::Back => { self.pop(); false }
            Transition::Push(v) => { self.push(v); false }
            Transition::Replace(v) => { self.replace(v); false }
        }
    }

    pub fn push(&mut self, v: Box<dyn View<S>>) { self.stack.push(v); }
    pub fn pop(&mut self) { if self.stack.len() > 1 { self.stack.pop(); } }
    pub fn replace(&mut self, v: Box<dyn View<S>>) {
        if !self.stack.is_empty() { self.stack.pop(); }
        self.stack.push(v);
    }
}
