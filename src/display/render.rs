use crate::display::{screen::Screen, theme::Theme};

pub trait RenderDisplay {
    async fn render(&mut self, screen: &mut Screen, theme: &Theme) -> Result<(), u16>;
}

pub async fn render<D>(display: &mut D, screen: &mut Screen) -> Result<(), u16>
where
    D: RenderDisplay,
{
    let theme = Theme::base();
    display.render(screen, &theme).await
}
