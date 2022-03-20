use ggez::{
    graphics::{Color, GlBackendSpec, Image, ImageGeneric},
    Context, GameResult,
};

#[allow(dead_code)]
pub struct Screen {
    pub img_arr: Vec<u8>,
    img_arr_len: usize,
    width: usize,
    height: usize,
}

impl Screen {
    pub fn new(widthf: f32, heightf: f32) -> Self {
        let width = widthf as usize;
        let height = heightf as usize;
        let img_arr = vec![0; (width * height) * 4];
        let img_arr_len = img_arr.len();
        Self {
            img_arr,
            img_arr_len,
            width,
            height,
        }
    }
    #[allow(dead_code)]
    pub fn reset_img(&mut self) {
        self.img_arr = vec![0; (self.width * self.height) * 4];
    }

    pub fn arr_to_rgba(&mut self, ctx: &mut Context) -> GameResult<ImageGeneric<GlBackendSpec>> {
        Image::from_rgba8(ctx, self.width as u16, self.height as u16, &self.img_arr)
    }

    pub fn draw_texture(
        &mut self,
        texture: &[u8],
        texture_position: [usize; 2],
        pixel_position: [usize; 2],
        width_rect: usize,
        shade: f32,
        length: usize,
    ) {
        let pos = (texture_position[1] * length + texture_position[0]) << 2; //position of current pixel
        for i in 0..width_rect {
            //(0..width_rect).for_each(|i| {
            // draws in rectangles of 1xwidth_rect size
            if pos + 4 > texture.len() {
                dbg!(texture_position, pixel_position, length);
            }
            let mut pixel: [u8; 4] = texture[pos..pos + 4].try_into().unwrap(); //rgba pixel

            if pixel[3] == 255 {
                if shade != 1.0 && pixel != [255, 0, 0, 255] {
                    //Draws shade depening of current lighting, darkening or brightening the pixel

                    pixel[0] = (pixel[0] as f32 * shade) as u8;
                    pixel[1] = (pixel[1] as f32 * shade) as u8;
                    pixel[2] = (pixel[2] as f32 * shade) as u8;
                }

                //Doesn't draw transparent pixels
                self.draw_pixel(pixel_position[0] + i, pixel_position[1], &pixel);
            }
        } // });
    }

    pub fn draw_pixel(&mut self, position_x: usize, position_y: usize, pixel: &[u8; 4]) {
        let i = position_y * self.width + position_x;

        self.img_arr[(i << 2)..(i << 2) + 4].copy_from_slice(pixel);
    }
    #[allow(dead_code)]
    pub fn draw_rect(
        &mut self,
        position: [usize; 2],
        width_rect: usize,
        height_rect: usize,
        color: Color,
    ) {
        for w_r in 0..width_rect {
            for h_r in 0..height_rect {
                let c = color.to_rgba();
                self.draw_pixel(position[0] + w_r, h_r + position[1], &[c.0, c.1, c.2, c.3]);
            }
        }
    }
}
