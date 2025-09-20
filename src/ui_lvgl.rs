use std::sync::mpsc;
use std::time::Instant;

use esp_idf_svc::sys::EspError;

use core::cell::UnsafeCell;

use lvgl::style::Style;
use lvgl::widgets::Label;
use lvgl::{Align, Color, Display, DrawBuffer, Part, Widget};

use cstr_core::CString;

use crate::global;

fn init_spi() -> Result<(), EspError> {
    use esp_idf_svc::sys::*;
    const GPIO_NUM_NC: i32 = -1;
    const DISPLAY_MOSI_PIN: i32 = 47;
    const DISPLAY_CLK_PIN: i32 = 21;
    ::log::info!("Initialize SPI bus");
    let mut buscfg = spi_bus_config_t::default();
    buscfg.__bindgen_anon_1.mosi_io_num = DISPLAY_MOSI_PIN;
    buscfg.__bindgen_anon_2.miso_io_num = GPIO_NUM_NC;
    buscfg.sclk_io_num = DISPLAY_CLK_PIN;
    buscfg.__bindgen_anon_3.quadwp_io_num = GPIO_NUM_NC;
    buscfg.__bindgen_anon_4.quadhd_io_num = GPIO_NUM_NC;
    buscfg.max_transfer_sz =
        (global::DISPLAY_WIDTH * global::DISPLAY_HEIGHT * std::mem::size_of::<u16>()) as i32;
    esp!(unsafe {
        spi_bus_initialize(
            spi_host_device_t_SPI3_HOST,
            &buscfg,
            spi_common_dma_t_SPI_DMA_CH_AUTO,
        )
    })
}

static mut ESP_LCD_PANEL_HANDLE: esp_idf_svc::sys::esp_lcd_panel_handle_t = std::ptr::null_mut();

fn init_lcd() -> Result<(), EspError> {
    use esp_idf_svc::sys::*;
    const DISPLAY_CS_PIN: i32 = 41;
    const DISPLAY_DC_PIN: i32 = 40;
    ::log::info!("Install panel IO");
    let mut panel_io: esp_lcd_panel_io_handle_t = std::ptr::null_mut();
    let mut io_config = esp_lcd_panel_io_spi_config_t::default();
    io_config.cs_gpio_num = DISPLAY_CS_PIN;
    io_config.dc_gpio_num = DISPLAY_DC_PIN;
    io_config.spi_mode = 3;
    io_config.pclk_hz = 40 * 1000 * 1000;
    io_config.trans_queue_depth = 10;
    io_config.lcd_cmd_bits = 8;
    io_config.lcd_param_bits = 8;
    esp!(unsafe {
        esp_lcd_new_panel_io_spi(spi_host_device_t_SPI3_HOST as _, &io_config, &mut panel_io)
    })?;

    ::log::info!("Install LCD driver");
    const DISPLAY_RST_PIN: i32 = 45;
    let mut panel_config = esp_lcd_panel_dev_config_t::default();
    let mut panel: esp_lcd_panel_handle_t = std::ptr::null_mut();

    panel_config.reset_gpio_num = DISPLAY_RST_PIN;
    panel_config.data_endian = lcd_rgb_data_endian_t_LCD_RGB_DATA_ENDIAN_LITTLE;
    panel_config.__bindgen_anon_1.rgb_ele_order = lcd_rgb_element_order_t_LCD_RGB_ELEMENT_ORDER_RGB;
    panel_config.bits_per_pixel = 16;

    esp!(unsafe { esp_lcd_new_panel_st7789(panel_io, &panel_config, &mut panel) })?;
    unsafe { ESP_LCD_PANEL_HANDLE = panel };

    const DISPLAY_MIRROR_X: bool = false;
    const DISPLAY_MIRROR_Y: bool = false;
    const DISPLAY_SWAP_XY: bool = false;
    const DISPLAY_INVERT_COLOR: bool = true;

    ::log::info!("Reset LCD panel");
    unsafe {
        esp!(esp_lcd_panel_reset(panel))?;
        esp!(esp_lcd_panel_init(panel))?;
        esp!(esp_lcd_panel_invert_color(panel, DISPLAY_INVERT_COLOR))?;
        esp!(esp_lcd_panel_swap_xy(panel, DISPLAY_SWAP_XY))?;
        esp!(esp_lcd_panel_mirror(
            panel,
            DISPLAY_MIRROR_X,
            DISPLAY_MIRROR_Y
        ))?;
        esp!(esp_lcd_panel_disp_on_off(panel, true))?; /* 启动屏幕 */
    }
    ::log::info!("LCD panel initialized successfully");
    Ok(())
}

