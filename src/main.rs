extern crate image;

use std::env::args as args;
use image::io::Reader as ImageReader;
#[allow(unused_imports)]
use image::{DynamicImage, GenericImageView, Pixel};
use std::collections::{BTreeMap, BTreeSet};


#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
struct RgbaF {
    r : f32,
    g : f32,
    b : f32,
    a : f32
}
impl RgbaF {
    #[inline(always)]
    fn new(r : f32, g : f32, b : f32, a : f32) -> RgbaF
    {
        RgbaF{r, g, b, a}
    }
    #[inline(always)]
    fn to_u8(&self) -> Rgba
    {
        Rgba{r : (self.r * 255.0).round() as u8, g : (self.g * 255.0).round() as u8, b : (self.b * 255.0).round() as u8, a : (self.a * 255.0).round() as u8}
    }
    #[inline(always)]
    fn add(&self, other : &RgbaF) -> RgbaF
    {
        RgbaF{r : self.r + other.r, g : self.g + other.g, b : self.b + other.b, a : self.a + other.a}
    }
    fn add_mut(&mut self, other : &RgbaF)
    {
        self.r += other.r;
        self.g += other.g;
        self.b += other.b;
        self.a += other.a;
    }
    #[inline(always)]
    fn mult(&self, thing : f32) -> RgbaF
    {
        RgbaF{r : self.r * thing, g : self.g * thing, b : self.b * thing, a : self.a * thing}
    }
}


#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Hash)]
#[derive(PartialOrd)]
#[derive(Ord)]
struct Rgba {
    r : u8,
    g : u8,
    b : u8,
    a : u8
}

impl Rgba {
    #[inline(always)]
    fn new(r : u8, g : u8, b : u8, a : u8) -> Rgba
    {
        Rgba{r, g, b, a}
    }
    #[inline(always)]
    fn to_float(&self) -> RgbaF
    {
        RgbaF{r : self.r as f32/255.0, g : self.g as f32/255.0, b : self.b as f32/255.0, a : self.a as f32/255.0}
    }
}

impl Eq for Rgba {}

/*
fn eq(a : &Vec<Vec<Rgba>>, b : &Vec<Vec<Rgba>>) -> bool
{
    if a.len() != b.len()
    {
        return false;
    }
    for (i, x) in a.iter().enumerate()
    {
        if &b[i] != x
        {
            return false;
        }
    }
    true
}
*/

const tilesize : usize = 16;

fn get_tile_from_image(a : &Vec<Vec<Rgba>>, (x, y) : (usize, usize)) -> Vec<Rgba>
{
    let mut ret = vec!(Rgba::new(255, 255, 255, 255); tilesize*tilesize);
    
    for iy in 0..tilesize
    {
        for ix in 0..tilesize
        {
            ret[iy*tilesize + ix] = a[y*tilesize+iy][x*tilesize+ix];
        }
    }
    
    ret
}

type TileId = usize;

fn default_neighbor(max_index : TileId) -> Vec<f64>
{
    let mut ret = Vec::with_capacity(max_index);
    for _ in 0..max_index
    {
        ret.push(0.0);
    }
    ret
}

