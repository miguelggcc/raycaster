#[allow(dead_code)]
pub struct Slice {
    slice_len: usize,
    width: usize,
    height: usize,
}
impl Slice{
    pub fn new( widthf: f32, heightf: f32) -> Self {
        let width = widthf as usize;
        let height = heightf as usize;
        let slice_len =(widthf*heightf*4.0) as usize;
        Self {
            slice_len,
            width,
            height,
        }
    }

    pub fn draw_texture(
        self,
        slice_v: &mut Vec<u8>,
        texture: &[u8],
        texture_position: [usize; 2],
        pixel_position: [usize; 2],
        width_rect: usize,
        shade: f32,
        length: usize,
    ) {
        let pos = (texture_position[1] * length + texture_position[0]) << 2; //position of current pixel
        for i in 0..width_rect {
            // draws in rectangles of 1xwidth_rect size
            if pos + 4 > texture.len() {
                dbg!(texture_position, pixel_position, length);
            }
            let mut pixel: [u8; 4] = texture[pos..pos + 4].try_into().unwrap(); //rgba pixel

            if pixel[3] == 255 {
                if shade != 1.0 {
                    //Draws shade depening of current lighting, darkening or brightening the pixel

                    pixel[0] = (pixel[0] as f32 * shade) as u8;
                    pixel[1] = (pixel[1] as f32 * shade) as u8;
                    pixel[2] = (pixel[2] as f32 * shade) as u8;
                }

                //Doesn't draw transparent pixels
                self.draw_pixel(slice_v,pixel_position[0] + i, pixel_position[1], &pixel);
            }
        } 
    }
    
    pub fn draw_pixel(self, slice_v: &mut Vec<u8>, position_x: usize, position_y: usize, pixel: &[u8; 4]) {
        let i = position_y * self.width + position_x;
        if (i << 2) + 4 < self.slice_len {
            slice_v[(i << 2)..(i << 2) + 4].copy_from_slice(pixel);
        }
    }
}
