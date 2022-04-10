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
    length_textures: usize,
    length_sprites: usize,
}

impl Screen {
    pub fn new(widthf: f32, heightf: f32, length_textures: usize, length_sprites: usize) -> Self {
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
            length_textures,
            length_sprites,
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
        pixel_height: usize,
        width_rect: usize,
        shade: f32,
    ) {
        let pos = (texture_position[1] * self.length_textures + texture_position[0]) << 2; //position of current pixel
        (0..width_rect).for_each(|i| {
            // draws in rectangles of 1xwidth_rect size
            if pos + 4 > self.wall_textures.len() {
                dbg!(texture_position, pixel_height);
            }
            let mut pixel: [u8; 4] = self.wall_textures[pos..pos + 4].try_into().unwrap(); //rgba pixel

            if pixel[3] == 255 {
                //if shade != 1.0 {
                pixel[0] = (pixel[0] as f32 * shade) as u8;
                pixel[1] = (pixel[1] as f32 * shade * 0.9) as u8;
                pixel[2] = (pixel[2] as f32 * shade * 0.75) as u8;
                                                  //}

                //Doesn't draw transparent pixels
                self.draw_pixel(img_arr, i * self.width + pixel_height, &pixel);
            }
        });
    }

    pub fn draw_sprite(
        &self,
        slice: &mut [u8],
        texture_position: [usize; 2],
        pixel_height: usize,
        width_rect: usize,
        shade: f32,
    ) {
        let pos = (texture_position[1] * self.length_sprites + texture_position[0]) << 2; //position of current pixel
        (0..width_rect).for_each(|i| {
            // draws in rectangles of 1xwidth_rect size
            let mut pixel: [u8; 4] = self.sprite_textures[pos..pos + 4].try_into().unwrap(); //rgba pixel

            if pixel[3] == 255 {
                if shade != 1.0 && pixel != [255, 0, 0, 255] {
                    //Draws shade depening of current lighting, darkening or brightening the pixel
                    (0..3).for_each(|j| pixel[j] = (pixel[j] as f32 * shade) as u8);
                }

                //Doesn't draw transparent pixels
                self.draw_pixel(slice, i * self.width + pixel_height, &pixel);
            }
        });
    }

    pub fn draw_pixel(&self, img_arr: &mut [u8], pos: usize, pixel: &[u8; 4]) {
        img_arr[(pos << 2)..(pos << 2) + 4].copy_from_slice(pixel);
    }
}
