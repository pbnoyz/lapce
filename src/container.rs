use crate::{
    command::{CraneCommand, CRANE_COMMAND},
    editor::Editor,
    editor::EditorState,
    palette::PaletteWrapper,
};
use crate::{palette::Palette, split::CraneSplit};
use crate::{scroll::CraneScroll, state::CRANE_STATE};
use druid::{
    kurbo::{Line, Rect},
    widget::IdentityWrapper,
    Command, MouseEvent, Selector, Target, WidgetId,
};
use druid::{
    theme, BoxConstraints, Cursor, Data, Env, Event, EventCtx, LayoutCtx,
    LifeCycle, LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx,
    Widget, WidgetExt, WidgetPod,
};

pub struct ChildState {
    pub origin: Option<Point>,
    pub size: Option<Size>,
    pub hidden: bool,
}

pub struct CraneContainer<T> {
    palette_max_size: Size,
    palette_rect: Rect,
    palette: WidgetPod<T, Box<dyn Widget<T>>>,
    editor_split: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> CraneContainer<T> {
    pub fn new() -> Self {
        let palette = PaletteWrapper::new();
        let palette_id = WidgetId::next();
        let palette =
            WidgetPod::new(IdentityWrapper::wrap(palette, palette_id)).boxed();
        CRANE_STATE
            .palette
            .lock()
            .unwrap()
            .set_widget_id(palette_id);

        let editor_id = WidgetId::next();
        let editor = Editor::new(editor_id);
        let scroll_id = WidgetId::next();
        let editor_state = EditorState::new(editor_id, scroll_id);
        CRANE_STATE
            .editor_split
            .lock()
            .unwrap()
            .editors
            .insert(editor_id, editor_state);
        CRANE_STATE
            .editor_split
            .lock()
            .unwrap()
            .set_active(editor_id);
        let editor_split = WidgetPod::new(CraneSplit::new(true).with_child(
            IdentityWrapper::wrap(
                CraneScroll::new(IdentityWrapper::wrap(editor, editor_id)),
                scroll_id,
            ),
        ))
        .boxed();

        CraneContainer {
            palette_max_size: Size::new(600.0, 400.0),
            palette_rect: Rect::ZERO
                .with_origin(Point::new(200.0, 100.0))
                .with_size(Size::new(600.0, 400.0)),
            palette,
            editor_split,
        }
    }
}

impl<T: Data> Widget<T> for CraneContainer<T> {
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut T,
        env: &Env,
    ) {
        ctx.request_focus();
        match event {
            Event::Internal(_) => {
                self.palette.event(ctx, event, data, env);
                self.editor_split.event(ctx, event, data, env);
            }
            Event::KeyDown(key_event) => CRANE_STATE.key_down(key_event),
            Event::Command(cmd) => {
                match cmd {
                    _ if cmd.is(CRANE_COMMAND) => {
                        let cmd = cmd.get_unchecked(CRANE_COMMAND);
                        match cmd {
                            CraneCommand::Palette => (),
                            _ => (),
                        };
                        self.palette.event(ctx, event, data, env)
                    }
                    _ => (),
                }
                return;
            }
            _ => (),
        }

        match event {
            Event::MouseDown(mouse)
            | Event::MouseUp(mouse)
            | Event::MouseMove(mouse)
            | Event::Wheel(mouse) => {
                if !CRANE_STATE.palette.lock().unwrap().hidden
                    && self.palette_rect.contains(mouse.pos)
                {
                    self.palette.event(ctx, event, data, env);
                    return;
                } else {
                    self.editor_split.event(ctx, event, data, env);
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        self.palette.lifecycle(ctx, event, data, env);
        self.editor_split.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &T,
        data: &T,
        env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let size = bc.max();

        let palette_bc = BoxConstraints::new(Size::ZERO, self.palette_max_size);
        let palette_size = self.palette.layout(ctx, &palette_bc, data, env);
        self.palette_rect = Rect::ZERO
            .with_origin(Point::new(
                (size.width - self.palette_max_size.width) / 2.0,
                ((size.height - self.palette_max_size.height) / 4.0).max(0.0),
            ))
            .with_size(palette_size);
        println!("palette_size {:?}", palette_size);
        self.palette
            .set_layout_rect(ctx, data, env, self.palette_rect);

        self.editor_split.layout(ctx, bc, data, env);
        self.editor_split.set_layout_rect(
            ctx,
            data,
            env,
            Rect::ZERO.with_size(size),
        );
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.editor_split.paint(ctx, data, env);
        self.palette.paint(ctx, data, env);
    }
}
