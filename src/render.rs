use crate::{MainState, utilities::{vector2::Vector2, math::ffmin}, Intersection, Orientation, map::Type, sprite::Sprite, lighting::Lighting};
const RAYSPERPIXEL: usize = 2;

#[inline(always)]
pub fn calculate_ray(
    ms: &MainState,
    ray_dir_player: Vector2<f32>,
    theta: f32,
) -> (Intersection, Vec<Intersection>) {
    let ray_dir_norm = Vector2::rotate(ray_dir_player, theta);
    let ray_unitstep_size = Vector2::new(
        (1.0 + (ray_dir_norm.y / ray_dir_norm.x) * (ray_dir_norm.y / ray_dir_norm.x)).sqrt(),
        (1.0 + (ray_dir_norm.x / ray_dir_norm.y) * (ray_dir_norm.x / ray_dir_norm.y)).sqrt(),
    );
    let startv = ms.player.pos;

    let mut map_checkv = Vector2::new(startv.x.floor(), startv.y.floor());
    let mut ray_length1_d = Vector2::new(0.0f32, 0.0);
    let mut orientation = Orientation::N;
    let mut wall_type = Type::TiledFloor;
    let mut transparent_walls = vec![];
    let mut stepv = Vector2::new(0.0f32, 0.0);
    let mut last_was_door = false;

    if ray_dir_norm.x < 0.0 {
        stepv.x = -1.0;
        ray_length1_d.x = (startv.x - map_checkv.x) * ray_unitstep_size.x;
    } else {
        stepv.x = 1.0;
        ray_length1_d.x = (map_checkv.x + 1.0 - startv.x) * ray_unitstep_size.x;
    }

    if ray_dir_norm.y < 0.0 {
        stepv.y = -1.0;
        ray_length1_d.y = (startv.y - map_checkv.y) * ray_unitstep_size.y;
    } else {
        stepv.y = 1.0;
        ray_length1_d.y = (map_checkv.y + 1.0 - startv.y) * ray_unitstep_size.y;
    }

    let mut tilefound = false;
    let mut distance = 0.0;

    while !tilefound && distance < 100.0 {
        //arbitrary max distance

        if ray_length1_d.x < ray_length1_d.y {
            map_checkv.x += stepv.x;
            distance = ray_length1_d.x;
            ray_length1_d.x += ray_unitstep_size.x;

            if ray_dir_norm.x < 0.0 {
                orientation = Orientation::W;
            } else {
                orientation = Orientation::E;
            }
        } else {
            map_checkv.y += stepv.y;
            distance = ray_length1_d.y;
            ray_length1_d.y += ray_unitstep_size.y;

            if ray_dir_norm.y < 0.0 {
                orientation = Orientation::S;
            } else {
                orientation = Orientation::N;
            }
        }
        if map_checkv.x >= 0.0
            && map_checkv.x < ms.map_size.0 as f32
            && map_checkv.y >= 0.0
            && map_checkv.y < ms.map_size.1 as f32
        {
            wall_type =
                ms.map.walls[map_checkv.y as usize * ms.map_size.0 + map_checkv.x as usize];

            if last_was_door && wall_type as usize> 0 {
                wall_type = Type::FrameWoodenDoor;
            }
            last_was_door = false;
            if wall_type == Type::WoodenDoor {
                //door
                let door_offset = ms
                    .map
                    .doors
                    .get(&(map_checkv.y as usize * ms.map_size.0 + map_checkv.x as usize))
                    .expect("error finding door")
                    .offset;

                tilefound = true;
                if orientation == Orientation::N || orientation == Orientation::S {
                    if ray_length1_d.y - 0.5 * ray_unitstep_size.y <= ray_length1_d.x {
                        distance = ray_length1_d.y - ray_unitstep_size.y * 0.5;

                        if door_offset < 1.0 {
                            let pos_x = (startv.x + ray_dir_norm.x * distance).fract();
                            if pos_x > door_offset * 0.5 && 1.0 - pos_x > door_offset * 0.5 {
                                last_was_door = true;
                                tilefound = false;
                            }
                        }
                    } else {
                        // side wall
                        if ray_dir_norm.x < 0.0 {
                            orientation = Orientation::W;
                            map_checkv.x -= 1.0;
                        } else {
                            orientation = Orientation::E;
                            map_checkv.x += 1.0;
                        }
                        wall_type = Type::FrameWoodenDoor;
                        distance = ray_length1_d.x;
                    }
                } else {
                    if ray_length1_d.x - 0.5 * ray_unitstep_size.x <= ray_length1_d.y {
                        distance = ray_length1_d.x - ray_unitstep_size.x * 0.5;
                        if door_offset < 1.0 {
                            let pos_y = (startv.y + ray_dir_norm.y * distance).fract();
                            if pos_y > door_offset * 0.5 && 1.0 - pos_y > door_offset * 0.5 {
                                last_was_door = true;
                                tilefound = false;
                            }
                        }
                    } else {
                        if ray_dir_norm.y < 0.0 {
                            orientation = Orientation::S;
                            map_checkv.y -= 1.0;
                        } else {
                            orientation = Orientation::N;
                            map_checkv.y += 1.0;
                        }
                        wall_type = Type::FrameWoodenDoor;
                        distance = ray_length1_d.y;
                    }
                }
            } else if wall_type == Type::Cowbeb || wall_type == Type::MetalBars {
                let mut offset = 0.0;
                if orientation == Orientation::N || orientation == Orientation::S {
                    if ray_length1_d.y - 0.5 * ray_unitstep_size.y <= ray_length1_d.x {
                        distance = ray_length1_d.y - ray_unitstep_size.y * 0.5;
                    } else {
                        if ray_dir_norm.x < 0.0 {
                            orientation = Orientation::W;
                            offset = -1.0;
                        } else {
                            orientation = Orientation::E;
                            offset = 1.0;
                        }
                        distance = ray_length1_d.x;
                    }
                    transparent_walls.push(Intersection::new(
                        (startv + ray_dir_norm * distance).to_array(),
                        distance,
                        (map_checkv.y) as usize * ms.map_size.0
                            + (map_checkv.x + offset) as usize,
                        orientation.clone(),
                        ms.map.walls[(map_checkv.y) as usize * ms.map_size.0
                            + (map_checkv.x + offset) as usize] as usize,
                    ));
                } else {
                    if ray_length1_d.x - 0.5 * ray_unitstep_size.x <= ray_length1_d.y {
                        distance = ray_length1_d.x - ray_unitstep_size.x * 0.5;
                    } else {
                        if ray_dir_norm.y < 0.0 {
                            orientation = Orientation::S;
                            offset = -1.0;
                        } else {
                            orientation = Orientation::N;
                            offset = 1.0;
                        }
                        distance = ray_length1_d.y;
                    }
                    transparent_walls.push(Intersection::new(
                        (startv + ray_dir_norm * distance).to_array(),
                        distance,
                        (map_checkv.y + offset) as usize * ms.map_size.0
                            + (map_checkv.x) as usize,
                        orientation.clone(),
                        ms.map.walls[(map_checkv.y + offset) as usize * ms.map_size.0
                            + (map_checkv.x) as usize] as usize,
                    ));
                }
            } else if wall_type == Type::Stairs  {
                transparent_walls.push(Intersection::new(
                    (startv + ray_dir_norm * distance).to_array(),
                    distance,
                    (map_checkv.y) as usize * ms.map_size.0 + (map_checkv.x) as usize,
                    orientation.clone(),
                    wall_type as usize,
                ));
            } else if wall_type as usize> 0 && wall_type != Type::Stairs2 {
                tilefound = true;
            }
            if ((orientation == Orientation::W || orientation == Orientation::E)
                && ms.map.walls[map_checkv.y as usize * ms.map_size.0
                    + (map_checkv.x - stepv.x) as usize]
                    == Type::WoodenDoor)
                || ((orientation == Orientation::N || orientation == Orientation::S)
                    && ms.map.walls[(map_checkv.y - stepv.y) as usize * ms.map_size.0
                        + map_checkv.x as usize]
                        == Type::WoodenDoor)
            {
                wall_type = Type::FrameWoodenDoor;
            }

            if ms.player.current_wall==Type::Stairs{
                let m = ray_dir_norm.y/ray_dir_norm.x;
                let pos_x0 = ms.player.pos.x.floor();
                let iy = m * (pos_x0-ms.player.pos.x)+ms.player.pos.y; // intersection y with the first step
                        let distance =-ms.player.pos.x.fract()*(1.0
                            + m *m)
                            .sqrt();
                transparent_walls.push(Intersection::new(
                    [pos_x0, iy],
                    distance,
                    (startv.y) as usize * ms.map_size.0 + (startv.x) as usize,
                    Orientation::E,
                    12,
                ));
            }
            if tilefound {
                break;
            }
        }
    }
    let int_point = startv + ray_dir_norm * distance;
    (
        Intersection::new(
            int_point.to_array(),
            distance,
            map_checkv.y as usize * ms.map_size.0 + map_checkv.x as usize,
            orientation,
            wall_type as usize,
        ),
        transparent_walls,
    )
}