pub struct UI {}

impl UI {
    pub fn new() -> Self {
        lvgl::init();

        init_spi().unwrap();
        init_lcd().unwrap();

        Self {}
    }

    pub fn run(&mut self, rx: mpsc::Receiver<String>) {
        log::info!("=============  Registering Display ====================");
        const HOR_RES: u32 = global::DISPLAY_WIDTH as u32;
        const VER_RES: u32 = global::DISPLAY_HEIGHT as u32;
        const LINES: u32 = 4; // The number of lines (rows) that will be refreshed  was 12
        let draw_buffer = DrawBuffer::<{ (HOR_RES * LINES) as usize }>::default();
        let display = Display::register(draw_buffer, HOR_RES, VER_RES, |refresh| {
            Self::set_pixels_lvgl_color(
                refresh.area.x1.into(),
                refresh.area.y1.into(),
                (refresh.area.x2 + 1i16).into(),
                (refresh.area.y2 + 1i16).into(),
                refresh.colors.into_iter(),
            )
            .unwrap();
        })
        .unwrap();

        log::info!("=============  Creating UI ====================");
        // Create screen and widgets
        let mut screen = display.get_scr_act().unwrap();
        let mut screen_style = Style::default();
        screen_style.set_bg_color(Color::from_rgb((0, 0, 139)));
        screen_style.set_radius(0);
        screen.add_style(Part::Main, &mut screen_style);

        // Create button label, align in center of button
        let mut lbl = Label::create(&mut screen).unwrap();
        lbl.set_align(Align::Center, 0, 0);
        lbl.set_text(CString::new("Rust lvgl demo").unwrap().as_c_str());

        loop {
            let data = rx.recv().unwrap();
            log::info!("lvgl recv");
            let start = Instant::now();
            lvgl::task_handler();
            lbl.set_text(CString::new(data).unwrap().as_c_str());
            lvgl::tick_inc(Instant::now().duration_since(start));
        }
    }

    ///
    /// Sets pixel colors in a rectangular region.
    ///
    /// The color values from the `colors` iterator will be drawn to the given region starting
    /// at the top left corner and continuing, row first, to the bottom right corner. No bounds
    /// checking is performed on the `colors` iterator and drawing will wrap around if the
    /// iterator returns more color values than the number of pixels in the given region.
    ///
    /// # Arguments
    ///
    /// * `sx` - x coordinate start
    /// * `sy` - y coordinate start
    /// * `ex` - x coordinate end
    /// * `ey` - y coordinate end
    /// * `colors` - anything that can provide `IntoIterator<Item = lvgl::Color>` to iterate over pixel data
    fn set_pixels_lvgl_color<T>(
        sx: i32,
        sy: i32,
        ex: i32,
        ey: i32,
        colors: T,
    ) -> Result<(), EspError>
    where
        T: IntoIterator<Item = lvgl::Color>,
    {
        let iter = UnsafeCell::new(colors);
        unsafe {
            let e = esp_idf_svc::sys::esp_lcd_panel_draw_bitmap(
                ESP_LCD_PANEL_HANDLE,
                sx,
                sy,
                ex,
                ey,
                &iter as *const _ as _, //colors.as_ptr() as *const c_void,
            );
            if e != 0 {
                log::warn!("flush_display error: {}", e);
            }
            log::info!("flush_display drawn to display");
        };

        Ok(())
    }
}
