use petgraph::algo::astar;
use petgraph::Graph;
use rustler::{Encoder, Env, Error, Term};
//use rustler::types::ListIterator;
//use petgraph::graph::NodeIndex;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;

use std::cell::Cell;
use std::time::Instant;

use geo_clipper::Clipper;
use geo_types::{Coordinate, LineString, Polygon};
use rustler::schedule::SchedulerFlags;
use serde::{Deserialize, Serialize};

mod funnel;
mod mesh_geo;
#[path = "algebra.rs"] mod algebra;

lazy_static! {
    //Level 40 quest
    static ref MAP_91: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Elven_Ruins_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    //Tutorial
    static ref MAP_92: mesh_geo::Map = {

        let entries = vec!["../assets/mapdata/dungeon/Elven_Ruins_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };

    static ref MAP_1: mesh_geo::Map = {
        let entries = fs::read_dir("../assets/mapdata/worldmap/")
            .unwrap()
            .map(|res| res.unwrap().path().to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };

    //Noob forlore temple
    static ref MAP_11: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Forgotten_Temple_B1_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };

    //Ant Cave
    static ref MAP_21: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Ant_Nest_B1_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_22: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Ant_Nest_B2_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_23: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Ant_Nest_B3_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };

    //Dungeons
    //Cruma
    static ref MAP_1031: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B1_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1032: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B2_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1033: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B3_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1034: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B4_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1035: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B5_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1036: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B6_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1037: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Tower_of_Cruma_B7_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    //Level 40
    static ref MAP_2001: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Battle_Island_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_2002: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Destroyed_Castle_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    //Level 45
    static ref MAP_1041: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Nest_of_Antaras_B1_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1042: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Nest_of_Antaras_B2_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1043: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Nest_of_Antaras_B3_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1044: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Nest_of_Antaras_B4_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1045: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Nest_of_Antaras_B5_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    static ref MAP_1046: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Nest_of_Antaras_B6_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
    //Event
    static ref MAP_5023: mesh_geo::Map = {
        let entries = vec!["../assets/mapdata/dungeon/Christmas_Island_NavTile.uexp".to_string()];
        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };

}

fn map_by_map_id(map_id: i64) -> &'static mesh_geo::Map {
    let map: &mesh_geo::Map = match map_id {
        1 => &MAP_1,
        91 => &MAP_91,
        92 => &MAP_92,
        11 => &MAP_11,
        21 => &MAP_21,
        22 => &MAP_22,
        23 => &MAP_23,
        1031 => &MAP_1031,
        1032 => &MAP_1032,
        1033 => &MAP_1033,
        1034 => &MAP_1034,
        1035 => &MAP_1035,
        1036 => &MAP_1036,
        1037 => &MAP_1037,
        2001 => &MAP_2001,
        2002 => &MAP_2002,
        1041 => &MAP_1041,
        1042 => &MAP_1042,
        1043 => &MAP_1043,
        1044 => &MAP_1044,
        1045 => &MAP_1045,
        1046 => &MAP_1046,
        5023 => &MAP_5023,
        _ => &MAP_1046,
    };
    return map;
}

mod atoms {
    rustler::rustler_atoms! {
        atom ok;
        atom error;
        atom same_polygon;
        //atom __true__ = "true";
        //atom __false__ = "false";
    }
}

rustler::rustler_export_nifs! {
    "Elixir.GameMap",
    [
        ("path", 3, path, SchedulerFlags::DirtyCpu),
        ("path_near", 3, path_near, SchedulerFlags::DirtyCpu),
        ("can_walk_direct", 3, is_can_walk_direct, SchedulerFlags::DirtyCpu),
        ("path_local", 4, path_local, SchedulerFlags::DirtyCpu),
        ("is_walkable", 2, is_walkable, SchedulerFlags::DirtyCpu),                
        ("around_boxes", 3, around_boxes, SchedulerFlags::DirtyCpu),
        ("heat_maps", 4, heat_maps, SchedulerFlags::DirtyCpu),        
        ("path_heatmap", 5, path_heatmap, SchedulerFlags::DirtyCpu)
    ],
    None
}

