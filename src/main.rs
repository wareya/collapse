extern crate image;

use std::env::args as args;
use image::io::Reader as ImageReader;
#[allow(unused_imports)]
use image::{DynamicImage, GenericImageView, Pixel};
use std::collections::{BTreeMap};

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
    //#[inline(always)]
    //fn add(&self, other : &RgbaF) -> RgbaF
    //{
    //    RgbaF{r : self.r + other.r, g : self.g + other.g, b : self.b + other.b, a : self.a + other.a}
    //}
    #[inline(always)]
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

const TILESIZE : usize = 16;
const DIRECTIONS : [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
//const DIRECTIONS : [(isize, isize); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
fn get_opposite_direction(dir_index : usize) -> usize
{
    (dir_index+DIRECTIONS.len()/2)%DIRECTIONS.len()
}

fn get_tile_from_image(a : &Vec<Vec<Rgba>>, (x, y) : (usize, usize)) -> Vec<Rgba>
{
    let mut ret = vec!(Rgba::new(255, 255, 255, 255); TILESIZE*TILESIZE);
    
    for iy in 0..TILESIZE
    {
        for ix in 0..TILESIZE
        {
            ret[iy*TILESIZE + ix] = a[y*TILESIZE+iy][x*TILESIZE+ix];
        }
    }
    
    ret
}

type TileId = usize;


//#[derive(Clone)]
//#[derive(Debug)]
//#[derive(PartialEq)]
//enum SuperTile
//{
//    Tile(TileId),
//    Field([Vec<f64>; DIRECTIONS.len()]),
//    Dead
//}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
//enum SuperTile
enum TileType
{
    Tile(TileId),
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
    
    struct Collapser
    {
        max_index : TileId,
        //tile_to_id : BTreeMap<Vec<Rgba>, usize>,
        id_to_tile : Vec<Vec<Rgba>>,
        //map : Vec<TileId>,
        forbidden_tiles : Vec<TileId>,
        freqs : Vec<f64>,
        total_freq : f64,
        ships : Vec<f64>,
        most_common : TileId,
        width : usize,
        height : usize,
        out_map_fields : Vec<f64>,
        out_map_types : Vec<TileType>,
    
        damage : Vec<(isize, isize)>,
        candidates : Vec<(isize, isize)>,
    
        out_freqs : Vec<f64>,
        out_total_freq : f64,
        
        namebase : String
    }
    
    impl Collapser
    {
        fn init(px_map : &Vec<Vec<Rgba>>, width : usize, height : usize, namebase : String) -> Collapser
        {
            let mut max_index = 0;
            let mut tile_to_id = BTreeMap::new();
            let mut id_to_tile = Vec::new();
            let mut map = Vec::with_capacity(height as usize/TILESIZE);
            for y in 0..height/TILESIZE
            {
                let mut row = Vec::with_capacity(width as usize/TILESIZE);
                for x in 0..width/TILESIZE
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
            
            let mut forbidden_tiles = Vec::new();
            forbidden_tiles.push(map[0][0]);
            
            //let mut ships = Vec::with_capacity(max_index);
            //for _ in 0..max_index
            //{
            //    ships.push(vec![default_neighbor(max_index); DIRECTIONS.len()]);
            //}
            let mut ships = vec!(0.0; max_index*max_index*DIRECTIONS.len());
    
            let mut freqs = vec!(0.0; max_index);
            
            //fn in_bounds(map : &Vec<Vec<TileId>>, (x, y) : (isize, isize)) -> bool
            //{
            //    x >= 0 && x < map[0].len() as isize && y >= 0 && y < map.len() as isize
            //}
            
            #[allow(unused_assignments)]
            let mut most_common = 0;
            let mut most_common_freq = 0.0;
            for y in 0..map.len()
            {
                for x in 0..map[0].len()
                {
                    freqs[map[y][x]] += 1.0;
                    let id = map[y][x];
                    let freq = freqs[map[y][x]];
                    if freq > most_common_freq
                    {
                        most_common_freq = freq;
                        most_common = id;
                    }
                    for (i, offset) in DIRECTIONS.iter().enumerate()
                    {
                        let offset = (offset.0 + x as isize, offset.1 + y as isize);
                        //if in_bounds(&map, offset)
                        {
                            // wrapping
                            let value = map[offset.1.wrapping_rem_euclid(map.len() as isize) as usize][offset.0.wrapping_rem_euclid(map[0].len() as isize) as usize];
                            // non-wrapping
                            //value = map[offset.1 as usize][offset.0 as usize];
                            //ships[id][i][value] += 1.0;
                            ships[id*max_index*DIRECTIONS.len() + value*DIRECTIONS.len() + i] += 1.0;
                            if id == 1 && value == 0 && i == 1
                            {
                                println!("added entry for {}->{} in direction {} at {},{}", id, value, i, x, y);
                            }
                            //println!("added to direction {} for tile type {}", i, id);
                            //ships[id][i][value] = 1.0;
                        }
                        //else
                        //{
                        //    //println!("direction {} for tile centered at {},{} is out of bounds", i, x, y);
                        //}
                    }
                }
            }
            
            let total_freq : f64 = freqs.iter().sum();
            
            for a in 0..max_index
            {
                if forbidden_tiles.contains(&a) // FIXME: ?????????
                {
                    continue;
                }
                for direction in 0..DIRECTIONS.len()
                {
                    //for other in &ships[i][j]
                    //{
                    //    //total += other;
                    //}
                    let mut total = 0.0;
                    
                    //ships[a*max_index*DIRECTIONS.len() + b*DIRECTIONS.len() + direction]
                    for b in 0..max_index
                    {
                        let other = ships[a*max_index*DIRECTIONS.len() + b*DIRECTIONS.len() + direction];
                        total += other;
                    }
                    assert!(total > 0.0, "failed tile type {} direction {}", a, direction);
                    for b in 0..max_index
                    {
                        let other = &mut ships[a*max_index*DIRECTIONS.len() + b*DIRECTIONS.len() + direction];
                        *other /= total;
                    }
                }
            }
            
            // ensure that all neighbor facts of form "A can exist to the right of B" match all facts of the form "B can exist to the left of A"
            for a in 0..max_index
            {
                for b in 0..max_index
                {
                    for dir in 0..DIRECTIONS.len()
                    {
                        //ships[a*max_index*DIRECTIONS.len() + b*DIRECTIONS.len() + dir]
                        let aw = ships[a*max_index*DIRECTIONS.len() + b*DIRECTIONS.len() + dir];
                        let bw = ships[b*max_index*DIRECTIONS.len() + a*DIRECTIONS.len() + get_opposite_direction(dir)];
                        if (aw == 0.0) != (bw == 0.0)
                        {
                            panic!("{} and {} don't match up for direction {} ({} vs {})", a, b, dir, aw, bw);
                        }
                    }
                }
            }
    
            let mut width  = 10*4;
            let mut height =  8*4;
            //let mut width  = 10*2;
            //let mut height =  8*2;
            width  += 2;
            height += 2;
            //let width  = 8*8;
            //let height = 8*8;
            //let mut out_map : Vec<SuperTile> = vec!(SuperTile::Field(Default::default()); (width+2)*(height+2));
            // one cell at a time, one direction at a time, one tile at a time, one row at a time
            let out_map_fields = vec!(1.0; width*height*DIRECTIONS.len()*max_index);
            let out_map_types  = vec!(TileType::Field; width*height);
            
            let candidates = Vec::new();
            let damage = Vec::new();
            
            let out_freqs = vec!(0.0; max_index);
            let out_total_freq = 0.0;
            
            let mut collapse = Collapser { max_index, id_to_tile, forbidden_tiles, freqs, total_freq, ships, most_common, out_map_fields, out_map_types, width, height, out_freqs, out_total_freq, damage, candidates, namebase };
            
            collapse.init_edges();
            
            collapse
        }
        fn init_edges(&mut self)
        {
            for y in 0..self.height as isize
            {
                *self.get_type_mut(           0, y) = TileType::Tile(0);
                *self.get_type_mut(self.width as isize-1, y) = TileType::Tile(0);
                self.damage.push((1, y));
                self.damage.push((self.width as isize-2, y));
            }
            for x in 0..self.width as isize
            {
                *self.get_type_mut(x,             0) = TileType::Tile(0);
                *self.get_type_mut(x, self.height as isize-1) = TileType::Tile(0);
                self.damage.push((x, 1));
                self.damage.push((x, self.height as isize-2));
            }
        }
        #[inline(always)]
        fn get_type(&self, x : isize, y : isize) -> TileType
        {
            let x = x.wrapping_rem_euclid(self.width as isize) as usize;
            let y = y.wrapping_rem_euclid(self.height as isize) as usize;
            let index = y*self.width+x;
            self.out_map_types[index]
        }
        #[inline(always)]
        fn get_all_fields<'a>(&'a self, x : isize, y : isize) -> &'a [f64]
        {
            let x = x.wrapping_rem_euclid(self.width as isize) as usize;
            let y = y.wrapping_rem_euclid(self.height as isize) as usize;
            let start_index = (y*self.width+x)*DIRECTIONS.len()*self.max_index;
            let end_index = start_index + DIRECTIONS.len()*self.max_index;
            &self.out_map_fields[start_index..end_index]
        }
        //fn get_fields<'a>(&'a self, x : usize, y : usize, direction : usize) -> &'a [f64]
        //{
        //    let x = x%self.width;
        //    let y = y%self.height;
        //    let start_index = (y*self.width+x)*DIRECTIONS.len()*self.max_index + direction*self.max_index;
        //    let end_index = start_index + self.max_index;
        //    &self.out_map_fields[start_index..end_index]
        //}
        #[inline(always)]
        fn get_type_mut<'a>(&'a mut self, x : isize, y : isize) -> &'a mut TileType
        {
            let x = x.wrapping_rem_euclid(self.width as isize) as usize;
            let y = y.wrapping_rem_euclid(self.height as isize) as usize;
            let index = y*self.width+x;
            &mut self.out_map_types[index]
        }
        #[inline(always)]
        fn get_all_fields_mut<'a>(&'a mut self, x : isize, y : isize) -> &'a mut [f64]
        {
            let x = x.wrapping_rem_euclid(self.width as isize) as usize;
            let y = y.wrapping_rem_euclid(self.height as isize) as usize;
            let start_index = (y*self.width+x)*DIRECTIONS.len()*self.max_index;
            let end_index = start_index + DIRECTIONS.len()*self.max_index;
            &mut self.out_map_fields[start_index..end_index]
        }
        //fn get_fields_mut<'a>(&'a mut self, x : usize, y : usize, direction : usize) -> &'a mut [f64]
        //{
        //    let x = x%self.width;
        //    let y = y%self.height;
        //    let start_index = (y*self.width+x)*DIRECTIONS.len()*self.max_index + direction*self.max_index;
        //    let end_index = start_index + self.max_index;
        //    &mut self.out_map_fields[start_index..end_index]
        //}
        #[inline(always)]
        fn edge_weight(&self, a : TileId, b : TileId, dir : usize) -> f64
        {
            //if self.forbidden_tiles.contains(&edge) { 0.0 }
            //else { self.ships[origin][direction][edge] }
            
            let index = a*self.max_index*DIRECTIONS.len() + b*DIRECTIONS.len() + dir;
            //let index = index%self.ships.len();
            //self.ships[index]
            unsafe { *self.ships.get_unchecked(index) }
            //self.ships[origin][direction][edge]
        }
        #[inline(always)]
        fn actual_weight(&self, (tile_type, fields) : (TileType, &[f64]), edge : TileId, direction : usize) -> f64
        {
            match tile_type
            {
                //TileType::Tile(id) => self.edge_weight(edge, id, get_opposite_direction(direction)), // ????????????
                TileType::Tile(id) => self.edge_weight(id, edge, direction),
                TileType::Field =>
                {
                    //let opposite_direction = get_opposite_direction(direction);
                    let mut total = 0.0;
                    for id in 0..self.max_index
                    {
                        //let mut glob : f64 = 1.0;
                        //for i in 0..DIRECTIONS.len()
                        //{
                        //    glob *= fields[i + id*DIRECTIONS.len()];
                        //}
                        let glob;
                        if DIRECTIONS.len() == 4 // optimizes away
                        {
                            // yes this is slightly faster
                            unsafe
                            {
                                glob = (*fields.get_unchecked(0 + id*DIRECTIONS.len()) * *fields.get_unchecked(2 + id*DIRECTIONS.len()))
                                     * (*fields.get_unchecked(1 + id*DIRECTIONS.len()) * *fields.get_unchecked(3 + id*DIRECTIONS.len()));
                            }
                        }
                        else if DIRECTIONS.len() == 8 // optimizes away
                        {
                            unsafe
                            {
                                glob = ((*fields.get_unchecked(0 + id*DIRECTIONS.len()) * *fields.get_unchecked(4 + id*DIRECTIONS.len()))
                                      * (*fields.get_unchecked(2 + id*DIRECTIONS.len()) * *fields.get_unchecked(6 + id*DIRECTIONS.len())))
                                     * ((*fields.get_unchecked(1 + id*DIRECTIONS.len()) * *fields.get_unchecked(5 + id*DIRECTIONS.len()))
                                      * (*fields.get_unchecked(3 + id*DIRECTIONS.len()) * *fields.get_unchecked(7 + id*DIRECTIONS.len())));
                            }
                        }
                        else
                        {
                            panic!("DIRECTIONS must beeither four or eight elements long");
                        }
                        if glob != 0.0
                        {
                            unsafe
                            {
                                total += self.edge_weight(id, edge, direction) * *fields.get_unchecked(direction + id*DIRECTIONS.len());
                            }
                        }
                    }
                    total// / self.max_index as f64
                }
                _ => 1.0
            }
        }
        
        fn write_image<T : ToString + std::fmt::Display>(
            &self,
            namesuffix : T,
            highlight : (isize, isize),
            )
        {
            let mut out = DynamicImage::new_rgba8((self.width*TILESIZE) as u32, (self.height*TILESIZE) as u32);
            let out_writer = out.as_mut_rgba8().unwrap();
            let mut output_tile = vec!(RgbaF::new(0.0, 0.0, 0.0, 0.0); TILESIZE*TILESIZE);
            for y in 0..self.height
            {
                for x in 0..self.width
                {
                    let cell_type = self.get_type(x as isize, y as isize);
                    match cell_type
                    {
                        TileType::Tile(id) =>
                        {
                            for ty in 0..TILESIZE
                            {
                                for tx in 0..TILESIZE
                                {
                                    let px = &self.id_to_tile[id][ty*TILESIZE + tx];
                                    let rgba = *image::Rgba::from_slice(&[px.r, px.g, px.b, px.a]);
                                    out_writer.put_pixel((x*TILESIZE + tx) as u32, (y*TILESIZE + ty) as u32, rgba);
                                }
                            }
                        }
                        TileType::Field =>
                        {
                            let cell_fields = self.get_all_fields(x as isize, y as isize);
                            output_tile.fill(RgbaF::new(0.0, 0.0, 0.0, 0.0));
                            let mut control = 0.0;
                            for id in 0..self.max_index
                            {
                                for j in 0..DIRECTIONS.len()
                                { 
                                    let f = cell_fields[j + id*DIRECTIONS.len()] as f32;
                                    for ty in 0..TILESIZE
                                    {
                                        for tx in 0..TILESIZE
                                        {
                                            let px = &self.id_to_tile[id][ty*TILESIZE + tx];
                                            output_tile[ty*TILESIZE + tx].add_mut(&px.to_float().mult(f));
                                        }
                                    }
                                    control += f;
                                }
                            }
                            for ty in 0..TILESIZE
                            {
                                for tx in 0..TILESIZE
                                {
                                    let mut px = output_tile[ty*TILESIZE + tx].mult(1.0/control as f32);
                                    fn overlay (a : f32, b : f32) -> f32
                                    {
                                        if a < 0.5
                                        {
                                            2.0*a*b
                                        }
                                        else
                                        {
                                            1.0-2.0*((1.0-a)*(1.0-b))
                                        }
                                    }
                                    px.r = overlay(px.r, 0.7);
                                    px.g = overlay(px.g, 0.3);
                                    px.b = overlay(px.b, 0.3);
                                    let px = px.to_u8();
                                    let rgba = *image::Rgba::from_slice(&[px.r, px.g, px.b, px.a]);
                                    out_writer.put_pixel((x*TILESIZE + tx) as u32, (y*TILESIZE + ty) as u32, rgba);
                                }
                            }
                        }
                        _ => {}
                    }
                    if (x, y) == (highlight.0 as usize, highlight.1 as usize)
                    {
                        for ty in 0..TILESIZE
                        {
                            let px = out_writer.get_pixel_mut((x*TILESIZE + 0         ) as u32, (y*TILESIZE + ty) as u32);
                            px[0] = 255;
                            px[2] = 255;
                            px[3] = 255;
                            let px = out_writer.get_pixel_mut((x*TILESIZE + TILESIZE-1) as u32, (y*TILESIZE + ty) as u32);
                            px[0] = 255;
                            px[2] = 255;
                            px[3] = 255;
                        }
                        for tx in 0..TILESIZE
                        {
                            let px = out_writer.get_pixel_mut((x*TILESIZE + tx) as u32, (y*TILESIZE + 0         ) as u32);
                            px[0] = 255;
                            px[2] = 255;
                            px[3] = 255;
                            let px = out_writer.get_pixel_mut((x*TILESIZE + tx) as u32, (y*TILESIZE + TILESIZE-1) as u32);
                            px[0] = 255;
                            px[2] = 255;
                            px[3] = 255;
                        }
                    }
                }
            }
            if format!("{}", namesuffix).as_str() != ""
            {
                out.save(format!("{}_{}.png", self.namebase, namesuffix)).unwrap();
            }
            else
            {
                out.save(&self.namebase).unwrap();
            }
        }
        fn recalculate (
            &mut self,
            scratch_fields : &mut Vec<f64>,
            (x, y) : (isize, isize),
            ) -> usize
        {
            let center_type = self.get_type(x, y);
            if center_type != TileType::Field
            {
                return 0;
            }
            
            let mut damaged = false;
            let mut num_real_neighbors = 0;
            // copy_from_slice
            scratch_fields.copy_from_slice(self.get_all_fields(x, y));
            
            for offset in DIRECTIONS.iter()
            {
                let neighbor_coord = (offset.0 + x, offset.1 + y);
                let neighbor_type = self.get_type(neighbor_coord.0, neighbor_coord.1).clone();
                if neighbor_type.is_real()
                {
                    num_real_neighbors += 1;
                }
            }
            
            let mut i = 0;
            let neighbors = DIRECTIONS.map(|offset : (isize, isize)|
            {
                //let offset : (isize, isize) = DIRECTIONS[direction];
                let neighbor_coord = (offset.0 + x, offset.1 + y);
                let neighbor_type = self.get_type(neighbor_coord.0, neighbor_coord.1);
                let neighbor_fields = self.get_all_fields(neighbor_coord.0, neighbor_coord.1);
                i += 1;
                (i-1, neighbor_type, neighbor_fields)
            });
            
            for (direction, neighbor_type, neighbor_fields) in neighbors.iter()
            {
                for j in 0..self.max_index
                {
                    let direction = *direction;
                    let neighbor_type = *neighbor_type;
                    
                    let index = j*DIRECTIONS.len() + direction;
                    
                    //assert!(index < scratch_fields.len());
                    if scratch_fields[index] != 0.0
                    {
                        let fiddle = if j != 0 { 1.0 } else { 0.0 };
                        let mut modifier = self.actual_weight((neighbor_type, &neighbor_fields), j, get_opposite_direction(direction));
                        modifier *= fiddle;
                        if modifier != 0.0
                        {
                            modifier = f64::max(modifier, 0.0001); // avoid making floats that, when raised to the 8th power, risk becoming subnormal
                            
                            let fiddle2 = if neighbor_type.is_real() { 1.0 } else { 0.0 };
                            let fiddle3 = 1.0 - fiddle2;
                            
                            //scratch_fields[index] = fiddle2 * modifier + fiddle3 * ( 0.005 + scratch_fields[index]*0.985 + modifier*0.010);
                            scratch_fields[index] = fiddle2 * modifier + fiddle3 * ( 0.005 + scratch_fields[index]*0.975 + modifier*0.020);
                            
                            //if neighbor_type.is_real()
                            //{
                            //    scratch_fields[index] = modifier;
                            //}
                            //else
                            //{
                            //    // controls the overall amount of chaos in the system; pure modifier is low-chaos
                            //    //field[j] = field[j]*0.995 + modifier*0.005;
                            //    //field[j] = field[j]*0.99 + modifier*0.01;
                            //    //field[j] = field[j]*0.985 + modifier*0.015;
                            //    //field[j] = field[j]*0.98 + modifier*0.02;
                            //    //field[j] = field[j]*0.9 + modifier*0.1;
                            //    //field[j] = 0.05 + field[j]*0.9 + modifier*0.05;
                            //    scratch_fields[index] = 0.005 + scratch_fields[index]*0.985 + modifier*0.010;
                            //    
                            //    
                            //    // unnecessary as long as the chaoticness expression has the 0.005 term in it
                            //    // don't let numbers get so small they might get clamped to 0
                            //    //if (scratch_fields[j].powi(8) as f32).is_subnormal()
                            //    //{
                            //    //    scratch_fields[j] = modifier;
                            //    //}
                            //}
                        }
                        else
                        {
                            scratch_fields[index] = 0.0;
                            damaged = true;
                        }
                    }
                }
            }
            
            let fields = self.get_all_fields_mut(x, y);
            fields.copy_from_slice(&scratch_fields[..]);
            
            if damaged
            {
                for dir in DIRECTIONS.iter()
                {
                    self.damage.push((x+dir.0, y+dir.1));
                }
            }
            
            let max_index = self.max_index;
            let fields = self.get_all_fields_mut(x, y);
            
            let mut dead = false;
            if damaged
            {
                dead = true;
                for j in 0..max_index
                {
                    let mut f = 1.0;
                    for i in 0..DIRECTIONS.len()
                    {
                        f *= fields[i + j*DIRECTIONS.len()];
                    }
                    if f == 0.0 || f.is_subnormal()
                    {
                        for i in 0..DIRECTIONS.len()
                        {
                            fields[i + j*DIRECTIONS.len()] = 0.0;
                        }
                        continue;
                    }
                    else
                    {
                        dead = false;
                    }
                }
            }
            
            if num_real_neighbors > 0 && !self.candidates.contains(&(x, y))
            {
                self.candidates.push((x, y));
            }
            if damaged && dead
            {
                *self.get_type_mut(x, y) = TileType::Dead;
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
        }
    
        fn recalculate_all (
            &mut self,
            scratch_fields : &mut Vec<f64>,
            collapse_iteration : usize,
            fail_early : bool,
            retry_count : usize,
            ) -> usize
        {
            let mut i = 0;
            let mut i2 = 0;
            let mut max_failstate = 0;
            while !self.damage.is_empty()
            {
                let choice = self.damage.pop().unwrap();
                let failstate = self.recalculate(scratch_fields, choice);
                max_failstate = std::cmp::max(max_failstate, failstate);
                if failstate > 0
                {
                    i += 1;
                }
                if failstate > 2
                {
                    i2 += 1;
                    self.write_image(format!("{}-{}-b{}", collapse_iteration.to_string(), retry_count, i2), choice);
                    if fail_early
                    {
                        println!("recalculated {} tiles and short circuited", i);
                        return failstate;   
                    }
                }
            }
            println!("recalculated {} tiles", i);
            max_failstate
        }
        
        fn add_to_freq(&mut self, id : TileId)
        {
            self.out_freqs[id] += 1.0;
            self.out_total_freq += 1.0;
        }
        fn get_freq(&self, id : TileId) -> f64
        {
            if self.out_total_freq > 0.0
            {
                (self.out_freqs[id]+1.0)/self.out_total_freq
            }
            else
            {
                1.0
            }
        }
        fn get_freq_multiplier(&self, id : TileId) -> f64
        {
            let truth = self.freqs[id]/self.total_freq;
            let local = self.get_freq(id);
            1000000000.0 * if local == 0.0
            {
                1.0
            }
            else
            {
                truth/local
            }
        }
        fn collapse (
            &mut self,
            scratch_fields : &mut Vec<f64>,
            collapse_iteration : usize,
            choice : (isize, isize),
            rng : &mut oorandom::Rand64,
            retry_count : usize
            ) -> bool
        {
            if self.get_type(choice.0, choice.1) != TileType::Field
            {
                return false;
            }
            let fields = self.get_all_fields(choice.0, choice.1);
            let mut decision = 0;
            
            let mut total = 0.0;
            let mut possible_fields = Vec::new();
            
            for i in 0..self.max_index
            {
                let mut f = self.get_freq_multiplier(i);
                for dir in 0..DIRECTIONS.len()
                {
                    f *= fields[dir + i*DIRECTIONS.len()];
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
                for i in 0..self.max_index
                {
                    let mut f = self.get_freq_multiplier(i);
                    for dir in 0..DIRECTIONS.len()
                    {
                        f *= fields[dir + i*DIRECTIONS.len()];
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
                decision = self.most_common;
                // FIXME: use a random neighbor instead?
                // this is a fallback case though (being here means that probability recalculation has a bug)
            }
            //let old_map_types = self.out_map_types.clone();
            let old_map_fields = self.out_map_fields.clone();
            let old_damage = self.damage.clone();
            let old_candidates = self.candidates.clone();
            *self.get_type_mut(choice.0, choice.1) = TileType::Tile(decision);
            
            //let comp = std::cmp::max(1, ((self.width*self.height) as f32/64.0).floor() as usize);
            //let comp = std::cmp::max(1, ((self.width*self.height) as f32/32.0).floor() as usize);
            //let comp = std::cmp::max(1, ((self.width*self.height) as f32/16.0).floor() as usize);
            let comp = std::cmp::max(1, ((self.width*self.height) as f32/8.0).floor() as usize);
            //let comp = 1;
            
            if false && collapse_iteration%comp == 0
            {
                println!("writing image for {}", collapse_iteration);
                self.write_image(format!("{}-a1", collapse_iteration.to_string()), choice);
                println!("wrote image");
            }
            for dir in DIRECTIONS.iter()
            {
                self.damage.push((choice.0 + dir.0, choice.1 + dir.1));
            }
            let failed = self.recalculate_all(scratch_fields, collapse_iteration, !force, retry_count) == 3;
            if force && failed
            {
                println!("!!!--- failed with tile {} at {},{}, but forced to live with it", decision, choice.0, choice.1);
            }
            else if failed
            {
                println!("!!!--- failed with tile {} at {},{}, retrying", decision, choice.0, choice.1);
                *self.get_type_mut(choice.0, choice.1) = TileType::Field;
                //self.out_map_types = old_map_types;
                self.out_map_fields = old_map_fields;
                self.damage = old_damage;
                self.candidates = old_candidates;
                self.candidates.push(choice);
                
                println!("invalidating decision {} at {},{}", decision, choice.0, choice.1);
                
                self.damage.push((choice.0, choice.1));
                for (i, dir) in DIRECTIONS.iter().enumerate()
                {
                // uncomment if invalidation doesn't work properly
                    self.get_all_fields_mut(choice.0, choice.1)
                    [decision*DIRECTIONS.len() + i] = 0.0;
                    self.get_all_fields_mut(choice.0 + dir.0, choice.1 + dir.1)
                    [decision*DIRECTIONS.len() + get_opposite_direction(i)] = 0.0;
                // uncomment if invalidation doesn't work properly
                    self.damage.push((choice.0 + dir.0, choice.1 + dir.1));
                }
                
                self.recalculate_all(scratch_fields, collapse_iteration, false, retry_count);
            }
            else
            {
                self.add_to_freq(decision);
                if collapse_iteration%comp == 0
                {
                    println!("writing image for {}", collapse_iteration);
                    self.write_image(&collapse_iteration.to_string(), choice);
                    println!("wrote image");
                }
            }
            return failed;
        }
    }
    
    
    //fn init(px_map : &Vec<Rgba>, width : usize, height : usize, namebase : String) -> Collapser
    
    let mut collapse_iteration = 0;
    let mut collapser = Collapser::init(&px_map, img.width() as usize, img.height() as usize, args[2].to_string());
    let mut scratch_fields = vec!(1.0; collapser.max_index*DIRECTIONS.len());
    collapser.recalculate_all(&mut scratch_fields, collapse_iteration, false, 0);
    collapse_iteration += 1;
    
    
    #[allow(unused_variables)]
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    //let time = 1627830772369;
    //let time = 1627832395045;
    //let time = 1627836423901;
    //let time = 1627901131263;
    //let time = 1627919001951;
    //let time = 1627923822683;
    //let time = 1628290023690;
    let time = 1628290385006;
    println!("seed: {}", time);
    let mut rng = oorandom::Rand64::new(time);
    
    if false
    {
        for y in 0..collapser.height
        {
            for x in 0..collapser.width
            {
                let choice = (x as isize, y as isize);
                let mut i = 0;
                while collapser.collapse(&mut scratch_fields, collapse_iteration, choice, &mut rng, i)
                {
                    i += 1;
                }
                collapse_iteration += 1;
            }
        }
    }
    else
    {
        while !collapser.candidates.is_empty()
        {
            let choice_index = rng.rand_range(0..collapser.candidates.len() as u64) as usize;
            let choice = collapser.candidates.remove(choice_index);
            let mut i = 0;
            while collapser.collapse(&mut scratch_fields, collapse_iteration, choice, &mut rng, i)
            {
                i += 1;
            }
            collapse_iteration += 1;
        }
    }
    
    let mut dead_tiles = Vec::new();
    for y in 0..collapser.height as isize
    {
        for x in 0..collapser.width as isize
        {
            match collapser.get_type(x, y)
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
        let index = y*collapser.width + x;
        match collapser.get_type(x, y)
        {
            TileType::Dead => {}
            _ => continue
        }
        let mut priority = Vec::<(usize, usize, usize, f64)>::new(); // (dir_index, map_index, tile_type, freq)
        for (dir_index, offset) in DIRECTIONS.iter().enumerate()
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
            let mut edges : [Vec<f64>; DIRECTIONS.len()] = Default::default();
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
            for i in 0..DIRECTIONS.len()
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
    
    collapser.write_image(&"".to_string(), (-1, -1));
}










