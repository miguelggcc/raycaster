use ggez::{
    graphics::{GlBackendSpec, Image, ImageGeneric},
    Context, GameResult,
};

#[allow(dead_code)]
pub struct Screen {
    pub img_arr: Vec<u8>,
    img_arr_len: usize,
    width: usize,
    height: usize,
    wall_textures: Vec<u8>,
    sprite_textures: Vec<u8>,
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
            wall_textures: Vec::new(),
            sprite_textures: Vec::new(),
        }
    }

    pub fn textures(&mut self, wall_textures: Vec<u8>, sprite_textures: Vec<u8>) {
        self.wall_textures = wall_textures;
        self.sprite_textures = sprite_textures;
    }
    #[allow(dead_code)]
    pub fn reset_img(&mut self) {
        self.img_arr = vec![0; (self.width * self.height) * 4];
    }

    pub fn arr_to_rgba(&mut self, ctx: &mut Context) -> GameResult<ImageGeneric<GlBackendSpec>> {
        Image::from_rgba8(ctx, self.width as u16, self.height as u16, &self.img_arr)
    }

    pub fn draw_texture(
        &self,
        img_arr: &mut [u8],
        texture_position: [usize; 2],
        pixel_position: [usize; 2],
        width_rect: usize,
        shade: f32,
        length: usize,
    ) {
        let pos = (texture_position[1] * length + texture_position[0]) << 2; //position of current pixel
        (0..width_rect).for_each(|i| {
            // draws in rectangles of 1xwidth_rect size
            let mut pixel: [u8; 4] = self.wall_textures[pos..pos + 4].try_into().unwrap(); //rgba pixel

            if pixel[3] == 255 {
                if shade != 1.0 {
                    (0..3).for_each(|j| pixel[j] = (pixel[j] as f32 * shade) as u8);
                }

                //Doesn't draw transparent pixels
                self.draw_pixel(img_arr, pixel_position[0] + i, pixel_position[1], &pixel);
            }
        });
    }

    pub fn draw_sprite(
        &mut self,
        texture_position: [usize; 2],
        pixel_position: [usize; 2],
        width_rect: usize,
        shade: f32,
        length: usize,
    ) {
        let pos = (texture_position[1] * length + texture_position[0]) << 2; //position of current pixel
        (0..width_rect).for_each(|i| {
            // draws in rectangles of 1xwidth_rect size
            let mut pixel: [u8; 4] = self.sprite_textures[pos..pos + 4].try_into().unwrap(); //rgba pixel

            if pixel[3] == 255 {
                if shade != 1.0 && pixel != [255, 0, 0, 255] {
                    //Draws shade depening of current lighting, darkening or brightening the pixel
                    (0..3).for_each(|j| pixel[j] = (pixel[j] as f32 * shade) as u8);
                }

                //Doesn't draw transparent pixels
                self.draw_pixel_sprite(pixel_position[0] + i, pixel_position[1], &pixel);
            }
        });
    }

    pub fn draw_pixel(
        &self,
        img_arr: &mut [u8],
        position_x: usize,
        position_y: usize,
        pixel: &[u8; 4],
    ) {
        let i = position_y + position_x;

        img_arr[(i << 2)..(i << 2) + 4].copy_from_slice(pixel);
    }

    pub fn draw_pixel_sprite(&mut self, position_x: usize, position_y: usize, pixel: &[u8; 4]) {
        let i = position_y * self.width + position_x;
        self.img_arr[(i << 2)..(i << 2) + 4].copy_from_slice(pixel);
    }
}