fn calc_path(
    map: &mesh_geo::Map,
    from: (i64, i64),
    to: (i64, i64),
) -> Option<Vec<(i64, i64, f32)>> {
    let g1 = mesh_geo::find_closest_node(&map.graph, from.0, from.1);
    let g2 = mesh_geo::find_closest_node(&map.graph, to.0, to.1);

    if g1.index() == g2.index() {
        return Some(vec![]);
    }

    thread_local!(static EXPANDED: Cell<u64> = Cell::new(0));
    let path = astar(
        &map.graph,
        g1,
        |finish| finish.index() == g2.index(),
        |e| {
            let (dist, _) = *e.weight();
            return dist;
        },
        |e| {
            EXPANDED.with(|thread_id| {
                let id = thread_id.get();
                thread_id.set(id + 1);
            });
            let (_id, p) = &map.graph[e];
            return heur_dist(&to, &(p.center.0 as i64, p.center.1 as i64));
        },
    );

    let (_weight, p1) = path.or(Some((0, [].to_vec()))).unwrap();
    if p1.len() == 0 {
        return None;
    }

    let mut prevnode = p1[0];
    let mut edges = Vec::new();
    for node in p1 {
        if prevnode != node {
            for edge in map.graph.edges_connecting(prevnode, node) {
                let (_d, l) = edge.weight();
                edges.push(l);
            }
        }
        prevnode = node;
    }

    let edgs: Vec<((i64, i64, f32), (i64, i64, f32))> = edges
        .iter()
        .map(|x| {
            let (l1, l2) = x;
            return (
                (l1.0 as i64, l1.1 as i64, l1.2),
                (l2.0 as i64, l2.1 as i64, l2.2),
            );
        })
        .collect();
    let to_z = get_z_from_poly(to.0 as i32, to.1 as i32, &map.graph[g2].1);
    let re = funnel::string_pull((from.0, from.1, 0.0), (to.0, to.1, to_z), edgs);
    return Some(re);
}

fn path<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let path_time_start = Instant::now();

    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let to: (i64, i64) = args[2].decode::<(i64, i64)>()?;

    let map: &mesh_geo::Map = map_by_map_id(map_id);

    match calc_path(map, from, to) {
        Some(res) => {
            if res.len() == 0 {
                return Ok(atoms::same_polygon().encode(env));
            } else {
                let path_took = path_time_start.elapsed().as_micros();
                return Ok((atoms::ok(), path_took as u64, res).encode(env));
            }
        }
        None => (),
    }
    return Ok((atoms::error(), path_time_start.elapsed().as_micros() as u64).encode(env));
}

fn is_can_walk_direct<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let path_time_start = Instant::now();

    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let to: (i64, i64) = args[2].decode::<(i64, i64)>()?;

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    let dist = heur_dist(&from, &to);
    if dist < 2000 && can_walk_direct(map, from, to) {
        return Ok((atoms::ok(), 0 as u64).encode(env));
    }
    return Ok((atoms::error(), 0 as u64).encode(env));
}

