use std::borrow::Borrow;

use iced::{event, Color, Element, Event, Length, Point, Rectangle};
use iced_native::{
    layout, mouse, overlay, renderer,
    widget::{self, tree, Operation, Tree},
    Clipboard, Layout, Shell, Widget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Layer {
    #[default]
    Background,
    Primary,
    Secondary,
}

pub trait CosmicWidget<M, R>: iced_native::Widget<M, R>
where
    R: iced_native::Renderer,
{
    /// indicates if the widget is a container
    fn is_container(&self) -> bool {
        false
    }

    /// the UI layer of the widget
    fn layer(&self) -> Layer {
        Layer::default()
    }

    /// indicates the UI layer of the widget's children
    fn child_layer(&self) -> Layer {
        if self.is_container() {
            match self.layer() {
                Layer::Background => Layer::Primary,
                Layer::Primary | Layer::Secondary => Layer::Secondary,
                // TODO log a warning if the layer is already secondary?
            }
        } else {
            self.layer()
        }
    }

    /// sets the layer for the widget and its children
    fn set_layer(&mut self, layer: Layer);
}

// TODO can't seem to implement a wrapper due to private types

pub enum CosmicWidgetWrapper<'a, Message, Renderer> {
    Iced(Element<'a, Message, Renderer>),
    Cosmic(CosmicElement<'a, Message, Renderer>),
}

pub struct CosmicElement<'a, Message, Renderer> {
    widget: Box<dyn CosmicWidget<Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> CosmicElement<'a, Message, Renderer> {
    /// Creates a new [`Element`] containing the given [`Widget`].
    pub fn new(widget: impl CosmicWidget<Message, Renderer> + 'a) -> Self
    where
        Renderer: iced_native::Renderer,
    {
        Self {
            widget: Box::new(widget),
        }
    }

    #[must_use]
    /// Returns a reference to the [`Widget`] of the [`Element`],
    pub fn as_widget(&self) -> &dyn CosmicWidget<Message, Renderer> {
        self.widget.as_ref()
    }

    /// Returns a mutable reference to the [`Widget`] of the [`Element`],
    pub fn as_widget_mut(&mut self) -> &mut dyn CosmicWidget<Message, Renderer> {
        self.widget.as_mut()
    }

    /// Applies a transformation to the produced message of the [`Element`].
    ///
    /// This method is useful when you want to decouple different parts of your
    /// UI and make them __composable__.
    ///
    /// # Example
    /// Imagine we want to use [our counter](index.html#usage). But instead of
    /// showing a single counter, we want to display many of them. We can reuse
    /// the `Counter` type as it is!
    ///
    /// We use composition to model the __state__ of our new application:
    ///
    /// ```
    /// # mod counter {
    /// #     pub struct Counter;
    /// # }
    /// use counter::Counter;
    ///
    /// struct ManyCounters {
    ///     counters: Vec<Counter>,
    /// }
    /// ```
    ///
    /// We can store the state of multiple counters now. However, the
    /// __messages__ we implemented before describe the user interactions
    /// of a __single__ counter. Right now, we need to also identify which
    /// counter is receiving user interactions. Can we use composition again?
    /// Yes.
    ///
    /// ```
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// # }
    /// #[derive(Debug, Clone, Copy)]
    /// pub enum Message {
    ///     Counter(usize, counter::Message)
    /// }
    /// ```
    ///
    /// We compose the previous __messages__ with the index of the counter
    /// producing them. Let's implement our __view logic__ now:
    ///
    /// ```
    /// # mod counter {
    /// #     type Text<'a> = iced_native::widget::Text<'a, iced_native::renderer::Null>;
    /// #
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     impl Counter {
    /// #         pub fn view(&mut self) -> Text {
    /// #             Text::new("")
    /// #         }
    /// #     }
    /// # }
    /// #
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use counter::Counter;
    /// #
    /// # struct ManyCounters {
    /// #     counters: Vec<Counter>,
    /// # }
    /// #
    /// # #[derive(Debug, Clone, Copy)]
    /// # pub enum Message {
    /// #    Counter(usize, counter::Message)
    /// # }
    /// use iced_native::Element;
    /// use iced_native::widget::Row;
    /// use iced_wgpu::Renderer;
    ///
    /// impl ManyCounters {
    ///     pub fn view(&mut self) -> Row<Message, Renderer> {
    ///         // We can quickly populate a `Row` by folding over our counters
    ///         self.counters.iter_mut().enumerate().fold(
    ///             Row::new().spacing(20),
    ///             |row, (index, counter)| {
    ///                 // We display the counter
    ///                 let element: Element<counter::Message, Renderer> =
    ///                     counter.view().into();
    ///
    ///                 row.push(
    ///                     // Here we turn our `Element<counter::Message>` into
    ///                     // an `Element<Message>` by combining the `index` and the
    ///                     // message of the `element`.
    ///                     element.map(move |message| Message::Counter(index, message))
    ///                 )
    ///             }
    ///         )
    ///     }
    /// }
    /// ```
    ///
    /// Finally, our __update logic__ is pretty straightforward: simple
    /// delegation.
    ///
    /// ```
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     impl Counter {
    /// #         pub fn update(&mut self, _message: Message) {}
    /// #     }
    /// # }
    /// #
    /// # use counter::Counter;
    /// #
    /// # struct ManyCounters {
    /// #     counters: Vec<Counter>,
    /// # }
    /// #
    /// # #[derive(Debug, Clone, Copy)]
    /// # pub enum Message {
    /// #    Counter(usize, counter::Message)
    /// # }
    /// impl ManyCounters {
    ///     pub fn update(&mut self, message: Message) {
    ///         match message {
    ///             Message::Counter(index, counter_msg) => {
    ///                 if let Some(counter) = self.counters.get_mut(index) {
    ///                     counter.update(counter_msg);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn map<B>(self, f: impl Fn(Message) -> B + 'a) -> CosmicElement<'a, B, Renderer>
    where
        Message: 'a,
        Renderer: iced_native::Renderer + 'a,
        B: 'a,
    {
        CosmicElement::new(CosmicMap::new(self.widget, f))
    }

    #[must_use]
    /// Marks the [`Element`] as _to-be-explained_.
    ///
    /// The [`Renderer`] will explain the layout of the [`Element`] graphically.
    /// This can be very useful for debugging your layout!
    ///
    /// [`Renderer`]: iced_native::Renderer
    pub fn explain<C: Into<Color>>(self, color: C) -> CosmicElement<'a, Message, Renderer>
    where
        Message: 'static,
        Renderer: iced_native::Renderer + 'a,
    {
        CosmicElement {
            widget: Box::new(CosmicExplain::new(self, color.into())),
        }
    }
}

/// Cosmic Explain struct
#[allow(missing_debug_implementations)]
pub struct CosmicExplain<'a, Message, Renderer: iced_native::Renderer> {
    /// explained element
    element: CosmicElement<'a, Message, Renderer>,
    /// explained color
    color: Color,
}

impl<'a, Message, Renderer> CosmicExplain<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn new(element: CosmicElement<'a, Message, Renderer>, color: Color) -> Self {
        CosmicExplain { element, color }
    }
}

impl<'a, Message, Renderer> CosmicWidget<Message, Renderer> for CosmicExplain<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn set_layer(&mut self, layer: Layer) {
        self.element.as_widget_mut().set_layer(layer);
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for CosmicExplain<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.element.widget.width()
    }

    fn height(&self) -> Length {
        self.element.widget.height()
    }

    fn tag(&self) -> tree::Tag {
        self.element.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.element.widget.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.element.widget.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.element.widget.diff(tree);
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.element.widget.layout(renderer, limits)
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        self.element.widget.operate(state, layout, operation);
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.element.widget.on_event(
            state,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        fn explain_layout<Renderer: iced_native::Renderer>(
            renderer: &mut Renderer,
            color: Color,
            layout: Layout<'_>,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_color: color,
                    border_width: 1.0,
                    border_radius: 0.0.into(),
                },
                Color::TRANSPARENT,
            );

            for child in layout.children() {
                explain_layout(renderer, color, child);
            }
        }

        self.element.widget.draw(
            state,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            viewport,
        );

        explain_layout(renderer, self.color, layout);
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element
            .widget
            .mouse_interaction(state, layout, cursor_position, viewport, renderer)
    }

    fn overlay<'b>(
        &'b self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.element.widget.overlay(state, layout, renderer)
    }
}

