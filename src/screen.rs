use ggez::{
    graphics::{GlBackendSpec, Image, ImageGeneric},
    Context, GameResult,
};

use simdeez::sse2::*;
use simdeez::sse41::*;

#[allow(dead_code)]
pub struct Screen {
    pub img_arr: Vec<u8>,
    img_arr_len: usize,
    width: usize,
    height: usize,
    wall_textures: Vec<f32>,
    sprite_textures: Vec<u8>,
    length_textures: usize,
    length_sprites: usize,
    shade_col: [f32; 4],
    flashlight_col: [f32; 4],
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
            shade_col: [1.5, 1.1, 0.6, 1.0],
            flashlight_col: [1.0, 0.9, 0.8, 1.0],
        }
    }

    pub fn textures(&mut self, wall_textures: Vec<u8>, sprite_textures: Vec<u8>) {
        self.wall_textures =wall_textures.iter().map(|&p| p as f32).collect();
        self.sprite_textures = sprite_textures;
    }
    #[allow(dead_code)]
    pub fn reset_img(&mut self) {
        self.img_arr = vec![0; (self.width * self.height) * 4];
    }

    pub fn arr_to_rgba(&self, ctx: &mut Context) -> GameResult<ImageGeneric<GlBackendSpec>> {
        Image::from_rgba8(ctx, self.width as u16, self.height as u16, &self.img_arr)
    }
    pub fn draw_texture(
        &self,
        img_arr: &mut [u8],
        texture_position: [usize; 2],
        pixel_height: usize,
        width_rect: usize,
        shade: f32,
        flashlight: f32,
    ) {
        let pos = (texture_position[1] * self.length_textures + texture_position[0]) << 2; //position of current pixel
            // draws in rectangles of 1xwidth_rect size
                let p = color_pixel_compiletime(
                    &self.wall_textures[pos..pos+4],
                    &[shade, shade, shade, 1.0],
                    &[flashlight, flashlight, flashlight, 1.0],
                    &self.shade_col,
                    &self.flashlight_col,
                );
                /*pixel[0] = (pixel[0] as f32 * (shade * 1.5 + flashlight * 1.0)) as u8;
                pixel[1] = (pixel[1] as f32 * (shade * 1.1 + flashlight * 0.9)) as u8;
                pixel[2] = (pixel[2] as f32 * (shade * 0.6 + flashlight * 0.8)) as u8;*/
                let p_int = [p[0] as u8, p[1] as u8, p[2] as u8,  p[3] as u8];
                self.draw_pixel(img_arr,  pixel_height, &p_int);
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
simd_compiletime_generate!(
    fn color_pixel(
        pixel: &[f32],
        shade: &[f32],
        flashlight: &[f32],
        shade_col: &[f32],
        flashlight_col: &[f32],
    ) -> [i32; 8] {
        //let  v_pixel = S::loadu_epi32(&pixel[0]);
        let v_pixel_f = S::loadu_ps(&pixel[0]);
        let v_shade = S::loadu_ps(&shade[0]);
        let v_flashlight = S::loadu_ps(&flashlight[0]);
        let v_shade_col = S::loadu_ps(&shade_col[0]);
        let v_flashlight_col = S::loadu_ps(&flashlight_col[0]);
        let v_twofivefive = S::set1_epi32(255);
        let mut pixel_out = [0i32; 8];
        let mut multiplicator = v_shade_col*v_shade + v_flashlight*v_flashlight_col;
        multiplicator *= v_pixel_f;
        let out = S::cvtps_epi32(multiplicator);

        S::storeu_epi32(&mut pixel_out[0], S::min_epi32(out, v_twofivefive));
        pixel_out
    }
);