fn path_near<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let path_time_start = Instant::now();

    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let to: (i64, i64) = args[2].decode::<(i64, i64)>()?;

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    let dist = heur_dist(&from, &to);
    if dist < 2000 && can_walk_direct(map, from, to) {
        return Ok((atoms::ok(), 0 as u64, vec![(to.0, to.1, 0.0)]).encode(env));
    }

    let g1 = mesh_geo::find_closest_node(&map.graph, from.0, from.1);
    let g2 = mesh_geo::find_closest_node(&map.graph, to.0, to.1);

    if g1.index() == g2.index() {
        return Ok(atoms::same_polygon().encode(env));
    }

    thread_local!(static EXPANDED: Cell<u64> = Cell::new(0));

    let _now = Instant::now();

    let path = astar(
        &map.graph,
        g1,
        |finish| finish.index() == g2.index(),
        |e| {
            let (dist, _) = *e.weight();
            return dist;
        },
        |e| {
            EXPANDED.with(|thread_id| {
                let id = thread_id.get();
                thread_id.set(id + 1);
            });
            let (_id, p) = &map.graph[e];
            return heur_dist(&to, &(p.center.0 as i64, p.center.1 as i64));
        },
    );

    //println!("search elapsed: {}us", now.elapsed().as_micros());
    //let search_took = now.elapsed().as_micros();
    let _now = Instant::now();

    let (_weight, p1) = path.or(Some((0, [].to_vec()))).unwrap();
    if p1.len() == 0 {
        return Ok((atoms::error(), path_time_start.elapsed().as_micros() as u64).encode(env));
    }

    let mut prevnode = p1[0];
    let mut edges = Vec::new();
    for node in p1 {
        if prevnode != node {
            for edge in map.graph.edges_connecting(prevnode, node) {
                let (_d, l) = edge.weight();
                edges.push(l);
            }
            //println!("e {:?}", edge.weight());
        }
        prevnode = node;
    }

    let edgs: Vec<((i64, i64, f32), (i64, i64, f32))> = edges
        .iter()
        .map(|x| {
            let (l1, l2) = x;
            return (
                (l1.0 as i64, l1.1 as i64, l1.2),
                (l2.0 as i64, l2.1 as i64, l2.2),
            );
        })
        .collect();
    let re = funnel::string_pull((from.0, from.1, 0.0), (to.0, to.1, 0.0), edgs);

    let path_took = path_time_start.elapsed().as_micros();
    Ok((atoms::ok(), path_took as u64, re).encode(env))
}

fn heur_dist((tox, toy): &(i64, i64), (nx, n_y): &(i64, i64)) -> u64 {
    let dx = tox - nx;
    let dy = toy - n_y;
    return ((dx * dx + dy * dy) as f32).sqrt().trunc() as u64;
}

fn get_z_from_poly(x: i32, y: i32, poly: &mesh_geo::PrePoly) -> f32 {
    let v1 = (
        poly.verts[0].0 as f64,
        poly.verts[0].1 as f64,
        poly.verts[0].2 as f64,
    );
    let v2 = (
        poly.verts[1].0 as f64,
        poly.verts[1].1 as f64,
        poly.verts[1].2 as f64,
    );
    let v3 = (
        poly.verts[2].0 as f64,
        poly.verts[2].1 as f64,
        poly.verts[2].2 as f64,
    );

    let a = v1.1 * (v2.2 - v3.2) + v2.1 * (v3.2 - v1.2) + v3.1 * (v1.2 - v2.2);
    let b = v1.2 * (v2.0 - v3.0) + v2.2 * (v3.0 - v1.0) + v3.2 * (v1.0 - v2.0);
    let c = v1.0 * (v2.1 - v3.1) + v2.0 * (v3.1 - v1.1) + v3.0 * (v1.1 - v2.1);
    let d = v1.0 * (v2.1 * v3.2 - v3.1 * v2.2)
        + v2.0 * (v3.1 * v1.2 - v1.1 * v3.2)
        + v3.0 * (v1.1 * v2.2 - v2.1 * v1.2);

    let z = (d - (a * x as f64) - (b * y as f64)) / c;
    return z as f32;
}

