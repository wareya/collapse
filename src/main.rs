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
    r : f64,
    g : f64,
    b : f64,
    a : f64
}
impl RgbaF {
    #[inline(always)]
    fn new(r : f64, g : f64, b : f64, a : f64) -> RgbaF
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
    #[inline(always)]
    fn mult(&self, thing : f64) -> RgbaF
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
        RgbaF{r : self.r as f64/255.0, g : self.g as f64/255.0, b : self.b as f64/255.0, a : self.a as f64/255.0}
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

fn get_tile_from_image(a : &Vec<Vec<Rgba>>, (x, y) : (usize, usize)) -> Vec<Vec<Rgba>>
{
    let mut ret = vec!(vec!(Rgba::new(255, 255, 255, 255); tilesize); tilesize);
    
    for iy in 0..tilesize
    {
        for ix in 0..tilesize
        {
            ret[iy][ix] = a[y*tilesize+iy][x*tilesize+ix];
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
    let mut id_to_tile = BTreeMap::new();
    let mut map = Vec::with_capacity(img.height() as usize/tilesize);
    for y in 0..img.height() as usize/tilesize
    {
        let mut row = Vec::with_capacity(img.width() as usize/tilesize);
        for x in 0..img.width() as usize/tilesize
        {
            let tile = get_tile_from_image(&px_map, (x, y));
            let id = tile_to_id.entry(tile.clone()).or_insert_with(|| {max_index += 1; max_index - 1});
            id_to_tile.entry(*id).or_insert_with(|| tile);
            
            row.push(*id);
            //print!("{} ", *id);
        }
        //println!();
        map.push(row);
    }
    println!("number of unique tiles: {}", max_index);
    
    let mut freqs = vec!(0.0; max_index);
    let mut ships = Vec::with_capacity(max_index);
    for _ in 0..max_index
    {
        ships.push(vec![
            default_neighbor(max_index),
            default_neighbor(max_index),
            default_neighbor(max_index),
            default_neighbor(max_index)
        ]);
    }
    println!("freqs len: {}", freqs.len());
    println!("ships len: {}", ships.len());
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let opposite_directions = [(-1, 0), (0, -1),(1, 0),  (0, 1)];
    
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
    
    for i in 0..ships.len()
    {
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
    
    #[derive(Clone)]
    #[derive(Debug)]
    #[derive(PartialEq)]
    enum SuperTile
    {
        Tile(TileId),
        Field([Vec<f64>; 4]),
    }
    
    
    let width  = 10;
    let height =  8;
    let mut out_map : Vec<Vec<SuperTile>> = vec!(vec!(SuperTile::Field(
        [vec!(1.0; (max_index) as usize),
         vec!(1.0; (max_index) as usize),
         vec!(1.0; (max_index) as usize),
         vec!(1.0; (max_index) as usize),
        ]); width+2); height+2);
    
    
    //let self_weight = |tile, value : TileId| -> f64
    //{
    //    match tile
    //    {
    //        SuperTile::Tile(value) => 1.0,
    //        SuperTile::Field(field) => field[value],
    //        _ => 0.0
    //    }
    //};
    
    let mut forbidden_tiles = BTreeSet::new();
    forbidden_tiles.insert(map[1][0]);
    forbidden_tiles.insert(*map[1].last().unwrap());
    forbidden_tiles.insert(map[0][1]);
    forbidden_tiles.insert(map.last().unwrap()[1]);
    forbidden_tiles.insert(map[0][0]);
    forbidden_tiles.insert(*map[0].last().unwrap());
    forbidden_tiles.insert(map.last().unwrap()[0]);
    forbidden_tiles.insert(*map.last().unwrap().last().unwrap());
    
    let edge_weight = |origin : TileId, edge : TileId, direction : usize| -> f64
    {
        if forbidden_tiles.contains(&edge) { 0.0 }
        else { ships[origin][direction][edge] }
    };
    let edge_weight_reverse = |origin : TileId, edge : TileId, direction : usize| -> f64
    {
        if forbidden_tiles.contains(&edge) { 0.0 }
        else { ships[edge][(direction+2)%4][origin] }
    };
    let actual_weight = |tile : &SuperTile, edge : TileId, direction : usize| -> f64
    {
        if forbidden_tiles.contains(&edge)
        {
            return 0.0;
        }
        match tile
        {
            SuperTile::Tile(id) => edge_weight(*id, edge, direction),
            SuperTile::Field(fields) =>
            {
                let mut total = 0.0;
                for (id, f) in fields[direction].iter().enumerate()
                {
                    let glob = fields[0][id] * fields[1][id] * fields[2][id] * fields[3][id];
                    if glob == 0.0
                    {
                        continue
                    }
                    else
                    {
                        //total += edge_weight(i, edge, direction)
                        total += edge_weight_reverse(id, edge, direction)
                           * f
                           //* glob
                           ;
                    }
                }
                total
            }
        }
    };
    
    let mut damage = BTreeSet::new();
    let mut candidates = BTreeSet::new();
    
    let recalculate_around = |
        out_map : &mut Vec<Vec<SuperTile>>,
        damage : &mut BTreeSet<(usize, usize)>,
        candidates : &mut BTreeSet<(usize, usize)>,
        (x, y) : (usize, usize)
        | -> bool
    {
        let in_bounds = |out_map : &Vec<Vec<SuperTile>>, (x, y)| -> bool { x < out_map[0].len() && y < out_map.len() };
        if !in_bounds(out_map, (x, y))
        {
            return false;
        }
        let base = out_map[y][x].clone();
        for (direction, offset) in directions.iter().enumerate()
        {
            let offset = ((offset.0 + x as isize) as usize, (offset.1 + y as isize) as usize);
            let in_bounds = |out_map : &Vec<Vec<SuperTile>>, (x, y)| -> bool { x > 0 && x < out_map[0].len()-1 && y > 0 && y < out_map.len()-1 };
            if !in_bounds(out_map, offset)
            {
                continue;
            }
            let tile = &mut out_map[offset.1][offset.0];
            let mut num_possible_ids = 0;
            let mut last_possible_id = 0;
            let mut damaged = false;
            match tile
            {
                SuperTile::Tile(_) => continue,
                SuperTile::Field(fields) =>
                {
                    let mut origin_is_real_tile = false;
                    match base
                    {
                        SuperTile::Tile(_) =>
                        {
                            origin_is_real_tile = true;
                            if !candidates.contains(&offset)
                            {
                                candidates.insert(offset);
                                //println!("added {},{} to candidates", offset.0, offset.1);
                            }
                        }
                        _ => {}
                    }
                    let field = &mut fields[(direction+2)%4];
                    let mut total = 0.0;
                    for j in 0..field.len()
                    {
                        if field[j] != 0.0
                        {
                            let modifier = actual_weight(&base, j, direction);
                            if modifier != 0.0
                            {
                                if origin_is_real_tile
                                {
                                    field[j] = field[j]*0.2 + modifier*0.8; // controls the overall amount of chaos in the system; pure modifier is low-chaos, pure field is high-chaos
                                    if field[j] == 0.0 // because of numerical instability/underflow
                                    {
                                        field[j] = modifier;
                                    }
                                }
                            }
                            else
                            {
                                field[j] = 0.0;
                            }
                            if field[j] == 0.0
                            {
                                damaged = true;
                            }
                        }
                        total += field[j];
                        if field[j] > 0.0
                        {
                            num_possible_ids += 1;
                            last_possible_id = j;
                        }
                    }
                    if total > 0.0
                    {
                        for j in 0..field.len()
                        {
                            field[j] /= total;
                        }
                    }
                    drop(field);
                    if damaged
                    {
                        //for y in -1..1
                        //{
                        //    for x in -1..1
                        //    {
                        //        damage.insert((offset.0 + x as usize, offset.1 + y as usize));
                        //    }
                        //}
                        damage.insert(offset);
                        for j in 0..fields[0].len()
                        {
                            for i in 0..fields.len()
                            {
                                if fields[i][j] == 0.0
                                {
                                    fields[0][j] = 0.0;
                                    fields[1][j] = 0.0;
                                    fields[2][j] = 0.0;
                                    fields[3][j] = 0.0;
                                }
                            }
                        }
                    }
                    //assert!(total > 0.0, "failure at {},{} with {}", offset.0, offset.1, total);
                }
            }
            //if num_possible_ids == 1
            //{
            //    *tile = SuperTile::Tile(last_possible_id);
            //    if !damaged
            //    {
            //        damage.insert(offset);
            //    }
            //}
        }
        true
    };
    
    for y in 0..height+2
    {
        out_map[y][0]       = SuperTile::Tile(map[1][0]);
        out_map[y][width+1] = SuperTile::Tile(*map[1].last().unwrap());
        damage.insert((0, y));
        damage.insert((width+1, y));
    }
    for x in 0..width+2
    {
        out_map[0][x]        = SuperTile::Tile(map[0][1]);
        out_map[height+1][x] = SuperTile::Tile(map.last().unwrap()[1]);
        damage.insert((x, 0));
        damage.insert((x, height+1));
    }
    out_map[0][0] = SuperTile::Tile(map[0][0]);
    *out_map[0].last_mut().unwrap() = SuperTile::Tile(*map[0].last().unwrap());
    out_map.last_mut().unwrap()[0] = SuperTile::Tile(map.last().unwrap()[0]);
    *out_map.last_mut().unwrap().last_mut().unwrap() = SuperTile::Tile(*map.last().unwrap().last().unwrap());
    
    let write_image = |out_map : &mut Vec<Vec<SuperTile>>, namesuffix, highlight : (usize, usize)|
    {
        let mut out = DynamicImage::new_rgba8((out_map[0].len()*tilesize) as u32, (out_map.len()*tilesize) as u32);
        let out_writer = out.as_mut_rgba8().unwrap();
        for y in 0..out_map.len()
        {
            for x in 0..out_map[0].len()
            {
                let cell = &out_map[y][x];
                match cell
                {
                    SuperTile::Tile(id) =>
                    {
                        for ty in 0..tilesize
                        {
                            for tx in 0..tilesize
                            {
                                let px = &id_to_tile[&id][ty][tx];
                                let rgba = *image::Rgba::from_slice(&[px.r, px.g, px.b, px.a]);
                                *out_writer.get_pixel_mut((x*tilesize + tx) as u32, (y*tilesize + ty) as u32) = rgba;
                            }
                        }
                    }
                    SuperTile::Field(fields) =>
                    {
                        for ty in 0..tilesize
                        {
                            for tx in 0..tilesize
                            {
                                let mut sum2 = RgbaF::new(0.0, 0.0, 0.0, 0.0);
                                for field in fields
                                { 
                                    let mut sum = RgbaF::new(0.0, 0.0, 0.0, 0.0);
                                    let mut total_f = 0.0;
                                    for (id, f) in field.iter().enumerate()
                                    {
                                        let px = &id_to_tile[&id][ty][tx];
                                        sum = sum.add(&px.to_float().mult(*f));
                                        total_f += f;
                                    }
                                    sum2 = sum2.add(&sum.mult(1.0/total_f));
                                }
                                sum2.g *= 0.75;
                                sum2.b *= 0.75;
                                let sum = sum2.mult(1.0/4.0).to_u8();
                                let mut rgba = *image::Rgba::from_slice(&[sum.r, sum.g, sum.b, sum.a]);
                                *out_writer.get_pixel_mut((x*tilesize + tx) as u32, (y*tilesize + ty) as u32) = rgba;
                            }
                        }
                    }
                }
                if (x, y) == highlight
                {
                    for ty in 0..tilesize
                    {
                        let px = out_writer.get_pixel_mut((x*tilesize + 0         ) as u32, (y*tilesize + ty) as u32);
                        px[0] = 255;
                        px[2] = 255;
                        let px = out_writer.get_pixel_mut((x*tilesize + tilesize-1) as u32, (y*tilesize + ty) as u32);
                        px[0] = 255;
                        px[2] = 255;
                    }
                    for tx in 0..tilesize
                    {
                        let px = out_writer.get_pixel_mut((x*tilesize + tx) as u32, (y*tilesize + 0         ) as u32);
                        px[0] = 255;
                        px[2] = 255;
                        let px = out_writer.get_pixel_mut((x*tilesize + tx) as u32, (y*tilesize + tilesize-1) as u32);
                        px[0] = 255;
                        px[2] = 255;
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
    
    let recalculate_all = |
        out_map : &mut Vec<Vec<SuperTile>>,
        damage : &mut BTreeSet<(usize, usize)>,
        candidates : &mut BTreeSet<(usize, usize)>,
        collapse_iteration : usize
        |
    {
        let mut i = 0;
        while !damage.is_empty()
        {
            let choice = damage.iter().next().unwrap().clone();
            damage.remove(&choice);
            recalculate_around(out_map, damage, candidates, choice);
            i += 1;
            if collapse_iteration == 1 || collapse_iteration == 14 || collapse_iteration == 15
            {
                write_image(out_map, format!("{}-b{}", collapse_iteration.to_string(), i), choice);
            }
        }
        println!("recalcuated {} tiles", i);
    };
    
    recalculate_all(&mut out_map, &mut damage, &mut candidates, collapse_iteration);
    
    #[allow(unused_variables)]
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    //let time = 1627830772369;
    //let time = 1627832395045;
    let time = 1627836423901;
    println!("seed: {}", time);
    let mut rng = oorandom::Rand64::new(time);
    
    let comp = std::cmp::max(1, (((width+2)*(height+2)) as f32/64.0).floor() as usize);
    let comp = std::cmp::max(1, (((width+2)*(height+2)) as f32/8.0).floor() as usize);
    let comp = 1;
    
    let mut collapse = |choice : (usize, usize),
        rng : &mut oorandom::Rand64,
        out_map : &mut Vec<Vec<SuperTile>>,
        damage : &mut BTreeSet<(usize, usize)>,
        candidates : &mut BTreeSet<(usize, usize)>| -> bool
    {
        let cell = &out_map[choice.1][choice.0];
        let mut decision = 0;
        let mut force = false;
        match cell
        {
            SuperTile::Tile(_) => return false,
            SuperTile::Field(fields) =>
            {
                let mut total = 0.0;
                let mut possible_fields = Vec::new();
                for i in 0..fields[0].len()
                {
                    let f = fields[0][i] * fields[1][i] * fields[2][i] * fields[3][i];
                    total += f;
                    if f != 0.0
                    {
                        possible_fields.push(i);
                    }
                }
                force = possible_fields.len() < 2;
                if true
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
                    for i in 0..fields[0].len()
                    {
                        let f = fields[0][i] * fields[1][i] * fields[2][i] * fields[3][i];
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
            }
        }
        if decision == 0
        {
            decision = most_common;
        }
        let old_copy = out_map.clone();
        let old_damage = damage.clone();
        let old_candidates = candidates.clone();
        out_map[choice.1][choice.0] = SuperTile::Tile(decision);
        if collapse_iteration%comp == 0
        {
            println!("writing image for {}", collapse_iteration);
            write_image(out_map, format!("{}-a1", collapse_iteration.to_string()), choice);
            println!("wrote image");
        }
        damage.insert(choice);
        recalculate_all(out_map, damage, candidates, collapse_iteration);
        let get_failed = ||
        {
            for row in out_map.iter()
            {
                for cell in row
                {
                    match cell
                    {
                        SuperTile::Tile(_) => {},
                        SuperTile::Field(fields) =>
                        {
                            let mut total = 0.0;
                            for i in 0..fields[0].len()
                            {
                                let f = fields[0][i] * fields[1][i] * fields[2][i] * fields[3][i];
                                total += f;
                            }
                            if total == 0.0
                            {
                                if !force
                                {
                                    println!("!!!!--- failed, rewinding");
                                    return true;
                                }
                                else
                                {
                                    println!("!!!!--- failed rewinding but forced to live with it");
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
            false
        };
        let failed = [get_failed(), false][1];
        if failed
        {
            *out_map = old_copy;
            *damage = old_damage;
            *candidates = old_candidates;
            candidates.insert(choice);
            match &mut out_map[choice.1][choice.0]
            {
                SuperTile::Tile(_) => {},
                SuperTile::Field(fields) =>
                {
                    fields[0][decision] = 0.0;
                    fields[1][decision] = 0.0;
                    fields[2][decision] = 0.0;
                    fields[3][decision] = 0.0;
                }
            }
            damage.insert(choice);
            recalculate_all(out_map, damage, candidates, collapse_iteration);
        }
        else
        {
            if collapse_iteration%comp == 0
            {
                println!("writing image for {}", collapse_iteration);
                write_image(out_map, collapse_iteration.to_string(), choice);
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
                while collapse(choice, &mut rng, &mut out_map, &mut damage, &mut candidates)
                {
                    // self-terminating
                }
            }
        }
    }
    else
    {
        while !candidates.is_empty()
        {
            let choice_index = rng.rand_range(0..candidates.len() as u64) as usize;
            let choice = candidates.iter().nth(choice_index).unwrap().clone();
            candidates.remove(&choice);
            while collapse(choice, &mut rng, &mut out_map, &mut damage, &mut candidates)
            {
                // self-terminating
            }
        }
    }
    
    /*
    while !candidates.is_empty()
    {
        let choice_index = rng.rand_range(0..candidates.len() as u32) as usize;
        let choice = candidates_list[choice_index].clone();
        candidates_list.remove(choice_index);
        candidates.remove(&choice);
        //println!("picked {},{} from candidates", choice.0, choice.1);
        let cell = &out_map[choice.1][choice.0];
        let mut decision = 0;
        match cell
        {
            SuperTile::Tile(_) => continue,
            SuperTile::Field(fields) =>
            {
                let mut field = fields[0].clone();
                for (i, f) in fields[1].iter().enumerate()
                {
                    field[i] *= f;
                }
                for (i, f) in fields[2].iter().enumerate()
                {
                    field[i] *= f;
                }
                for (i, f) in fields[3].iter().enumerate()
                {
                    field[i] *= f;
                }
                let mut total = 0.0;
                for f in field
                {
                    total += f;
                }
                let n = rng.rand_float()*total;
                //assert!(total > 0.0);
                let mut total = 0.0;
                for (i, f) in field.iter().enumerate()
                {
                    if *f == 0.0
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
        }
        if decision == 0
        {
            decision = most_common;
        }
        out_map[choice.1][choice.0] = SuperTile::Tile(decision);
        damage.insert(choice);
        recalculate_all(&mut out_map, &mut damage, &mut candidates, &mut candidates_list);
        
        if i%50 == 0
        {
            write_image(&mut out_map, i.to_string());
        }
        i += 1;
    }
    */
    
    write_image(&mut out_map, "".to_string(), (!0usize, !0usize));
}










