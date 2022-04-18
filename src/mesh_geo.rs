use petgraph::graph::NodeIndex;
use petgraph::Graph;

use serde::{Deserialize, Serialize};
use serde_json::Result;

use std::collections::HashMap;
use std::fs;

use std::cmp;

use vec_map::VecMap;

use std::time::Instant;

use std::cell::Cell;

use std::convert::TryInto;
use geo_clipper::Clipper;
use geo_types::{Coordinate, LineString, MultiPolygon, Polygon};

#[path = "algebra.rs"] mod algebra;
#[path = "heat_map.rs"] mod heat_map;

type EdgeWeight = (u64, Line);

pub struct Map {
    pub extended_tiles: Vec<PreTile>,
    pub tiles: Vec<PreTile>,
    pub grid: VecMap<PreTile>,
    pub tiles_cols: i64,
    pub tiles_rows: i64,
    pub bounds: (Point, Point),
    pub graph: Graph<(u64, PrePoly), EdgeWeight>,
    pub links: Vec<(u64, u64, Line)>,
    pub polygons: Vec<Vec<Polygon<f64>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct PolyData {
    verts: Vec<u64>,
    vertCount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TileData {
    polys: Vec<PolyData>,
    points: Vec<(i32, i32, f32)>,
    bounds: ((i64, i32, i64), (i64, i32, i64)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreTile {
    pub sourcefile: String,
    pub areas: Vec<PrePoly>,
    pub bounds: ((i64, i32, i64), (i64, i32, i64)),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedMap {
    pub extended_tiles: Vec<PreTile>,
    pub tiles: Vec<PreTile>,
    pub links: Vec<(u64, u64, Line)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PrePoly {
    pub verts: Vec<(i32, i32, f32)>,
    pub vert_count: u64,
    pub center: (i32, i32),
    pub id: u64,
}

pub type PrePolyWithHeat = (PrePoly, i32);

pub type Line = ((i32, i32, f32), (i32, i32, f32));


pub fn point_inside_poly(x: i32, y: i32, poly: &PrePoly) -> i32 {
    let polygon: Vec<(f64, f64)> = poly
        .verts
        .iter()
        .map(|(x, y, _)| {
            return (*x as f64, *y as f64);
        })
        .collect();
    return algebra::point_inside_poly((x as f64, y as f64), &polygon);
}

pub fn poly_area(poly: &PrePoly) -> f64 {
    // Initialze area
    let mut area: f64 = 0.0;
    // Calculate value of shoelace formula
    let mut j = poly.verts.len() - 1;
    for i in 0..poly.verts.len() {
        area += ((poly.verts[j].0 + poly.verts[i].0) * (poly.verts[j].1 - poly.verts[i].1)) as f64;
        j = i;
    }
    area /= 2.0;
    return area.abs();
}


pub fn is_intersect_polys(poly1: &PrePoly, poly2: &PrePoly) -> bool {
    let mut lines1 = vec![];
    for v in &poly1.verts {
        lines1.insert(
            lines1.len(),
            Coordinate {
                x: v.0 as f64,
                y: v.1 as f64,
            },
        );
    }
    lines1.insert(
        lines1.len(),
        Coordinate {
            x: poly1.verts[0].0 as f64,
            y: poly1.verts[0].1 as f64,
        },
    );
    let p1 = Polygon::new(LineString(lines1), vec![]);

    let mut lines2 = vec![];
    for v in &poly2.verts {
        lines2.insert(
            lines2.len(),
            Coordinate {
                x: v.0 as f64,
                y: v.1 as f64,
            },
        );
    }
    lines2.insert(
        lines2.len(),
        Coordinate {
            x: poly2.verts[0].0 as f64,
            y: poly2.verts[0].1 as f64,
        },
    );
    let p2 = Polygon::new(LineString(lines2), vec![]);

    // println!("before p1={:?}", p1);
    // println!("before p2={:?}", p2);
    let res = p1.intersection(&p2, 1.0);
    // let res = p1.union(&p2);
    //println!("after {:?}", res);

    let mut polys = 0;
    for x in res {
        polys = polys + 1;
    }
    return polys > 0;
}


pub fn merge_two_polygon(p1: &Polygon<f64>, p2: &Polygon<f64>) -> Option<Polygon<f64>> {
    let res = p1.union(p2, 1.0);
    let mut polys = res.into_iter().collect::<Vec<Polygon<f64>>>();
    if polys.len() != 1 {
        return None;
    }
    return Some(polys[0].clone());
}

pub fn dedup_coordinates(pts: Vec<Coordinate<f64>>) -> Vec<Coordinate<f64>> {
    let mut points: Vec<Coordinate<f64>> = pts.clone();

    let len = points.len();
    for i in (1..len).rev() {
        let mut exist = false;
        for j in (0..i) {
            if points[i].x == points[j].x && points[i].y == points[j].y {
                exist = true;
                break;
            }
        }
        if exist {
            points.remove(i);
        }
    }
    return points;
}

pub fn merge_tiles(tiles: &Vec<&PreTile>) -> PreTile {
    // keep z poses, make polygons
    let mut polys = vec![];
    let mut pos_to_z = HashMap::<(i32, i32), f32>::new();

    for t in tiles {
        for p in &t.areas {
            let mut lines = vec![];
            for v in &p.verts {
                lines.insert(
                    lines.len(),
                    Coordinate {
                        x: v.0 as f64,
                        y: v.1 as f64,
                    },
                );
            }
            lines.insert(
                lines.len(),
                Coordinate {
                    x: p.verts[0].0 as f64,
                    y: p.verts[0].1 as f64,
                },
            );

            let pp = Polygon::new(LineString(lines), vec![]);
            polys.insert(polys.len(), pp);

            for v in &p.verts {
                match pos_to_z.get(&(v.0, v.1)) {
                    Some(_) => (),
                    None => {
                        pos_to_z.insert((v.0, v.1), v.2);
                    }
                }
            }
        }
    }

    //merge polys
    let mut merged_poly = true;
    while merged_poly == true {
        merged_poly = false;
        for i_poly in 0..polys.len() - 1 {
            for j_poly in (i_poly + 1)..polys.len() {
                match merge_two_polygon(&polys[i_poly], &polys[j_poly]) {
                    Some(new_poly) => {
                        merged_poly = true;
                        polys.remove(j_poly);
                        polys.remove(i_poly);
                        polys.insert(i_poly, new_poly);
                    }
                    None => (),
                }
                if merged_poly == true {
                    break;
                }
            }
            if merged_poly == true {
                break;
            }
        }
    }

    // println!("merge_two_tile1={:?}", polys);
    // split into triangles
    let mut new_polys: Vec<PrePoly> = vec![];
    for new_p in &polys {
        let tris = algebra::polygon2tris(&new_p);
        for tri in tris {
            let mut ptts: Vec<(i32, i32, f32)> = vec![];
            ptts.push( ((tri.0).0 as i32, (tri.0).1 as i32, 0.) );
            ptts.push( ((tri.1).0 as i32, (tri.1).1 as i32, 0.));
            ptts.push( ((tri.2).0 as i32, (tri.2).1 as i32, 0.));
            for i in 0..3{
                let mut z: f32 = 0.0;
                match pos_to_z.get(&(ptts[i].0, ptts[i].1)) {
                    Some(z_) => {
                        z = *z_;
                    },
                    None => {
                        z = (tiles[0].bounds.1).1 as f32;
                    }
                }
                ptts[i].2 = z;
            }
            let center = (
                (ptts[0].0 + ptts[1].0 + ptts[2].0) / 3,
                (ptts[0].1 + ptts[1].1 + ptts[2].1) / 3,
            );
            let n_p = PrePoly {
                vert_count: 3,
                verts: ptts,
                center: center,
                id: 0,
            };
            new_polys.insert(new_polys.len(), n_p);
        }
    }

    // for new_p in polys{
    //     let mut points: Vec<Coordinate<f64>> = new_p.exterior().clone().into_iter().collect();
    //     points.remove(points.len()-1);
    //     let pts: Vec<(i32, i32, f32)> = points.iter().map(|pt|{
    //         return (pt.x as i32, pt.y as i32, 0.0);
    //     }).collect();

    //     let n_p = PrePoly{
    //         vert_count: pts.len() as u64,
    //         verts: pts,
    //         center: (0, 0),
    //         id: 0,
    //     };
    //     new_polys.insert(new_polys.len(), n_p);
    // }

    let bounds = tiles[0].bounds;

    return PreTile {
        areas: new_polys,
        bounds: bounds,
        sourcefile: "".to_string(),
    };
}

pub fn merge_tile(tile: &PreTile) -> Vec<Polygon<f64>> {
    // keep z poses, make polygons
    let mut polys = vec![];
    let mut pos_to_z = HashMap::<(i32, i32), f32>::new();
    for p in &tile.areas {
        let mut lines = vec![];
        for v in &p.verts {
            lines.insert(
                lines.len(),
                Coordinate {
                    x: v.0 as f64,
                    y: v.1 as f64,
                },
            );
        }
        lines.insert(
            lines.len(),
            Coordinate {
                x: p.verts[0].0 as f64,
                y: p.verts[0].1 as f64,
            },
        );

        let pp = Polygon::new(LineString(lines), vec![]);
        polys.insert(polys.len(), pp);

        for v in &p.verts {
            match pos_to_z.get(&(v.0, v.1)) {
                Some(_) => (),
                None => {
                    pos_to_z.insert((v.0, v.1), v.2);
                }
            }
        }
    }

    //merge polys
    let mut merged_poly = true;
    while merged_poly == true {
        merged_poly = false;
        for i_poly in 0..polys.len() - 1 {
            for j_poly in (i_poly + 1)..polys.len() {
                match merge_two_polygon(&polys[i_poly], &polys[j_poly]) {
                    Some(new_poly) => {
                        merged_poly = true;
                        polys.remove(j_poly);
                        polys.remove(i_poly);
                        polys.insert(i_poly, new_poly);
                    }
                    None => (),
                }
                if merged_poly == true {
                    break;
                }
            }
            if merged_poly == true {
                break;
            }
        }
    }
    return polys;
}


pub fn files_to_tiles(files: Vec<String>) -> Vec<PreTile> {
    let alltiles: Vec<Vec<PreTile>> = files
        .iter()
        .map(|filename| {
            println!("reading: {:?}", filename);

            let contents =
                fs::read_to_string(filename).expect("Something went wrong reading the file");

            let mut _id: u64 = 0;

            let tiles: Result<Vec<TileData>> = serde_json::from_str(&contents);

            thread_local!(static MONSTER_ID: Cell<u64> = Cell::new(0));

            let mut tiles2: Vec<PreTile> = tiles
                .unwrap()
                .iter()
                .map(|tile| {
                    let mut areas: Vec<PrePoly> = tile
                        .polys
                        .iter()
                        .map(|area| {
                            //slice it by vercount
                            let mut verts: Vec<(i32, i32, f32)> = area
                                .verts
                                .iter()
                                .map(|vert| {
                                    return tile.points[*vert as usize];
                                })
                                .collect();
                            verts.truncate(area.vertCount.try_into().unwrap());

                            let mut t_x = 0;
                            let mut t_y = 0;

                            for n in 0..area.vertCount {
                                t_x += verts[n as usize].0;
                                t_y += verts[n as usize].1;
                            }
                            let center = (t_x / area.vertCount as i32, t_y / area.vertCount as i32);

                            return PrePoly {
                                vert_count: area.vertCount,
                                verts: verts,
                                center: center,
                                id: 0,
                            };
                        })
                        .collect();

                    //tile boundry
                    let tilebounds = (
                        (
                            cmp::max((tile.bounds.0).0, (tile.bounds.1).0),
                            cmp::max((tile.bounds.0).1, (tile.bounds.1).1),
                            cmp::max((tile.bounds.0).2, (tile.bounds.1).2),
                        ),
                        (
                            cmp::min((tile.bounds.0).0, (tile.bounds.1).0),
                            cmp::min((tile.bounds.0).1, (tile.bounds.1).1),
                            cmp::min((tile.bounds.0).2, (tile.bounds.1).2),
                        ),
                    );

                    // remove out of boundry
                    let len = areas.len();
                    for i in (0..len).rev() {
                        let mut b_out = false;
                        for v in &areas[i].verts {
                            if v.0 < (tilebounds.1).0 as i32
                                || v.0 > (tilebounds.0).0 as i32
                                || v.1 < (tilebounds.1).2 as i32
                                || v.1 > (tilebounds.0).2 as i32
                            {
                                b_out = true;
                                break;
                            }
                        }
                        if b_out {
                            areas.remove(i);
                        }
                    }

                    return PreTile {
                        sourcefile: filename.to_string(),
                        areas: areas,
                        bounds: tilebounds,
                    };
                })
                .collect();

            // process dup tiles
            let mut bound_exist = HashMap::<(i64, i64), bool>::new();
            for i_tile in 0..tiles2.len() {
                if tiles2[i_tile].areas.len() == 0 {
                    continue;
                }

                let bounds = tiles2[i_tile].bounds;
                match bound_exist.get(&((bounds.1).0, (bounds.1).2)) {
                    Some(_) => (),
                    None => {
                        bound_exist.insert(((bounds.1).0, (bounds.1).2), true);

                        let mut idxs: Vec<usize> = vec![];
                        let mut tiles00: Vec<&PreTile> = vec![];
                        // collect same boundry tiles
                        for j_tile in i_tile..tiles2.len() {
                            let bounds1 = tiles2[j_tile].bounds;
                            if (bounds.1).0 == (bounds1.1).0 && (bounds.1).2 == (bounds1.1).2 {
                                idxs.insert(idxs.len(), j_tile);
                                tiles00.insert(tiles00.len(), &tiles2[j_tile]);
                            }
                        }

                        if tiles00.len() > 1 {
                            // println!("merge_tiles={:?}", tiles00.len());
                            let new_t = merge_tiles(&tiles00);
                            for j_tile in idxs {
                                tiles2[j_tile].areas.clear();
                            }

                            tiles2.remove(i_tile);
                            tiles2.insert(i_tile, new_t);
                        } else {
                            //check dup polys in same tile
                            let mut exist_dup_poly = false;
                            let len = tiles2[i_tile].areas.len();
                            for i in 0..len {
                                for j in 0..len {
                                    if i == j {
                                        continue;
                                    }
                                    if is_intersect_polys(
                                        &tiles2[i_tile].areas[i],
                                        &tiles2[i_tile].areas[j],
                                    ) {
                                        exist_dup_poly = true;
                                        break;
                                    }
                                }
                                if exist_dup_poly {
                                    break;
                                }
                            }
                            if exist_dup_poly {
                                tiles00.insert(0, &tiles2[i_tile]);
                                let new_t = merge_tiles(&tiles00);
                                tiles2.remove(i_tile);
                                tiles2.insert(i_tile, new_t);
                            }
                        }
                    }
                }
            }

            // merge
            // let mut bound_exist = HashMap::<(i64, i64), (usize, bool)>::new();
            // for i_tile in 0..tiles2.len() {
            //     let bounds = tiles2[i_tile].bounds;
            //     match bound_exist.get(&((bounds.1).0, (bounds.1).2)){
            //         Some((i_org, _)) => {
            //             // println!("merge");
            //             match merge_two_tile(&tiles2[*i_org], &tiles2[i_tile]){
            //                 Some(new_t) => {
            //                     tiles2.remove(*i_org);
            //                     tiles2.insert(*i_org, new_t);
            //                     tiles2[i_tile].areas.clear();
            //                 },
            //                 None => ()
            //             }
            //             bound_exist.insert(((bounds.1).0, (bounds.1).2), (*i_org, true));
            //             // println!("merge 0");
            //         },
            //         None =>{
            //             bound_exist.insert(((bounds.1).0, (bounds.1).2), (i_tile, false));
            //         }
            //     }
            // }

            // // split
            // for i_tile in 0..tiles2.len() {
            //     if tiles2[i_tile].areas.len() == 0{continue;}
            //     let bounds = tiles2[i_tile].bounds;
            //     match bound_exist.get(&((bounds.1).0, (bounds.1).2)){
            //         Some((i_org, merged)) => {
            //             if *merged{
            //                 // println!("split");
            //                 let new_t = split_tile2tri(&tiles2[*i_org]);
            //                 // println!("split0");
            //                 tiles2.remove(*i_org);
            //                 tiles2.insert(*i_org, new_t);
            //                 // println!("split1");
            //             }
            //         },
            //         None =>()
            //     }
            // }

            tiles2.retain(|t| t.areas.len() > 0);

            //set poly ids
            for i_t in 0..tiles2.len() {
                for i_a in 0..tiles2[i_t].areas.len() {
                    let id = MONSTER_ID.with(|thread_id| {
                        let id = thread_id.get();
                        thread_id.set(id + 1);
                        return id;
                    });
                    tiles2[i_t].areas[i_a].id = id;
                }
            }

            return tiles2;
        })
        .collect();

    return alltiles.into_iter().flatten().collect::<Vec<PreTile>>();
}

pub fn files_to_map<'a>(files: Vec<String>, _outname: &str) -> Map {
    let tiles2 = files_to_tiles(files);

    let min_x = (tiles2
        .iter()
        .min_by(|tile, tile2| ((tile.bounds.1).0).cmp(&(tile2.bounds.1).0))
        .unwrap()
        .bounds
        .1)
        .0;
    let min_y = (tiles2
        .iter()
        .min_by(|tile, tile2| ((tile.bounds.1).2).cmp(&(tile2.bounds.1).2))
        .unwrap()
        .bounds
        .1)
        .2;
    let max_x = (tiles2
        .iter()
        .max_by(|tile, tile2| ((tile.bounds.1).0).cmp(&(tile2.bounds.1).0))
        .unwrap()
        .bounds
        .1)
        .0;
    let max_y = (tiles2
        .iter()
        .max_by(|tile, tile2| ((tile.bounds.1).2).cmp(&(tile2.bounds.1).2))
        .unwrap()
        .bounds
        .1)
        .2;
    println!("{} {} {} {}", min_x, max_x, min_y, max_y);
    let tile = tiles2.first();
    let bounds = tile.unwrap().bounds;
    let geom = ((bounds.0).0 - (bounds.1).0, (bounds.0).2 - (bounds.1).2);
    println!("geometry {:?} ", geom);
    let cols = (max_x - min_x) / geom.0;
    let rows = (max_y - min_y) / geom.1;
    println!("{} {}", cols, rows);

    let mut grid: VecMap<PreTile> = VecMap::with_capacity((cols * rows) as usize);

    println!("tiles len: {}", tiles2.len());

    let mut extended_tiles = Vec::<PreTile>::new();

    let mut polygons = Vec::<Vec<Polygon<f64>>>::new();

    let mut dups = 0;
    tiles2.iter().for_each(|tile| {
        let tile_polys = merge_tile(&tile);
        polygons.push(tile_polys);

        let (l, _z, t) = tile.bounds.1;
        let topleft = ((l - min_x) / 1260) + ((t - min_y) / 1260) * cols;
        let prev = grid.insert(topleft as usize, tile.clone());
        match prev {
            Some(v) => {
                let centers1: Vec<(i32, i32)> = tile.areas.iter().map(|area| area.center).collect();
                let centers2: Vec<(i32, i32)> = v.areas.iter().map(|area| area.center).collect();

                let fcenters1: Vec<&(i32, i32)> =
                    centers1.iter().filter(|e| !centers2.contains(e)).collect();
                let fcenters2: Vec<&(i32, i32)> =
                    centers2.iter().filter(|e| !centers1.contains(e)).collect();

                if (fcenters1.len() > 0) | (fcenters2.len() > 0) {
                    dups = dups + 1;
                    /*
                      println!("duplicate tile {:?} {:?}", tile.sourcefile, v.sourcefile);
                                    println!("duplicate tile pos {:?} {:?}", tile.bounds, v.bounds);
                                    println!("{:?} ", tile.areas);
                                    println!("{:?} ", v.areas);
                                    println!("{:?} {:?}", fcenters1, fcenters2);
                    */
                    let mut x = Vec::<PrePoly>::new();
                    for area in &tile.areas {
                        x.push(area.clone())
                    }
                    for area in &v.areas {
                        x.push(area.clone())
                    }

                    extended_tiles.push(PreTile {
                        areas: x,
                        bounds: tile.bounds,
                        sourcefile: "combined".to_string(),
                    });
                }
            }
            None => return,
        }
    });

    extended_tiles.iter().for_each(|tile| {
        let (l, _z, t) = tile.bounds.1;
        let topleft = ((l - min_x) / 1260) + ((t - min_y) / 1260) * cols;
        grid.insert(topleft as usize, tile.clone());
    });

    println!("grids len: {}", grid.len());
    println!("dups: {}", dups);

    let mut links = Vec::new();
    let now = Instant::now();
    // println!("elapsed: {}", now.elapsed().as_millis());
    for (_index, tile) in &grid {
        tile.areas.iter().for_each(|area| {
            let (l, _z, t) = tile.bounds.1;
            let xoff = (l - min_x) / 1260;
            let yoff = (t - min_y) / 1260;
            let pos = [(-1, 0), (0, 0), (0, -1), (0, 1), (1, 0)];
            for (x, y) in pos.iter() {
                if x + xoff >= 0 && x + xoff < cols {
                    if y + yoff >= 0 && y + yoff < rows {
                        let topleft = xoff + x + yoff * cols + y * cols;
                        //println!("{}", topleft);
                        match grid.get(topleft as usize) {
                            Some(atile) => atile.areas.iter().for_each(|area2| {
                                if area.id != area2.id {
                                    match area_intersect(area, area2, false) {
                                        Some(edge) => links.push((area.id, area2.id, edge)),
                                        None => (),
                                    }
                                }
                            }),
                            None => (),
                        };
                    }
                }
            }
        });
    }

    println!("elapsed: {}", now.elapsed().as_millis());

    println!("links: {}", links.len());

    /* let sm = SavedMap {
         extended_tiles: extended_tiles,
         tiles: tiles2,
         links: links
     };
     let mut f = File::create(outname.to_string()).unwrap();
     bincode::serialize_into(&mut f, &sm).unwrap();
    */
    let mut graph = Graph::<(u64, PrePoly), (u64, Line)>::new();

    for tile in grid.values().into_iter() {
        for area in &tile.areas {
            graph.add_node((area.id, area.clone()));
        }
    }

    let mut name_to_node = HashMap::<u64, NodeIndex<u32>>::new();

    for node in graph.node_indices() {
        let (areaid, _) = &graph[node];
        name_to_node.insert(*areaid, node);
    }

    let edges: Vec<(NodeIndex<u32>, NodeIndex<u32>, (u64, Line))> = links
        .iter()
        .map(|(a, b, line)| {
            let an = *name_to_node.get(a).unwrap();
            let bn = *name_to_node.get(b).unwrap();
            let (_, p1) = &graph[an];
            let (_, p2) = &graph[bn];
            let dx = p1.center.0 - p2.center.0;
            let dy = p1.center.1 - p2.center.1;
            let d = ((dx * dx + dy * dy) as f64).sqrt().round() as u64;
            let weight_mul = match line_len(*line) {
                0..=100 => 3.2,
                101..=200 => 2.1,
                _ => 1.0,
            };
            let weight = ((d as f64) * weight_mul).trunc() as u64;
            return (an, bn, (weight, line.clone()));
        })
        .collect();

    graph.extend_with_edges(edges);

    return Map {
        graph: graph,
        bounds: (Point { x: min_x, y: min_y }, Point { x: max_x, y: max_y }),
        tiles: tiles2,
        extended_tiles: extended_tiles,
        grid: grid,
        tiles_cols: cols,
        tiles_rows: rows,
        links: links,
        polygons: polygons,
    };
}

pub fn line_len((a, b): Line) -> u64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    let d = ((dx * dx + dy * dy) as f64).sqrt().round() as u64;
    return d;
}

pub fn segments_overlap(l1: Line, l2: Line) -> Option<Line> {
    let (a1, b1) = l1;
    let (a2, b2) = l2;

    let zeroonx = (a1.0 - b1.0 == 0) || ((a2.0 - b2.0) == 0);
    let nonzeroonx = (a1.0 - b1.0 != 0) || ((a2.0 - b2.0) != 0);
    let zeroony = (a1.1 - b1.1 == 0) || ((a2.1 - b2.1) == 0);
    let nonzeroony = (a1.1 - b1.1 != 0) || ((a2.1 - b2.1) != 0);
    if zeroonx && nonzeroonx {
        return None;
    }
    if zeroony && nonzeroony {
        return None;
    }

    let (x1, y1, _) = a1;
    let (x2, y2, _) = b1;
    let (x3, y3, _) = a2;
    let (x4, y4, _) = b2;

    let tria1 = x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2);
    let tria2 = x1 * (y2 - y4) + x2 * (y4 - y1) + x4 * (y1 - y2);
    if (tria1 != 0) || (tria2 != 0) {
        return None;
    }

    let min1 = (cmp::min(a1.0, b1.0), cmp::min(a1.1, b1.1));
    let max1 = (cmp::max(a1.0, b1.0), cmp::max(a1.1, b1.1));

    let min2 = (cmp::min(a2.0, b2.0), cmp::min(a2.1, b2.1));
    let max2 = (cmp::max(a2.0, b2.0), cmp::max(a2.1, b2.1));

    //l1 in l2 or l2 in l1

    //collinearlity was tested above, if min or max vector are the same

    let min_intersection = (cmp::max(min1.0, min2.0), cmp::max(min1.1, min2.1));
    let max_intersection = (cmp::min(max1.0, max2.0), cmp::min(max1.1, max2.1));

    let _dx = min_intersection.0 - max_intersection.0;
    let _dy = min_intersection.1 - max_intersection.1;

    if (min_intersection.0 > max_intersection.0) | (min_intersection.1 > max_intersection.1) {
        return None;
    }

    if (min_intersection.0 == max_intersection.0) & (min_intersection.1 == max_intersection.1) {
        return None;
    }

    let mut verts = vec![a1, b1, a2, b2];
    if _dx != 0 {
        verts.sort_by_key(|x| x.0);
    } else {
        verts.sort_by_key(|x| x.1);
    }
    return Some((verts[1], verts[2]));

    // return Some((
    //     (min_intersection.0, min_intersection.1, 0.0),
    //     (max_intersection.0, max_intersection.1, 0.0),
    // ));

    // let aa1 = (a1.0 as f64, a1.1 as f64, a1.2 as f32);
    // let bb1 = (b1.0 as f64, b1.1 as f64, b1.2 as f32);
    // let aa2 = (a2.0 as f64, a2.1 as f64, a2.2 as f32);
    // let bb2 = (b2.0 as f64, b2.1 as f64, b2.2 as f32);

    // if segment_intersects(aa1, bb1, aa2, bb2) !=-1{
    //     return None;
    // }

    // let dx_ = b1.0 - a1.0;
    // let mut verts = vec![a1, b1, a2, b2];
    // if dx_ != 0{
    //     verts.sort_by_key(|x| x.0);
    // }
    // else{
    //     verts.sort_by_key(|x| x.1);
    // }
    // return Some((verts[1], verts[2]));
}

pub fn _area_intersect_v(
    area1: Vec<(i32, i32, f32)>,
    area2: Vec<(i32, i32, f32)>,
    debug: bool,
) -> Option<Line> {
    let mut pvert = area1.len() - 1;
    for i in 0..area1.len() {
        let edge1 = (area1[pvert as usize], area1[i as usize]);
        pvert = i;
        let mut jpvert = area2.len() - 1;
        for j in 0..area2.len() {
            let edge2 = (area2[jpvert as usize], area2[j as usize]);
            jpvert = j;
            if debug {
                println!("comparing {:?} {:?}", edge1, edge2);
            }
            let intres = segments_overlap(edge1, edge2);
            if intres != None {
                return intres;
            }
        }
    }
    return None;
}

pub fn area_intersect(area1: &PrePoly, area2: &PrePoly, _debug: bool) -> Option<Line> {
    let mut pvert = area1.vert_count - 1;
    for i in 0..area1.vert_count {
        let edge1 = (area1.verts[pvert as usize], area1.verts[i as usize]);
        pvert = i;
        let mut jpvert = area2.vert_count - 1;
        for j in 0..area2.vert_count {
            let edge2 = (area2.verts[jpvert as usize], area2.verts[j as usize]);
            jpvert = j;
            let intres = segments_overlap(edge1, edge2);
            //if (area1.id == "B242367A207263" && intres);
            //    debug = true;
            //if (debug == true) {
            //    console.log(sp(l1[0]), sp(l1[1]), sp(l2[0]), sp(l2[1]), intres);
            //}
            if intres != None {
                return intres;
            }
        }
    }
    return None;
}

fn hdist(node: &(u64, PrePoly), x: i64, y: i64) -> i64 {
    let (_, area) = &*node;
    let dx = area.center.0 as i64 - x;
    let dy = area.center.1 as i64 - y;
    return dx * dx + dy * dy;
}

fn _dist(area: &PrePoly, x: i64, y: i64) -> i64 {
    let dx = area.center.0 as i64 - x;
    let dy = area.center.1 as i64 - y;
    return dx * dx + dy * dy;
}

pub fn find_closest_node(
    graph: &Graph<(u64, PrePoly), EdgeWeight>,
    x: i64,
    y: i64,
) -> NodeIndex<u32> {
    let mut closest = NodeIndex::<u32>::new(0);
    let mut closestdist = hdist(&graph[closest], x, y);
    for node in graph.node_indices() {
        let tdist = hdist(&graph[node], x, y);
        if tdist < 2000 * 2000 && point_inside_poly(x as i32, y as i32, &graph[node].1) != 0 {
            closest = node;
            break;
        }
        if tdist < closestdist {
            closest = node;
            closestdist = tdist;
        }
    }
    return closest;
}

pub fn get_around_polys(map: &Map, x: i64, y: i64, range: i64) -> Vec<&PrePoly> {
    let dist: i64 = range * range;
    let mut polys = vec![];

    for t in &map.tiles {
        if (t.bounds.1).0 >= x + range {
            continue;
        }
        if (t.bounds.1).2 >= y + range {
            continue;
        }
        if (t.bounds.0).0 <= x - range {
            continue;
        }
        if (t.bounds.0).2 <= y - range {
            continue;
        }

        for i in 0..t.areas.len() {
            let tdist = _dist(&t.areas[i], x, y);
            if tdist < dist {
                polys.push(&t.areas[i]);
            }
        }
    }

    return polys;
}

pub fn get_around_tiles(map: &Map, x: i64, y: i64, range: i64) -> Vec<usize> {
    let mut tiles = vec![];
    for i in 0..map.tiles.len() {
        let t = &map.tiles[i];
        if (t.bounds.1).0 >= x + range {
            continue;
        }
        if (t.bounds.1).2 >= y + range {
            continue;
        }
        if (t.bounds.0).0 <= x - range {
            continue;
        }
        if (t.bounds.0).2 <= y - range {
            continue;
        }
        tiles.push(i);
    }
    return tiles;
}

pub fn find_closest_idx(polys: &Vec<PrePoly>, x: i64, y: i64) -> usize {
    let mut closest = 0;
    let mut closestdist = _dist(&polys[0], x, y);
    for i in 0..polys.len() {
        let tdist = _dist(&polys[i], x, y);
        if tdist < 5000 * 5000 && point_inside_poly(x as i32, y as i32, &polys[i]) != 0 {
            closest = i;
            break;
        }

        if tdist < closestdist {
            closest = i;
            closestdist = tdist;
            closestdist = tdist;
        }
    }
    return closest;
}

pub fn find_closest_idx_0(polys: &Vec<PrePolyWithHeat>, x: i64, y: i64) -> usize {
    let mut closest = 0;
    let mut closestdist = _dist(&polys[0].0, x, y);
    for i in 0..polys.len() {
        let tdist = _dist(&polys[i].0, x, y);
        if tdist < 5000 * 5000 && point_inside_poly(x as i32, y as i32, &polys[i].0) != 0 {
            closest = i;
            break;
        }

        if tdist < closestdist {
            closest = i;
            closestdist = tdist;
            closestdist = tdist;
        }
    }
    return closest;
}



pub fn create_local_graph(
    map: &Map,
    from: (i64, i64),
    players: &Vec<(i32, i32, f32)>,
) -> Option<(Graph<usize, (u64, Line)>, Vec<PrePoly>, Vec<NodeIndex>)> {
    let mut pos_to_z = HashMap::<(i32, i32), f32>::new();
    let mut tiles = get_around_tiles(&map, from.0, from.1, 1260);
    let mut polys = vec![];
    if tiles.len() == 0 {
        return None;
    }

    // take tile's poly and z poses
    for i_t in tiles {
        for p in &map.polygons[i_t] {
            polys.push(p.clone());
        }

        for area in &map.tiles[i_t].areas {
            for v in &area.verts {
                pos_to_z.insert((v.0, v.1), v.2);
            }
        }
    }

    // make players as holes and keep player's Z pos *** players supposed are in radus 1200
    for player in players {
        if line_len((
            (from.0 as i32, from.1 as i32, 0.0 as f32),
            (player.0, player.1, 0.0 as f32),
        )) > 1200
        {
            continue;
        }

        let x1 = (player.0 - 30) as f64;
        let x2 = (player.0 + 30) as f64;
        let y1 = (player.1 - 30) as f64;
        let y2 = (player.1 + 30) as f64;

        pos_to_z.insert((x1 as i32, y1 as i32), player.2);
        pos_to_z.insert((x1 as i32, y2 as i32), player.2);
        pos_to_z.insert((x2 as i32, y2 as i32), player.2);
        pos_to_z.insert((x2 as i32, y1 as i32), player.2);

        let player_hole = Polygon::new(
            LineString(vec![
                Coordinate { x: x1, y: y1 },
                Coordinate { x: x1, y: y2 },
                Coordinate { x: x2, y: y2 },
                Coordinate { x: x2, y: y1 },
                Coordinate { x: x1, y: y1 },
            ]),
            vec![],
        );

        for i_poly in (0..polys.len()).rev() {
            let res = polys[i_poly].intersection(&player_hole, 1.);
            let mut intersections = 0;
            for r in res {
                intersections = intersections + 1;
            }
            if intersections == 0 {
                continue;
            }

            let res1 = polys[i_poly].difference(&player_hole, 1.);
            polys.remove(i_poly);
            for r in res1 {
                polys.push(r.clone());
            }
        }
    }

    // println!("made all holes");

    // split into triangles
    let mut poly_id: u64 = 1;
    let mut new_polys: Vec<PrePoly> = vec![];
    for new_p in &polys {
        let tris = algebra::polygon2tris(new_p);
        for tri in &tris{
            let mut ptts: Vec<(i32, i32, f32)> = vec![];
            ptts.push( ((tri.0).0 as i32, (tri.0).1 as i32, 0.) );
            ptts.push( ((tri.1).0 as i32, (tri.1).1 as i32, 0.));
            ptts.push( ((tri.2).0 as i32, (tri.2).1 as i32, 0.));
            for i in 0..3{
                let mut z: f32 = 0.0;
                match pos_to_z.get(&(ptts[i].0, ptts[i].1)) {
                    Some(z_) => {
                        z = *z_;
                    }
                    None => (),
                }
                ptts[i].2 = z;
            }

            let center = (
                (ptts[0].0 + ptts[1].0 + ptts[2].0) / 3,
                (ptts[0].1 + ptts[1].1 + ptts[2].1) / 3,
            );
            let n_p = PrePoly {
                vert_count: 3,
                verts: ptts,
                center: center,
                id: poly_id,
            };
            new_polys.insert(new_polys.len(), n_p);
            poly_id = poly_id + 1;
        }
    }

    // println!("splited all tris");
    // make graph
    let mut graph = Graph::<usize, (u64, Line)>::new();
    let mut links = Vec::new();
    let mut nodes = vec![];

    for i_poly in 0..new_polys.len() {
        nodes.push(graph.add_node(i_poly));
    }

    //make link
    for i_poly in 0..new_polys.len() {
        // for j_poly in 0..new_polys.len() {
        for j_poly in (i_poly + 1)..new_polys.len() {            
            if i_poly == j_poly {
                continue;
            }
            match area_intersect(&new_polys[i_poly], &new_polys[j_poly], false) {
                Some(edge) => {
                    let dx = new_polys[i_poly].center.0 - new_polys[j_poly].center.0;
                    let dy = new_polys[i_poly].center.1 - new_polys[j_poly].center.1;
                    let d = ((dx * dx + dy * dy) as f64).sqrt().round() as u64;
                    let edge_len = line_len(edge);
                    let weight_mul = match edge_len {
                        0..=60 => 1000.,
                        61..=100 => 100.,
                        101..=200 => 10.1,
                        0..=100 => 3.2,
                        101..=200 => 2.1,
                        _ => 1.0,
                    };
                    let weight = ((d as f64) * weight_mul).trunc() as u64;
                    links.push((nodes[i_poly], nodes[j_poly], (weight, edge)));
                    links.push((nodes[j_poly], nodes[i_poly], (weight, edge)));
                }
                None => (),
            }
        }
    }
    graph.extend_with_edges(links);
    return Some((graph, new_polys, nodes));
}

pub fn create_heatmap_graph(
    map: &Map,
    from: (i64, i64),
    players: &Vec<((i32, i32, f32), i32)>, 
    foes: &Vec<((i32, i32, f32), i32)>,
) -> Option<(Graph<usize, (f64, Line)>, Vec<PrePolyWithHeat>, Vec<NodeIndex>)> {
    let mut pos_to_z = HashMap::<(i32, i32), f32>::new();
    let mut tiles = get_around_tiles(&map, from.0, from.1, 2520);
    let mut polys = vec![];
    if tiles.len() == 0 {
        return None;
    }

    // take tile's poly and z poses
    for i_t in tiles {
        for p in &map.polygons[i_t] {
            polys.push(p.clone());
        }

        for area in &map.tiles[i_t].areas {
            for v in &area.verts {
                pos_to_z.insert((v.0, v.1), v.2);
            }
        }
    }

    for f in players{
        pos_to_z.insert(((f.0).0, (f.0).1), (f.0).2);
        pos_to_z.insert(((f.0).0 - f.1, (f.0).1 - f.1), (f.0).2);
        pos_to_z.insert(((f.0).0 - f.1, (f.0).1 + f.1), (f.0).2);
        pos_to_z.insert(((f.0).0 + f.1, (f.0).1 + f.1), (f.0).2);
        pos_to_z.insert(((f.0).0 + f.1, (f.0).1 - f.1), (f.0).2);
    }
    for f in foes{
        pos_to_z.insert(((f.0).0 as i32, (f.0).1 as i32), (f.0).2);
        pos_to_z.insert(((f.0).0 - f.1, (f.0).1 - f.1), (f.0).2);
        pos_to_z.insert(((f.0).0 - f.1, (f.0).1 + f.1), (f.0).2);
        pos_to_z.insert(((f.0).0 + f.1, (f.0).1 + f.1), (f.0).2);
        pos_to_z.insert(((f.0).0 + f.1, (f.0).1 - f.1), (f.0).2);
    }

    let heated_polys = heat_map::generate_heat_map_layout(&polys, players, foes);

    // split into triangles
    let mut poly_id: u64 = 1;
    let mut new_polys: Vec<PrePolyWithHeat> = vec![];
    for new_p in &heated_polys {
        let tris = algebra::polygon2tris(&new_p.0);
        for tri in &tris{
            let mut ptts: Vec<(i32, i32, f32)> = vec![];
            ptts.push( ((tri.0).0 as i32, (tri.0).1 as i32, 0.) );
            ptts.push( ((tri.1).0 as i32, (tri.1).1 as i32, 0.));
            ptts.push( ((tri.2).0 as i32, (tri.2).1 as i32, 0.));
            for i in 0..3{
                let mut z: f32 = 0.0;
                match pos_to_z.get(&(ptts[i].0, ptts[i].1)) {
                    Some(z_) => {
                        z = *z_;
                    }
                    None => (),
                }
                ptts[i].2 = z;
            }

            let center = (
                (ptts[0].0 + ptts[1].0 + ptts[2].0) / 3,
                (ptts[0].1 + ptts[1].1 + ptts[2].1) / 3,
            );
            let n_p = (PrePoly{
                vert_count: 3,
                verts: ptts,
                center: center,
                id: poly_id
            }, new_p.1);
            new_polys.insert(new_polys.len(), n_p);
            poly_id = poly_id + 1;
        }
    }

    // println!("splited all tris");
    // make graph
    let mut graph = Graph::<usize, (f64, Line)>::new();
    let mut links = Vec::new();
    let mut nodes = vec![];

    for i_poly in 0..new_polys.len() {
        nodes.push(graph.add_node(i_poly));
    }

    //make link
    for i_poly in 0..new_polys.len() {
        for j_poly in 0..new_polys.len() {
            if i_poly == j_poly {
                continue;
            }
            match area_intersect(&new_polys[i_poly].0, &new_polys[j_poly].0, false) {
                Some(edge) => {
                    let dx = (new_polys[i_poly].0).center.0 - (new_polys[j_poly].0).center.0;
                    let dy = (new_polys[i_poly].0).center.1 - (new_polys[j_poly].0).center.1;
                    let d = ((dx * dx + dy * dy) as f64).sqrt().round() as u64;

                    let danger = new_polys[i_poly].1 + new_polys[j_poly].1;
                    let mut weight_mul: f64 = 1.;
                    let mut multiple: f64 = 100.0;
                    if danger > 0 {multiple = 0.01;}
                    for i in 0..i32::abs(danger){
                        weight_mul = weight_mul * multiple;
                    }
                    // let edge_len = line_len(edge);
                    // let weight_mul = match edge_len {
                    //     0..=60 => 1000.,
                    //     61..=100 => 100.,
                    //     101..=200 => 10.1,
                    //     0..=100 => 3.2,
                    //     101..=200 => 2.1,
                    //     _ => 1.0,
                    // };
                    let weight = (d as f64) * weight_mul;
                    links.push((nodes[i_poly], nodes[j_poly], (weight, edge)));
                }
                None => (),
            }
        }
    }
    graph.extend_with_edges(links);
    return Some((graph, new_polys, nodes));
}