pub fn can_walk_direct(map: &mesh_geo::Map, from: (i64, i64), to: (i64, i64)) -> bool {
    let mut tiles = mesh_geo::get_around_tiles(&map, from.0, from.1, 2400);
    let mut polys = vec![];

    if tiles.len() == 0 {
        return false;
    }
    // make polygons
    for i_t in tiles {
        for p in &map.polygons[i_t] {
            polys.push(p.clone());
        }
    }

    // println!("can_walk_direct1={:?}", polys.len());

    //merge polys
    let mut merged_poly = true;
    while merged_poly == true {
        merged_poly = false;
        for i_poly in 0..polys.len() - 1 {
            for j_poly in (i_poly + 1)..polys.len() {
                match mesh_geo::merge_two_polygon(&polys[i_poly], &polys[j_poly]) {
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

    // check if line is intersect with polys lines
    let a = (from.0 as f64, from.1 as f64);
    let b = (to.0 as f64, to.1 as f64);
    let mut can_walk = true;
    for p in polys {
        let mut points = mesh_geo::dedup_coordinates(p.exterior().clone().into_iter().collect());
        points.push(Coordinate {
            x: points[0].x,
            y: points[0].y,
        });
        for i in 0..(points.len() - 1) {
            let c = (points[i].x, points[i].y);
            let d = (points[i + 1].x, points[i + 1].y);
            if algebra::segment_intersects(a, b, c, d) > 0 {
                can_walk = false;
                break;
            }
        }
        if can_walk == false {
            break;
        }

        for line in p.interiors() {
            let mut pointts = mesh_geo::dedup_coordinates(line.clone().into_iter().collect());
            pointts.push(Coordinate {
                x: pointts[0].x,
                y: pointts[0].y,
            });
            for i in 0..(pointts.len() - 1) {
                let c = (pointts[i].x, pointts[i].y);
                let d = (pointts[i + 1].x, pointts[i + 1].y);
                if algebra::segment_intersects(a, b, c, d) > 0 {
                    can_walk = false;
                    break;
                }
            }
            if can_walk == false {
                break;
            }
        }
        if can_walk == false {
            break;
        }
    }

    // check if line is in polys
    // let line = LineString(vec![Coordinate{x: from.0 as f64, y: from.1 as f64}, Coordinate{x: to.0 as f64, y: to.1 as f64}]);
    // let mut can_walk = false;
    // for p in polys{
    //     can_walk = p.contains(&line);
    //     if can_walk {break;}
    // }
    return can_walk;
}

fn around_boxes<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let players: Vec<(i32, i32, f32)> = args[2].decode::<Vec<(i32, i32, f32)>>()?;

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    match mesh_geo::create_local_graph(map, from, &players) {
        Some(res) => {
            let (_, polys, _) = res;
            let mut re = vec![];
            for p in polys {
                re.push(p.verts);
                // let verts = p.verts.iter().map(|v|{
                //     return ()
                // }).collect();
            }
            return Ok((atoms::ok(), 1 as u64, re).encode(env));
        }
        None => (),
    }
    return Ok((atoms::error(), 1 as u64).encode(env));
}

fn path_local<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let to: (i64, i64) = args[2].decode::<(i64, i64)>()?;
    let mut players: Vec<(i32, i32, f32)> = args[3].decode::<Vec<(i32, i32, f32)>>()?;
    let path_time_start = Instant::now();

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    let mut normal_path = match calc_path(map, from, to) {
        Some(res) => res,
        None => {
            return Ok((atoms::error(), path_time_start.elapsed().as_micros() as u64).encode(env));
        }
    };

    players.retain(|x| {
        return mesh_geo::line_len((
            (from.0 as i32, from.1 as i32, 0.0 as f32),
            (x.0, x.1, 0.0 as f32),
        )) <= 1200;
    });

    if players.len() == 0 {
        return Ok((
            atoms::ok(),
            path_time_start.elapsed().as_micros() as u64,
            normal_path,
        )
            .encode(env));
    }

    // find to position
    let mut to_center = (to.0, to.1, 0. as f32);
    let mut to_center_idx = -1;
    for i in 0..normal_path.len() {
        if heur_dist(&from, &(normal_path[i].0 as i64, normal_path[i].1 as i64)) >= 1260 {
            to_center_idx = i as i32;
            to_center = (
                normal_path[i].0 as i64,
                normal_path[i].1 as i64,
                normal_path[i].2,
            );
            break;
        }
    }

    //remove path to_center
    if to_center_idx >= 0 {
        for i in 0..(to_center_idx + 1) {
            normal_path.remove(0);
        }
    }

    let mut edges = Vec::new();
    match mesh_geo::create_local_graph(map, from, &players) {
        Some(res) => {
            let (graph, polys, nodes) = res;
            let from_idx = mesh_geo::find_closest_idx(&polys, from.0, from.1);
            let to_idx = mesh_geo::find_closest_idx(&polys, to_center.0, to_center.1);

            let path = astar(
                &graph,
                nodes[from_idx],
                |finish| finish.index() == nodes[to_idx].index(),
                |e| {
                    let (dist, _) = *e.weight();
                    return dist;
                },
                |e| 0,
            );

            let (_weight, p1) = path.or(Some((0, [].to_vec()))).unwrap();
            if p1.len() == 0 {
                return Ok(
                    (atoms::error(), path_time_start.elapsed().as_micros() as u64).encode(env),
                );
            }
            let mut prevnode = p1[0];
            for node in p1 {
                if prevnode != node {
                    for edge in graph.edges_connecting(prevnode, node) {
                        let (_d, l) = edge.weight();
                        let (l0, l1) = l;
                        edges.push(((l0.0, l0.1, l0.2), (l1.0, l1.1, l1.2)));
                    }
                }
                prevnode = node;
            }
        }
        None => {
            return Ok((atoms::error(), 0 as u64).encode(env));
        }
    }

    let edgs: Vec<((i64, i64, f32), (i64, i64, f32))> = edges
        .iter()
        .map(|x| {
            let (l1, l2) = x;
            return (
                (l1.0 as i64, l1.1 as i64, l1.2),
                (l2.0 as i64, l2.1 as i64, l2.2),
            );
        })
        .collect();

    let mut re = funnel::string_pull(
        (from.0, from.1, 0.0),
        (to_center.0, to_center.1, to_center.2),
        edgs,
    );

    if re[0].0 == from.0 && re[0].1 == from.1 {
        re.remove(0);
    }

    for n in normal_path {
        re.push(n);
    }
    return Ok((
        atoms::ok(),
        path_time_start.elapsed().as_micros() as u64,
        re,
    )
        .encode(env));
}

fn is_walkable<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    let g1 = mesh_geo::find_closest_node(&map.graph, from.0, from.1);    

    if mesh_geo::point_inside_poly(from.0 as i32, from.1 as i32, &map.graph[g1].1) == 0{
        return Ok(
            (atoms::error(), 1 as u64).encode(env)
        );
    }
    return Ok(
        (atoms::ok(), 1 as u64).encode(env)
    );
}

fn heat_maps<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let players: Vec<((i32, i32, f32), i32)> = args[2].decode::<Vec<((i32, i32, f32), i32)>>()?;
    let foes: Vec<((i32, i32, f32), i32)> = args[3].decode::<Vec<((i32, i32, f32), i32)>>()?;    

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    match mesh_geo::create_heatmap_graph(map, from, &players, &foes) {
        Some(res) => {
            let (_, polys, _) = res;
            let mut re = vec![];
            for p in polys {
                re.push((p.0.verts, p.1));
                // let verts = p.verts.iter().map(|v|{
                //     return ()
                // }).collect();
            }
            return Ok((atoms::ok(), 1 as u64, re).encode(env));
        }
        None => (),
    }
    return Ok((atoms::error(), 1 as u64).encode(env));
}