#[inline(always)]
pub fn draw_slice(ms: &MainState, slice: &mut [u8], j: usize, h: f32) {
    let (intersection, transparent_walls) =
        calculate_ray(ms,ms.player.dir_norm, ms.angles[j]);

    let corrected_distance = intersection.distance * ms.angles[j].cos();
    let pos_z = ms.player.jump / corrected_distance + ms.player.pitch;

    let rect_h = (ms.player.planedist / corrected_distance * 100.0).round() / 100.0;
    let rect_ceiling = -rect_h+(h - rect_h) * 0.5;
    let rect_floor = (h + rect_h) * 0.5;

    draw_wall(ms, slice, h, &intersection, h * 0.5, rect_h, &pos_z,&ms.lighting_1,);
    draw_wall(ms, slice, h, &intersection,  h * 0.5
        - ms.player.planedist
            / (corrected_distance), rect_h, &(ms.player.jump / corrected_distance + ms.player.pitch),&ms.lighting_2,);

    //draw floor
    for y in (pos_z + rect_floor) as usize..(h) as usize {
        if !(j > 24 / RAYSPERPIXEL && j < 308 / RAYSPERPIXEL && y > 805) {
            // Don't draw the floor behind the minimap image
            let current_dist = ms.buffer_floors[y]; // Use a buffer since they're always the same values
            let weight = current_dist / corrected_distance;

            let rhs = ms.player.pos * (1.0 - weight);
            let current_floor_x = weight * intersection.point[0] + rhs.x;
            let current_floor_y = weight * intersection.point[1] + rhs.y;

            let location =
            unsafe{
                current_floor_x.to_int_unchecked::<usize>() + current_floor_y.to_int_unchecked::<usize>() * ms.map_size.0}; //Cant be negative
            let floor_type = ms.map.floors[location];
            let ftx =  unsafe { (current_floor_x * 128.0).to_int_unchecked::<usize>() % 128 }; //Cant be negative
            let fty = unsafe { (current_floor_y * 128.0).to_int_unchecked::<usize>() % 128 }; //Cant be negative
            let lighting = ms.lighting_1.get_lighting_floor(
                ftx as f32,
                fty as f32,
                location,
            );
            ms.screen.draw_texture(
                slice,
                [ftx, (floor_type * 128) + fty],
                y,
                ms.torch.intensity * lighting,
                ffmin(3.0 / (current_dist * current_dist),1.5),
            )
        }
    }
    //draw ceiling
    let mut rect_top_draw = rect_ceiling;
    if rect_ceiling + pos_z > h {
        rect_top_draw = h - pos_z;
    }
    for y in 0..(rect_top_draw + pos_z) as usize {
        let current_dist = ms.buffer_floors[y];
        let weight = current_dist / corrected_distance;

        let rhs = ms.player.pos * (1.0 - weight);
        let current_floor_x = weight * intersection.point[0] + rhs.x;
        let current_floor_y = weight * intersection.point[1] + rhs.y;

        let location =
            unsafe{
                current_floor_x.to_int_unchecked::<usize>() + current_floor_y.to_int_unchecked::<usize>() * ms.map_size.0};

            let ftx =  unsafe { (current_floor_x * 128.0).to_int_unchecked::<usize>() % 128 };
            let fty = unsafe { (current_floor_y * 128.0).to_int_unchecked::<usize>() % 128 };

         ms.screen.draw_texture(
            slice,
            [ftx, 1152 + fty],
            y,
            ms.torch.intensity
                * ms.lighting_2.get_lighting_floor(
                    ftx as f32,
                    fty as f32,
                    location,
                ),
            ffmin(3.0 / (current_dist * current_dist),1.5),
        );
        //ms.screen.draw_pixel(slice, y as usize, &[0, 0, 0, 0]);
    }
    if !&transparent_walls.is_empty() {
        let mut twandsp = transparent_walls
            .into_iter()
            .map(|e| TWandSprites::TW(e))
            .collect::<Vec<_>>();
        twandsp.extend(ms.sprites.iter().map(|e| TWandSprites::Sprites(e)));

        twandsp.sort_by(|a, b| {
            b.distance2()
                .partial_cmp(&a.distance2())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        twandsp.iter().for_each(|e| {
            match e {
                TWandSprites::Sprites(sprite) => {
                    sprite.draw(slice, &ms.player, j, &ms.screen, corrected_distance)
                }
                TWandSprites::TW(tw) => {
                    let tw_corrected_distance = tw.distance * ms.angles[j].cos();
                    if tw.wall_type == 10 || tw.wall_type == 11 {
                        draw_wall(
                            ms,
                            slice,
                            h,
                            &tw,
                            h * 0.5,
                            ms.player.planedist / tw_corrected_distance,
                            &(ms.player.pitch + ms.player.jump / tw_corrected_distance),
                            &ms.lighting_1,
                        )
                    } else if tw.wall_type == 12
                        && (tw.orientation == Orientation::W
                            || tw.orientation == Orientation::E)
                    {
            
                        let m = (ms.player.pos.y - tw.point[1])
                            / (ms.player.pos.x - tw.point[0]); //slope of the line
                        let iy = m * (tw.point[0] + 1.0 / 4.0 - ms.player.pos.x)
                            + ms.player.pos.y; // intersection y with the first step
                        let delta_distance = (1.0 / (4.0 * 4.0)
                            + (iy - tw.point[1]) * (iy - tw.point[1]))
                            .sqrt(); // distance between steps
                            let start =  if ms.player.current_wall==Type::Stairs{
                                (ms.player.pos.x.fract()*4.0).ceil() as usize
                            }else{
                                    1
                                };
                        for i in (start..8).rev() {
                            let tw2 = Intersection::new(
                                [
                                    tw.point[0] + 1.0 * (i as f32) / 4.0,
                                    m * (tw.point[0] + 1.0 * (i as f32) / 4.0
                                        - ms.player.pos.x)
                                        + ms.player.pos.y,
                                ],
                                tw.distance + delta_distance * (i as f32),
                                tw.map_checkv,
                                Orientation::E,
                                12,
                            );
                            if tw2.point[1] > tw.point[1] -5.0
                                && tw2.point[1] < tw.point[1] +5.0
                            {
                                if ms.map.walls[tw2.point[0] as usize 
                                    + tw2.point[1] as usize * ms.map_size.0]
                                    == Type::Stairs
                                    || ms.map.walls[tw2.point[0] as usize
                                        + tw2.point[1] as usize * ms.map_size.0]
                                        == Type::Stairs2
                                {
                                    draw_wall(
                                        ms,
                                        slice,
                                        h,
                                        &tw2,
                                        h * 0.5
                                            + ms.player.planedist
                                                / (tw2.distance * ms.angles[j].cos())
                                                * ((3.0 - i as f32) / 8.0 + 1.0 / 16.0),
                                        1.0 / 8.0 * ms.player.planedist
                                            / (tw2.distance * ms.angles[j].cos()),
                                        &(ms.player.pitch
                                            + ms.player.jump
                                                / (tw2.distance * ms.angles[j].cos())),
                                                &ms.lighting_1,
                                    );
                                    for y in (ms.player.pitch
                                        + ms.player.jump
                                            / (tw2.distance * ms.angles[j].cos())
                                        + h * 0.5
                                        + ms.player.planedist
                                            / (tw2.distance * ms.angles[j].cos())
                                            * ((3.0 - i as f32) / 8.0 + 1.0 / 8.0))
                                        as usize
                                        ..(ms.player.pitch
                                            + ms.player.jump
                                                / ((tw2.distance - delta_distance)
                                                    * ms.angles[j].cos())
                                            + h * 0.5
                                            + ms.player.planedist
                                                / ((tw2.distance - delta_distance)
                                                    * ms.angles[j].cos())
                                                * ((4.0 - i as f32) / 8.0)).min(h)
                                            as usize
                                    {
                                        let current_dist = (ms.player.planedist
                                            * (1.0 - (i as f32) / 4.0)
                                            + 2.0 * (ms.player.jump))
                                            / (2.0 * (-ms.player.pitch + y as f32) - (h));
                                        let weight = current_dist / corrected_distance;

                                        let rhs = ms.player.pos * (1.0 - weight);
                                        let current_floor_x =
                                            weight * intersection.point[0] + rhs.x;
                                        let current_floor_y =
                                            weight * intersection.point[1] + rhs.y;

                                        let ftx =
                                            (current_floor_x * ms.cell_size) as usize % 128;
                                        let fty =
                                            (current_floor_y * ms.cell_size) as usize % 128;
                                        /*let lighting = ms.lighting.get_lighting_floor(
                                            ftx as f32 / 128.0,
                                            fty as f32 / 128.0,
                                            location,
                                        );*/
                                        ms.screen.draw_texture(
                                            slice,
                                            [ftx, (0 * 128) + fty],
                                            y,
                                            0.05,
                                            (3.0 / (current_dist * current_dist)).min(1.5),
                                        )
                                    }
                                } else if ms.player.planedist * (1.0 - (i as f32) / 4.0)
                                    > -2.0 * (ms.player.jump)
                                {
                                    for y in (pos_z + h * 0.5 + rect_h * (4.0 - i as f32) / 8.0)
                                        as usize
                                        ..(ms.player.pitch
                                            + ms.player.jump
                                                / ((tw2.distance - delta_distance)
                                                    * ms.angles[j].cos())
                                            + h * 0.5
                                            + ms.player.planedist
                                                / ((tw2.distance - delta_distance)
                                                    * ms.angles[j].cos())
                                                * ((4.0 - i as f32) / 8.0))
                                            .min(h)
                                            as usize
                                    {
                                        let current_dist = (ms.player.planedist
                                            * (1.0 - (i as f32) / 4.0)
                                            + 2.0 * (ms.player.jump))
                                            / (2.0 * (-ms.player.pitch + y as f32) - h); // Use a buffer since they're always the same values
                                        let weight = current_dist / corrected_distance;

                                        let rhs = ms.player.pos * (1.0 - weight);
                                        let current_floor_x =
                                            weight * intersection.point[0] + rhs.x;
                                        let current_floor_y =
                                            weight * intersection.point[1] + rhs.y;

                                        let ftx =
                                            (current_floor_x * ms.cell_size) as usize % 128;
                                        let fty =
                                            (current_floor_y * ms.cell_size) as usize % 128;
                                        /*let lighting = ms.lighting.get_lighting_floor(
                                            ftx as f32 / 128.0,
                                            fty as f32 / 128.0,
                                            location,
                                        );*/
                                        ms.screen.draw_texture(
                                            slice,
                                            [ftx, (0 * 128) + fty],
                                            y,
                                            0.05,
                                            ffmin(3.0 / (current_dist * current_dist),1.5),
                                        )
                                    }
                                }
                            }
                        }
                        draw_wall(
                            ms,
                            slice,
                            h,
                            &tw,
                            h * 0.5
                                + ms.player.planedist / tw_corrected_distance * 7.0 / 16.0,
                            1.0 / 8.0 * ms.player.planedist / tw_corrected_distance,
                            &(ms.player.pitch + ms.player.jump / tw_corrected_distance),
                            &ms.lighting_1,
                        );
                    }
                }
            }
        });
    } else {
        ms.sprites.iter().for_each(|sprite| {
            sprite.draw(slice, &ms.player, j, &ms.screen, corrected_distance)
        });
    }
}

#[inline(always)]
fn draw_wall(
    ms: &MainState,
    slice: &mut [u8],
    h: f32,
    intersection: &Intersection,
    center: f32,
    height: f32,
    pos_z: &f32,
    lighting_wall: &Lighting,
) {
    //TODO: REMOVE THIS IF
    let ty_step = {
        if intersection.wall_type == 12 {
            (ms.cell_size) / (8.0 * height)
        } else {
            (ms.cell_size) / (height)
        }
    };

    let inter_x = intersection.point[0].fract();
    let inter_y = intersection.point[1].fract();

    let rect_top = center - height * 0.5;
    let rect_bottom = center + height * 0.5;

    let rect_bottom_draw = {
        if pos_z + rect_bottom >= h {
            h - pos_z
        } else {
            rect_bottom
        }
    };

    //draw walls
    let mut ty = {
        if rect_top + pos_z <= 0.0 {
            -(pos_z + rect_top) * ty_step
        } else if rect_top + pos_z >= h {
            -(pos_z + rect_bottom) * ty_step
        } else {
            0.0
        }
    };

    let mut tx;
    match intersection.orientation {
        Orientation::N => {
            tx = inter_x * ms.cell_size;
            tx = ms.cell_size - 1.0 - tx.floor();
        }
        Orientation::E => {
            tx = inter_y * ms.cell_size;
        }
        Orientation::S => {
            tx = inter_x * ms.cell_size;
        }
        Orientation::W => {
            tx = inter_y * ms.cell_size;
            tx = ms.cell_size - 1.0 - tx.floor();
        }
    }
    if intersection.wall_type == 6 {
        let offset = 1.0
            - ms
                .map
                .doors
                .get(&intersection.map_checkv)
                .expect("error to draw door")
                .offset;
        match intersection.orientation {
            Orientation::N => {
                if inter_x < 0.5 {
                    tx -= offset * 64.0;
                } else {
                    tx += offset * 64.0;
                }
            }
            Orientation::E => {
                if inter_y > 0.5 {
                    tx -= offset * 64.0;
                } else {
                    tx += offset * 64.0;
                }
            }
            Orientation::S => {
                if inter_x > 0.5 {
                    tx -= offset * 64.0;
                } else {
                    tx += offset * 64.0;
                }
            }
            Orientation::W => {
                if inter_y < 0.5 {
                    tx -= offset * 64.0;
                } else {
                    tx += offset * 64.0;
                }
            }
        }
    }

    for y in (pos_z + rect_top) as usize..(pos_z + rect_bottom_draw) as usize {
        //TODO: FIX THIS FLOAT POINT ROUNDING ERROR
        if ty >= 128.0 {
            dbg!(
                height,
                rect_top,
                rect_bottom,
                ty,
                pos_z + rect_top,
                pos_z + rect_bottom_draw,
                ms.player.pitch
            );
            ty = 127.0;
        }
unsafe{
        ms.screen.draw_texture(
            slice,
            [tx.to_int_unchecked::<usize>(), intersection.wall_type * 128 + ty.to_int_unchecked::<usize>()],
            y,
            ms.torch.intensity
                * lighting_wall.get_lighting_wall(
                    tx ,
                    ty * 3.0, //*3.0/128.0
                    intersection.map_checkv,
                    &intersection.orientation,
                ),
            ffmin(3.0 / (intersection.distance * intersection.distance),1.5),
        );
    }
        ty += ty_step;
    }
}

enum TWandSprites<'a> {
    TW(Intersection),
    Sprites(&'a Sprite),
}

impl TWandSprites<'_> {
    fn distance2(&self) -> f32 {
        match self {
            TWandSprites::TW(tw) => tw.distance * tw.distance,
            TWandSprites::Sprites(sprite) => sprite.distance2,
        }
    }
}