fn main() {
    let args = args().collect::<Vec<String>>();
    assert!(args.len() >= 3);
    let img = ImageReader::open(&args[1]).unwrap().decode().unwrap();
    let mut px_map = Vec::with_capacity(img.height() as usize);
    for y in 0..img.height()
    {
        let mut row = Vec::with_capacity(img.width() as usize);
        for x in 0..img.width()
        {
            let px : image::Rgba<u8> = img.get_pixel(x, y);
            row.push(Rgba::new(px[0], px[1], px[2], px[3]));
        }
        px_map.push(row);
    }
    
    let mut max_index = 0;
    let mut tile_to_id = BTreeMap::new();
    let mut id_to_tile = Vec::new();
    let mut map = Vec::with_capacity(img.height() as usize/tilesize);
    for y in 0..img.height() as usize/tilesize
    {
        let mut row = Vec::with_capacity(img.width() as usize/tilesize);
        for x in 0..img.width() as usize/tilesize
        {
            let tile = get_tile_from_image(&px_map, (x, y));
            let id = tile_to_id.entry(tile.clone()).or_insert_with(|| {max_index += 1; max_index - 1});
            if *id >= id_to_tile.len()
            {
                id_to_tile.push(tile);
            }
            
            row.push(*id);
            //print!("{} ", *id);
        }
        //println!();
        map.push(row);
    }
    println!("number of unique tiles: {}", max_index);
    
    let mut forbidden_tiles = BTreeSet::new();
    forbidden_tiles.insert(map[1][0]);
    forbidden_tiles.insert(*map[1].last().unwrap());
    forbidden_tiles.insert(map[0][1]);
    forbidden_tiles.insert(map.last().unwrap()[1]);
    forbidden_tiles.insert(map[0][0]);
    forbidden_tiles.insert(*map[0].last().unwrap());
    forbidden_tiles.insert(map.last().unwrap()[0]);
    forbidden_tiles.insert(*map.last().unwrap().last().unwrap());
    
    const directions : [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    //const directions : [(isize, isize); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
    let get_opposite_direction = |dir_index : usize| -> usize
    {
        (dir_index+directions.len()/2)%directions.len()
    };
    
    let mut freqs = vec!(0.0; max_index);
    let mut ships = Vec::with_capacity(max_index);
    for _ in 0..max_index
    {
        ships.push(vec![default_neighbor(max_index); directions.len()]);
    }
    
    println!("freqs len: {}", freqs.len());
    println!("ships len: {}", ships.len());
    let in_bounds = |(x, y)| x < map[0].len() && y < map.len();
    #[allow(unused_assignments)]
    let mut most_common = 0;
    let mut most_common_freq = 0.0;
    for y in 0..map.len()
    {
        for x in 0..map[0].len()
        {
            freqs[map[y][x]] += 1.0;
            let freq = freqs[map[y][x]];
            for (i, offset) in directions.iter().enumerate()
            {
                let offset = ((offset.0 + x as isize) as usize, (offset.1 + y as isize) as usize);
                let value;
                if in_bounds(offset)
                {
                    value = map[offset.1][offset.0];
                    let id = map[y][x];
                    if freq > most_common_freq
                    {
                        most_common_freq = freq;
                        most_common = id;
                    }
                    ships[id][i][value] += 1.0;
                    //println!("added to direction {} for tile type {}", i, id);
                    //ships[id][i][value] = 1.0;
                }
                else
                {
                    //println!("direction {} for tile centered at {},{} is out of bounds", i, x, y);
                }
            }
        }
    }
    
    let total_freq : f64 = freqs.iter().sum();
    
    for i in 0..ships.len()
    {
        if forbidden_tiles.contains(&i)
        {
            continue;
        }
        for j in 0..directions.len()
        {
            //for other in &ships[i][j]
            //{
            //    //total += other;
            //}
            let mut total = 0.0;
            for other in &ships[i][j]
            {
                total += other;
            }
            //assert!(total > 0.0, "failed tile type {} direction {}", i, j);
            for other in &mut ships[i][j]
            {
                *other /= total;
            }
        }
    }
    
    //#[derive(Clone)]
    //#[derive(Debug)]
    //#[derive(PartialEq)]
    //enum SuperTile
    //{
    //    Tile(TileId),
    //    Field([Vec<f64>; directions.len()]),
    //    Dead
    //}
    
    #[derive(Clone)]
    #[derive(Copy)]
    #[derive(Debug)]
    #[derive(PartialEq)]
    //enum SuperTile
    enum TileType
    {
        Tile(usize),
        Field,
        Dead
    }
    
    impl TileType
    {
        fn is_real(&self) -> bool
        {
            match self
            {
                //SuperTile::Tile(_) => true,
                TileType::Tile(_) => true,
                _ => false
            }
        }
    }
    
    let width  = 10*2;
    let height =  8*2;
    //let width  = 8*8;
    //let height = 8*8;
    //let mut out_map : Vec<SuperTile> = vec!(SuperTile::Field(Default::default()); (width+2)*(height+2));
    // one cell at a time, one direction at a time, one tile at a time, one row at a time
    let mut out_map_fields : Vec<f64> = vec!(1.0; (width+2)*(height+2)*directions.len()*max_index);
    let mut out_map_types : Vec<TileType> = vec!(TileType::Field; (width+2)*(height+2));
    let get_type = |out_map_types : &Vec<TileType>, x : usize, y : usize|
    {
        let index = y*(width+2)+x;
        out_map_types[index].clone()
    };
    let get_all_fields : Box<dyn for<'a> Fn(&'a Vec<f64>, usize, usize) -> &'a [f64]> = Box::new(|out_map_fields, x, y|
    {
        let start_index = (y*(width+2)+x)*directions.len()*max_index;
        let end_index = start_index + directions.len()*max_index;
        &out_map_fields[start_index..end_index]
    });
    let get_fields : Box<dyn for<'a> Fn(&'a Vec<f64>, usize, usize, usize) -> &'a [f64]> = Box::new(|out_map_fields, x, y, direction|
    {
        let start_index = (y*(width+2)+x)*directions.len()*max_index + direction*max_index;
        let end_index = start_index + max_index;
        &out_map_fields[start_index..end_index]
    });
    let get_type_mut : Box<dyn for<'a> Fn(&'a mut Vec<TileType>, usize, usize) -> &'a mut TileType> = Box::new(|out_map_types, x , y |
    {
        let index = y*(width+2)+x;
        &mut out_map_types[index]
    });
    let get_all_fields_mut : Box<dyn for<'a> Fn(&'a mut Vec<f64>, usize, usize) -> &'a mut [f64]> = Box::new(|out_map_fields, x , y|
    {
        let start_index = (y*(width+2)+x)*directions.len()*max_index;
        let end_index = start_index + directions.len()*max_index;
        &mut out_map_fields[start_index..end_index]
    });
    let get_fields_mut : Box<dyn for<'a> Fn(&'a mut Vec<f64>, usize, usize, usize) -> &'a mut [f64]> = Box::new(|out_map_fields, x, y, direction|
    {
        let start_index = (y*(width+2)+x)*directions.len()*max_index + direction*max_index;
        let end_index = start_index + max_index;
        &mut out_map_fields[start_index..end_index]
    });
    
    
    let edge_weight = |origin : TileId, edge : TileId, direction : usize| -> f64
    {
        if forbidden_tiles.contains(&edge) { 0.0 }
        else { ships[origin][direction][edge] }
    };
    let edge_weight_reverse = |origin : TileId, edge : TileId, direction : usize| -> f64
    {
        if forbidden_tiles.contains(&edge) { 0.0 }
        else { ships[edge][get_opposite_direction(direction)][origin] }
    };
    let actual_weight = |(tile_type, fields) : (TileType, &[f64]), edge : TileId, direction : usize| -> f64
    {
        if forbidden_tiles.contains(&edge)
        {
            return 0.0;
        }
        match tile_type
        {
            TileType::Tile(id) => edge_weight_reverse(id, edge, direction),
            TileType::Field =>
            {
                let mut total = 0.0;
                for id in 0..max_index
                {
                    
                    let mut glob = 1.0;
                    for i in 0..directions.len()
                    {
                        glob *= fields[i*max_index + id];
                    }
                    if glob == 0.0
                    {
                        continue
                    }
                    else
                    {
                        //total += edge_weight(i, edge, direction)
                        total += edge_weight(id, edge, direction)
                           * fields[direction*max_index + id]
                           //* glob
                           ;
                    }
                }
                total
            }
            _ => return 1.0
        }
    };
    
    let mut damage = Vec::new();
    let mut candidates = Vec::new();
    
    let mut scratch_fields = vec!(1.0; max_index*directions.len());
    
    let mut recalculate = |
        out_map_types : &mut Vec<TileType>,
        out_map_fields : &mut Vec<f64>,
        damage : &mut Vec<(usize, usize)>,
        candidates : &mut Vec<(usize, usize)>,
        (x, y) : (usize, usize),
        basic_mode : bool
        |
    {
        let in_bounds = |x, y| -> bool { x > 0 && x < width+1 && y > 0 && y < height+1 };
        if !in_bounds(x, y)
        {
            return 0;
        }
        let center_type = get_type(out_map_types, x, y);
        if center_type != TileType::Field
        {
            return 0;
        }
        
        let mut damaged = false;
        let mut num_real_neighbors = 0;
        let mut neighbor_realness = [false; directions.len()];
        // copy_from_slice
        scratch_fields.copy_from_slice(get_all_fields(out_map_fields, x, y));
        
        for (direction, offset) in directions.iter().enumerate()
        {
            let offset = ((offset.0 + x as isize) as usize, (offset.1 + y as isize) as usize);
            let neighbor_type = get_type(out_map_types, offset.0, offset.1).clone();
            if neighbor_type.is_real()
            {
                num_real_neighbors += 1;
                neighbor_realness[direction] = true;
            }
            if neighbor_type == TileType::Dead
            {
                continue;
            }
            
            let neighbor_fields = get_all_fields(out_map_fields, offset.0, offset.1);
            for j in 0..max_index
            {
                if scratch_fields[j] != 0.0
                {
                    let mut modifier = actual_weight((neighbor_type, &neighbor_fields), j, get_opposite_direction(direction));
                    if modifier != 0.0
                    {
                        while (modifier.powi(8) as f32).is_subnormal()
                        {
                            modifier *= 2.0;
                        }
                        if neighbor_type.is_real() && !basic_mode
                        {
                            scratch_fields[j] = modifier;
                        }
                        else if !basic_mode
                        {
                            let orig = scratch_fields[j];
                            // controls the overall amount of chaos in the system; pure modifier is low-chaos
                            //field[j] = field[j]*0.995 + modifier*0.005;
                            //field[j] = field[j]*0.99 + modifier*0.01;
                            //field[j] = field[j]*0.985 + modifier*0.015;
                            //field[j] = field[j]*0.98 + modifier*0.02;
                            //field[j] = field[j]*0.9 + modifier*0.1;
                            //field[j] = 0.05 + field[j]*0.9 + modifier*0.05;
                            scratch_fields[j] = 0.005 + scratch_fields[j]*0.985 + modifier*0.010;
                            // don't let numbers get so small they might get clamped to 0
                            while (scratch_fields[j].powi(8) as f32).is_subnormal()
                            {
                                scratch_fields[j] *= 2.0;
                            }
                            if scratch_fields[j] == 0.0
                            {
                                scratch_fields[j] = orig;
                            }
                        }
                    }
                    else
                    {
                        scratch_fields[j] = 0.0;
                    }
                    if scratch_fields[j] == 0.0
                    {
                        damaged = true;
                    }
                }
            }
        }
        
        let fields = get_all_fields_mut(out_map_fields, x, y);
        for i in 0..scratch_fields.len()
        {
            fields[i] = scratch_fields[i];
        }
        
        let mut dead = false;
        if damaged
        {
            for dir in directions.iter()
            {
                damage.push(((x as isize+dir.0) as usize, (y as isize+dir.1) as usize));
            }
            dead = true;
            for j in 0..max_index
            {
                let mut f = 1.0;
                for i in 0..directions.len()
                {
                    f *= fields[i*max_index + j];
                }
                if f.is_subnormal() || f == 0.0
                {
                    for i in 0..directions.len()
                    {
                        fields[i*max_index + j] = 0.0;
                    }
                    continue;
                }
                else
                {
                    dead = false;
                }
            }
        }
        
        
        if num_real_neighbors > 0 && !candidates.contains(&(x, y))
        {
            candidates.push((x, y));
        }
        if damaged && dead
        {
            *get_type_mut(out_map_types, x, y) = TileType::Dead;
            println!("!!!!---- killed tile at {},{}", x, y);
            return 3;
        }
        else if damaged
        {
            return 2;
        }
        else
        {
            return 1;
        }
    };
    
    for y in 0..height+2
    {
        *get_type_mut(&mut out_map_types,       0, y) = TileType::Tile(0);
        *get_type_mut(&mut out_map_types, width+1, y) = TileType::Tile(0);
        damage.push((1, y));
        damage.push((width, y));
    }
    for x in 0..width+2
    {
        *get_type_mut(&mut out_map_types, x,        0) = TileType::Tile(0);
        *get_type_mut(&mut out_map_types, x, height+1) = TileType::Tile(0);
        damage.push((x, 1));
        damage.push((x, height));
    }
    //out_map[0 * (width+2) + 0] = SuperTile::Tile(map[0][0]);
    //out_map[0 * (width+2) + width+1] = SuperTile::Tile(*map[0].last().unwrap());
    //out_map[(height+1) * (width+2) + width+1] = SuperTile::Tile(map.last().unwrap()[0]);
    //*out_map.last_mut().unwrap() = SuperTile::Tile(*map.last().unwrap().last().unwrap());
    
    let write_image = |
        out_map_types : &Vec<TileType>,
        out_map_fields : &Vec<f64>,
        namesuffix,
        highlight : (usize, usize)
        |
    {
        let mut out = DynamicImage::new_rgba8(((width+2)*tilesize) as u32, ((height+2)*tilesize) as u32);
        let out_writer = out.as_mut_rgba8().unwrap();
        let mut output_tile = vec!(RgbaF::new(0.0, 0.0, 0.0, 0.0); tilesize*tilesize);
        for y in 0..height+2
        {
            for x in 0..width+2
            {
                let cell_type = get_type(out_map_types, x, y);
                match cell_type
                {
                    TileType::Tile(id) =>
                    {
                        for ty in 0..tilesize
                        {
                            for tx in 0..tilesize
                            {
                                let px = &id_to_tile[id][ty*tilesize + tx];
                                let rgba = *image::Rgba::from_slice(&[px.r, px.g, px.b, px.a]);
                                out_writer.put_pixel((x*tilesize + tx) as u32, (y*tilesize + ty) as u32, rgba);
                            }
                        }
                    }
                    TileType::Field =>
                    {
                        let cell_fields = get_all_fields(out_map_fields, x, y);
                        output_tile.fill(RgbaF::new(0.0, 0.0, 0.0, 0.0));
                        let mut control = 0.0;
                        for j in 0..directions.len()
                        { 
                            for id in 0..max_index
                            {
                                let f = cell_fields[j*max_index + id] as f32;
                                for ty in 0..tilesize
                                {
                                    for tx in 0..tilesize
                                    {
                                        let px = &id_to_tile[id][ty*tilesize + tx];
                                        output_tile[ty*tilesize + tx].add_mut(&px.to_float().mult(f));
                                    }
                                }
                                control += f;
                            }
                        }
                        for ty in 0..tilesize
                        {
                            for tx in 0..tilesize
                            {
                                let mut px = output_tile[ty*tilesize + tx].mult(1.0/control as f32);
                                let overlay = |a, b|
                                {
                                    if a < 0.5
                                    {
                                        2.0*a*b
                                    }
                                    else
                                    {
                                        1.0-2.0*((1.0-a)*(1.0-b))
                                    }
                                };
                                px.r = overlay(px.r, 0.7);
                                px.g = overlay(px.g, 0.3);
                                px.b = overlay(px.b, 0.3);
                                let px = px.to_u8();
                                let mut rgba = *image::Rgba::from_slice(&[px.r, px.g, px.b, px.a]);
                                out_writer.put_pixel((x*tilesize + tx) as u32, (y*tilesize + ty) as u32, rgba);
                            }
                        }
                    }
                    _ => {}
                }
                if (x, y) == highlight
                {
                    for ty in 0..tilesize
                    {
                        let px = out_writer.get_pixel_mut((x*tilesize + 0         ) as u32, (y*tilesize + ty) as u32);
                        px[0] = 255;
                        px[2] = 255;
                        px[3] = 255;
                        let px = out_writer.get_pixel_mut((x*tilesize + tilesize-1) as u32, (y*tilesize + ty) as u32);
                        px[0] = 255;
                        px[2] = 255;
                        px[3] = 255;
                    }
                    for tx in 0..tilesize
                    {
                        let px = out_writer.get_pixel_mut((x*tilesize + tx) as u32, (y*tilesize + 0         ) as u32);
                        px[0] = 255;
                        px[2] = 255;
                        px[3] = 255;
                        let px = out_writer.get_pixel_mut((x*tilesize + tx) as u32, (y*tilesize + tilesize-1) as u32);
                        px[0] = 255;
                        px[2] = 255;
                        px[3] = 255;
                    }
                }
            }
        }
        if format!("{}", namesuffix).as_str() != ""
        {
            out.save(format!("{}_{}.png", &args[2], namesuffix)).unwrap();
        }
        else
        {
            out.save(&args[2]).unwrap();
        }
    };
    
    let mut collapse_iteration = 0;
    
    let mut recalculate_all = |
        out_map_types : &mut Vec<TileType>,
        out_map_fields : &mut Vec<f64>,
        damage : &mut Vec<(usize, usize)>,
        candidates : &mut Vec<(usize, usize)>,
        collapse_iteration : usize,
        fail_early : bool,
        retry_count : usize,
        |
    {
        let mut i = 0;
        let mut i2 = 0;
        let mut max_failstate = 0;
        while !damage.is_empty()
        {
            let choice = damage.pop().unwrap();
            let failstate = recalculate(out_map_types, out_map_fields, damage, candidates, choice, false);
            max_failstate = std::cmp::max(max_failstate, failstate);
            if failstate > 0
            {
                i += 1;
            }
            if failstate > 2
            {
                i2 += 1;
                //if false && (collapse_iteration == 1 || collapse_iteration == 14 || collapse_iteration == 15)
                {
                    write_image(out_map_types, out_map_fields, format!("{}-{}-b{}", collapse_iteration.to_string(), retry_count, i2), choice);
                }
                if fail_early
                {
                    println!("recalculated {} tiles and short circuited", i);
                    return failstate;   
                }
            }
        }
        //for y in 1..height+1
        //{
        //    for x in 1..width+1
        //    {
        //        let worth_drawing = recalculate(out_map, damage, candidates, (x, y), true);
        //        if worth_drawing == 2
        //        {
        //            panic!("logic error: found something worth re-damaging after exhausting damage options");
        //        }
        //    }
        //}
        println!("recalculated {} tiles", i);
        return max_failstate;
    };
    
    recalculate_all(&mut out_map_types, &mut out_map_fields, &mut damage, &mut candidates, collapse_iteration, false, 0);
    
    #[allow(unused_variables)]
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    //let time = 1627830772369;
    //let time = 1627832395045;
    //let time = 1627836423901;
    //let time = 1627901131263;
    //let time = 1627919001951;
    let time = 1627923822683;
    println!("seed: {}", time);
    let mut rng = oorandom::Rand64::new(time);
    
    //let comp = std::cmp::max(1, (((width+2)*(height+2)) as f32/64.0).floor() as usize);
    //let comp = std::cmp::max(1, (((width+2)*(height+2)) as f32/32.0).floor() as usize);
    let comp = std::cmp::max(1, (((width+2)*(height+2)) as f32/16.0).floor() as usize);
    //let comp = std::cmp::max(1, (((width+2)*(height+2)) as f32/8.0).floor() as usize);
    //let comp = 1;
    
    let mut out_freqs = vec!(0.0; max_index);
    let mut out_total_freq = 0.0;
    macro_rules! add_to_freq {
        ($id:expr) => 
        {
            out_freqs[$id] += 1.0;
            out_total_freq += 1.0;
        }
    }
    macro_rules! get_freq {
        ($id:expr) => 
        {
            if out_total_freq > 0.0
            {
                (out_freqs[$id]+1.0)/out_total_freq
            }
            else
            {
                1.0
            }
        }
    }
    macro_rules! get_freq_multiplier {
        ($id:expr) => 
        {
            {
                let truth = freqs[$id]/total_freq;
                let local = get_freq!($id);
                if truth == 0.0
                {
                    0.0
                }
                else if local == 0.0
                {
                    1.0
                }
                else
                {
                    truth/local
                }
            }
        }
    }
    
    let mut collapse = |
        choice : (usize, usize),
        rng : &mut oorandom::Rand64,
        out_map_types : &mut Vec<TileType>,
        out_map_fields : &mut Vec<f64>,
        damage : &mut Vec<(usize, usize)>,
        candidates : &mut Vec<(usize, usize)>,
        retry_count : usize| -> bool
    {
        if get_type(out_map_types, choice.0, choice.1) != TileType::Field
        {
            return false;
        }
        let fields = get_all_fields(out_map_fields, choice.0, choice.1);
        let mut decision = 0;
        
        let mut total = 0.0;
        let mut possible_fields = Vec::new();
        
        for i in 0..max_index
        {
            let mut f = get_freq_multiplier!(i);
            for dir in 0..directions.len()
            {
                f *= fields[dir*max_index + i];
            }
            total += f;
            if f != 0.0
            {
                possible_fields.push(i);
            }
        }
        let force = possible_fields.len() < 2;
        if false
        {
            if possible_fields.len() > 0
            {
                decision = possible_fields[rng.rand_range(0..possible_fields.len() as u64) as usize];
            }
        }
        else
        {
            let n = rng.rand_float()*total;
            //assert!(total > 0.0);
            let mut total = 0.0;
            for i in 0..max_index
            {
                let mut f = get_freq_multiplier!(i);
                for dir in 0..directions.len()
                {
                    f *= fields[dir*max_index + i];
                }
                if f == 0.0
                {
                    continue;
                }
                total += f;
                if total >= n
                {
                    decision = i;
                    break;
                }
            }
            //assert!(decision != 0);
        }
        if decision == 0
        {
            println!("!!!!!===== picking a random candidate failed, using the most common tiles");
            println!("!!!!!===== (this means probability recalculation or damage tracking has a bug somewhere!)");
            decision = most_common;
            // FIXME: use a random neighbor instead?
            // this is a fallback case though (being here means that probability recalculation has a bug)
        }
        let old_map_types = out_map_types.clone();
        let old_map_fields = out_map_fields.clone();
        let old_damage = damage.clone();
        let old_candidates = candidates.clone();
        *get_type_mut(out_map_types, choice.0, choice.1) = TileType::Tile(decision);
        if false && collapse_iteration%comp == 0
        {
            println!("writing image for {}", collapse_iteration);
            write_image(out_map_types, out_map_fields, format!("{}-a1", collapse_iteration.to_string()), choice);
            println!("wrote image");
        }
        for dir in directions.iter()
        {
            damage.push(((choice.0 as isize + dir.0) as usize, (choice.1 as isize + dir.1) as usize));
        }
        let mut failed = recalculate_all(out_map_types, out_map_fields, damage, candidates, collapse_iteration, !force, retry_count) == 3;
        if force && failed
        {
            println!("!!!--- failed with tile {} at {},{}, but forced to live with it", decision, choice.0, choice.1);
        }
        else if failed
        {
            println!("!!!--- failed with tile {} at {},{}, retrying", decision, choice.0, choice.1);
            *out_map_types = old_map_types;
            *out_map_fields = old_map_fields;
            *damage = old_damage;
            *candidates = old_candidates;
            candidates.push(choice);
            
            for i in 0..directions.len()
            {
                get_fields_mut(out_map_fields, choice.0, choice.1, i)[decision] = 0.0;
            }
            println!("invalidating decision {} at {},{}", decision, choice.0, choice.1);
            
            for dir in directions.iter()
            {
                damage.push(((choice.0 as isize + dir.0) as usize, (choice.1 as isize + dir.1) as usize));
            }
            recalculate_all(out_map_types, out_map_fields, damage, candidates, collapse_iteration, false, retry_count);
        }
        else
        {
            add_to_freq!(decision);
            if collapse_iteration%comp == 0
            {
                println!("writing image for {}", collapse_iteration);
                write_image(out_map_types, out_map_fields, collapse_iteration.to_string(), choice);
                println!("wrote image");
            }
            collapse_iteration += 1;
        }
        return failed;
    };
    
    
    if false
    {
        for y in 1..height+1
        {
            for x in 1..width+1
            {
                let choice = (x, y);
                let mut i = 0;
                while collapse(choice, &mut rng, &mut out_map_types, &mut out_map_fields, &mut damage, &mut candidates, i)
                {
                    i += 1;
                }
            }
        }
    }
    else
    {
        while !candidates.is_empty()
        {
            let choice_index = rng.rand_range(0..candidates.len() as u64) as usize;
            let choice = candidates.remove(choice_index);
            let mut i = 0;
            while collapse(choice, &mut rng, &mut out_map_types, &mut out_map_fields, &mut damage, &mut candidates, i)
            {
                i += 1;
            }
        }
    }
    
    let mut dead_tiles = Vec::new();
    for y in 1..height+1
    {
        for x in 1..width+1
        {
            match get_type(&out_map_types, x, y)
            {
                TileType::Dead =>
                    dead_tiles.push((x, y)),
                TileType::Field =>
                    panic!("oops! an undecided tile was let through to the end of the algorithm! this indicates a bug with propagating probabilities"),
                _ => continue
            }
        }
    }
    
    if !dead_tiles.is_empty()
    {
        println!("!!!!---- failed tile coordinates:");
        for (x, y) in dead_tiles.iter()
        {
            println!("{},{}", x, y);
        }
        println!("these tiles will be decided based on the neighbors they ended up with");
        println!("the tiles in these locations may not match their neighbors");
    }
    
    // bring dead cells back to life
    /*
    while !dead_tiles.is_empty()
    {
        let (x, y) = dead_tiles.pop().unwrap();
        let index = y * (width+2) + x;
        match &out_map[index]
        {
            TileType::Dead => {}
            _ => continue
        }
        let mut priority = Vec::<(usize, usize, usize, f64)>::new(); // (dir_index, map_index, tile_type, freq)
        for (dir_index, offset) in directions.iter().enumerate()
        {
            let map_index = (index as isize + (offset.1) * (width+2) as isize + offset.0) as usize;
            let tile = &out_map[map_index];
            match tile
            {
                SuperTile::Tile(tile_type) =>
                {
                    priority.push((dir_index, map_index, *tile_type, freqs[*tile_type]));
                }
                _ => {}
            }
        }
        priority.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        let mut success = false;
        while !priority.is_empty()
        {
            let mut edges : [Vec<f64>; directions.len()] = Default::default();
            for edge in edges.iter_mut()
            {
                *edge = vec!(1.0; (max_index) as usize);
            }
            //let mut dummy_tile = SuperTile::Field(edges);
            for (dir_index, map_index, tile_type, freq) in &priority
            {
                let mut edge = &mut edges[*dir_index];
                for center_type in 0..edge.len()
                {
                    edge[center_type] = edge_weight(center_type, *tile_type, *dir_index);
                }
            }
            let mut mix_edges = vec!(1.0; (max_index) as usize);
            for i in 0..directions.len()
            {
                for j in 0..mix_edges.len()
                {
                    mix_edges[j] *= edges[i][j];
                }
            }
            
            let mut decision = 0;
            let mut best_decision_chance = 0.0;
            for i in 0..mix_edges.len()
            {
                if mix_edges[i] > best_decision_chance
                {
                    decision = i;
                    best_decision_chance = mix_edges[i];
                }
            }
            
            if decision > 0
            {
                out_map[index] = SuperTile::Tile(decision);
                success = true;
                break;
            }
            
            priority.pop();
        }
        if !success
        {
            dead_tiles.push((x, y));
        }
    }
    */
    
    write_image(&out_map_types, &out_map_fields, "".to_string(), (!0usize, !0usize));
}