fn path_heatmap<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let map_id: i64 = args[0].decode::<i64>()?;
    let from: (i64, i64) = args[1].decode::<(i64, i64)>()?;
    let to: (i64, i64) = args[2].decode::<(i64, i64)>()?;
    let mut players: Vec<((i32, i32, f32), i32)> = args[3].decode::<Vec<((i32, i32, f32), i32)>>()?;
    let mut foes: Vec<((i32, i32, f32), i32)> = args[4].decode::<Vec<((i32, i32, f32), i32)>>()?;
    let path_time_start = Instant::now();

    let map: &mesh_geo::Map = map_by_map_id(map_id);
    let mut normal_path = match calc_path(map, from, to) {
        Some(res) => res,
        None => {
            return Ok((atoms::error(), path_time_start.elapsed().as_micros() as u64).encode(env));
        }
    };

    players.retain(|x| {
        return mesh_geo::line_len((
            (from.0 as i32, from.1 as i32, 0.0 as f32),
            ((x.0).0, (x.0).1, 0.0 as f32),
        )) <= 2000;
    });

    foes.retain(|x| {
        return mesh_geo::line_len((
            (from.0 as i32, from.1 as i32, 0.0 as f32),
            ((x.0).0, (x.0).1, 0.0 as f32),
        )) <= 2000;
    });


    if players.len() == 0 && foes.len() ==0{
        return Ok((
            atoms::ok(),
            path_time_start.elapsed().as_micros() as u64,
            normal_path,
        )
            .encode(env));
    }

    // find to position
    let mut to_center = (to.0, to.1, 0. as f32);
    let mut to_center_idx = -1;
    for i in 0..normal_path.len() {
        if heur_dist(&from, &(normal_path[i].0 as i64, normal_path[i].1 as i64)) >= 1260 {
            to_center_idx = i as i32;
            to_center = (
                normal_path[i].0 as i64,
                normal_path[i].1 as i64,
                normal_path[i].2,
            );
            break;
        }
    }

    //remove path to_center
    if to_center_idx >= 0 {
        for i in 0..(to_center_idx + 1) {
            normal_path.remove(0);
        }
    }

    let mut edges = Vec::new();
    match mesh_geo::create_heatmap_graph(map, from, &players, &foes) {
        Some(res) => {
            let (graph, polys, nodes) = res;
            let from_idx = mesh_geo::find_closest_idx_0(&polys, from.0, from.1);
            let to_idx = mesh_geo::find_closest_idx_0(&polys, to_center.0, to_center.1);

            let path = astar(
                &graph,
                nodes[from_idx],
                |finish| finish.index() == nodes[to_idx].index(),
                |e| {
                    let (dist, _) = *e.weight();
                    return dist;
                },
                |e| 0.0,
            );

            let (_weight, p1) = path.or(Some((0.0, [].to_vec()))).unwrap();
            if p1.len() == 0 {
                return Ok(
                    (atoms::error(), path_time_start.elapsed().as_micros() as u64).encode(env),
                );
            }
            let mut prevnode = p1[0];
            for node in p1 {
                if prevnode != node {
                    for edge in graph.edges_connecting(prevnode, node) {
                        let (_d, l) = edge.weight();
                        let (l0, l1) = l;
                        edges.push(((l0.0, l0.1, l0.2), (l1.0, l1.1, l1.2)));
                    }
                }
                prevnode = node;
            }
        }
        None => {
            return Ok((atoms::error(), 0 as u64).encode(env));
        }
    }

    let edgs: Vec<((i64, i64, f32), (i64, i64, f32))> = edges
        .iter()
        .map(|x| {
            let (l1, l2) = x;
            return (
                (l1.0 as i64, l1.1 as i64, l1.2),
                (l2.0 as i64, l2.1 as i64, l2.2),
            );
        })
        .collect();

    let mut re = funnel::string_pull(
        (from.0, from.1, 0.0),
        (to_center.0, to_center.1, to_center.2),
        edgs,
    );

    if re[0].0 == from.0 && re[0].1 == from.1 {
        re.remove(0);
    }

    for n in normal_path {
        re.push(n);
    }
    return Ok((
        atoms::ok(),
        path_time_start.elapsed().as_micros() as u64,
        re,
    )
        .encode(env));
}

