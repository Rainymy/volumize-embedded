use crate::display::screen::Screen;

pub trait RenderDisplay {
    async fn render(&mut self, screen: Screen) -> Result<(), u16>;
}

pub async fn render<D>(display: &mut D, screen: Screen) -> Result<(), u16>
where
    D: RenderDisplay,
{
    display.render(screen).await
}