#[allow(missing_debug_implementations)]
/// Cosmic Map operation
pub struct CosmicMap<'a, A, B, Renderer> {
    /// widget to be mapped
    widget: Box<dyn CosmicWidget<A, Renderer> + 'a>,
    /// mapper
    mapper: Box<dyn Fn(A) -> B + 'a>,
}

impl<'a, A, B, Renderer> CosmicMap<'a, A, B, Renderer> {
    /// create a new map
    pub fn new<F>(
        widget: Box<dyn CosmicWidget<A, Renderer> + 'a>,
        mapper: F,
    ) -> CosmicMap<'a, A, B, Renderer>
    where
        F: 'a + Fn(A) -> B,
    {
        CosmicMap {
            widget,
            mapper: Box::new(mapper),
        }
    }
}

impl<'a, A, Renderer, B> CosmicWidget<B, Renderer> for CosmicMap<'a, A, B, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    A: 'a,
    B: 'a,
{
    fn set_layer(&mut self, layer: Layer) {
        self.widget.set_layer(layer);
    }
}

impl<'a, A, B, Renderer> Widget<B, Renderer> for CosmicMap<'a, A, B, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    A: 'a,
    B: 'a,
{
    fn tag(&self) -> tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.widget.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.widget.diff(tree);
    }

    fn width(&self) -> Length {
        self.widget.width()
    }

    fn height(&self) -> Length {
        self.widget.height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.widget.layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn widget::Operation<B>,
    ) {
        struct MapOperation<'a, B> {
            operation: &'a mut dyn widget::Operation<B>,
        }

        impl<'a, T, B> widget::Operation<T> for MapOperation<'a, B> {
            fn container(
                &mut self,
                id: Option<&widget::Id>,
                operate_on_children: &mut dyn FnMut(&mut dyn widget::Operation<T>),
            ) {
                self.operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }

            fn focusable(
                &mut self,
                state: &mut dyn widget::operation::Focusable,
                id: Option<&widget::Id>,
            ) {
                self.operation.focusable(state, id);
            }

            fn scrollable(
                &mut self,
                state: &mut dyn widget::operation::Scrollable,
                id: Option<&widget::Id>,
            ) {
                self.operation.scrollable(state, id);
            }

            fn text_input(
                &mut self,
                state: &mut dyn widget::operation::TextInput,
                id: Option<&widget::Id>,
            ) {
                self.operation.text_input(state, id);
            }
        }

        self.widget
            .operate(tree, layout, &mut MapOperation { operation });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let status = self.widget.on_event(
            tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            &mut local_shell,
        );

        shell.merge(local_shell, &self.mapper);

        status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.widget.draw(
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.widget
            .mouse_interaction(tree, layout, cursor_position, viewport, renderer)
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, B, Renderer>> {
        let mapper = &self.mapper;

        self.widget
            .overlay(tree, layout, renderer)
            .map(move |overlay| overlay.map(mapper))
    }
}

impl<'a, Message, Renderer> Borrow<dyn CosmicWidget<Message, Renderer> + 'a>
    for CosmicElement<'a, Message, Renderer>
{
    fn borrow(&self) -> &(dyn CosmicWidget<Message, Renderer> + 'a) {
        self.widget.borrow()
    }
}

impl<'a, Message, Renderer> Borrow<dyn CosmicWidget<Message, Renderer> + 'a>
    for &CosmicElement<'a, Message, Renderer>
{
    fn borrow(&self) -> &(dyn CosmicWidget<Message, Renderer> + 'a) {
        self.widget.borrow()
    }
}