use std::arch::x86_64::*;

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
    wall_textures: Vec<f32>,
    sprite_textures: Vec<u8>,
    length_textures: usize,
    length_sprites: usize,
    shade_col: __m128,
    flashlight_col: __m128,

}

impl Screen {
    pub unsafe fn new(widthf: f32, heightf: f32, length_textures: usize, length_sprites: usize) -> Self {
        let width = widthf as usize;
        let height = heightf as usize;
        let img_arr = vec![0; (width * height) * 4];
        let img_arr_len = img_arr.len();
        let shade_v = [1.5, 1.1, 0.6, 1.0];
        let flashlight_v = [1.0, 0.9, 0.8, 1.0];
        Self {
            img_arr,
            img_arr_len,
            width,
            height,
            wall_textures: Vec::new(),
            sprite_textures: Vec::new(),
            length_textures,
            length_sprites,
            shade_col: _mm_loadu_ps(&shade_v[0]),
            flashlight_col:_mm_loadu_ps(&flashlight_v[0]),
        }
    }

    pub fn textures(&mut self, wall_textures: Vec<u8>, sprite_textures: Vec<u8>) {
        self.wall_textures = wall_textures.iter().map(|&p| p as f32).collect();
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
        slice: &mut [u8],
        texture_position: [usize; 2],
        pixel_height: usize,
        shade: f32,
        flashlight: f32,
    ) {
        let pos = (texture_position[1] * self.length_textures + texture_position[0]) << 2; //position of current pixel
        if self.wall_textures[pos + 3] == 255.0 {
            unsafe{
               let p_int = {
            let p = color_pixel(
                &self.wall_textures[pos..pos + 3],
                shade,
                flashlight,
                self.shade_col,
                self.flashlight_col,
            );
            [p[0].to_ne_bytes()[0], p[1].to_ne_bytes()[0], p[2].to_ne_bytes()[0], 255]};
     
            slice[(pixel_height << 2)..(pixel_height << 2) + 4].copy_from_slice(&p_int);
        }
        }
    }
    pub fn draw_sprite(
        &self,
        slice: &mut [u8],
        texture_position: [usize; 2],
        pixel_height: usize,
        shade: f32,
    ) {
        let pos = (texture_position[1] * self.length_sprites + texture_position[0]) << 2; //position of current pixel
                                                                                          // draws in rectangles of 1xwidth_rect size
        let mut pixel: [u8; 4] = self.sprite_textures[pos..pos + 4].try_into().unwrap(); //rgba pixel
                                                                                         //Doesn't draw transparent pixels
        if pixel[3] == 255 {
            if shade != 1.0 && pixel != [255, 0, 0, 255] {
                //Draws shade depening of current lighting, darkening or brightening the pixel
                (0..3).for_each(|j| pixel[j] = (pixel[j] as f32 * shade) as u8);
            }

            slice[(pixel_height << 2)..(pixel_height << 2) + 4].copy_from_slice(&pixel);
        }
    }
/*
    pub fn draw_pixel(&self, img_arr: &mut [u8], pos: usize, pixel: &[u8; 4]) {
        img_arr[(pos << 2)..(pos << 2) + 4].copy_from_slice(pixel);
    }*/
}

#[inline(always)]
   unsafe fn color_pixel(
        pixel: &[f32],
        shade: f32,
        flashlight: f32,
        v_shade_col: __m128,
        v_flashlight_col: __m128,

    ) -> [i32; 8] {
        let v_pixel = _mm_loadu_ps(&pixel[0]);
        let v_shade = _mm_set1_ps(shade);
        let v_flashlight = _mm_set1_ps(flashlight);
        let v_twofivefive  = _mm_set1_epi32(255);
        let mut pixel_out = [0i32; 8];
        let mut multiplicator = _mm_add_ps(_mm_mul_ps(v_shade_col,v_shade), _mm_mul_ps(v_flashlight, v_flashlight_col));
        multiplicator = _mm_mul_ps(multiplicator, v_pixel);
        let out = _mm_cvtps_epi32(multiplicator);

        _mm_storeu_si128(pixel_out.as_mut_ptr() as *mut _, _mm_min_epi32(out, v_twofivefive));
        pixel_out
    }
  