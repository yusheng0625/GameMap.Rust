//use petgraph::Graph;
use petgraph::algo::astar;
//use petgraph::graph::NodeIndex;

use lazy_static::lazy_static;

use std::{fs, io};

//use geo::Polygon;

use std::time::Instant;

use std::cell::Cell;

use std::convert::TryInto;

use std::collections::HashMap;

mod mesh_geo;

mod waypoints;

lazy_static! {
    static ref MAP_1: mesh_geo::Map = {
        let entries = fs::read_dir("../../../assets/mapdata/worldmap/")
            .unwrap()
            .map(|res| res.unwrap().path().to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        return mesh_geo::files_to_map(entries, "mapdata/map1.bin");
    };
}

fn _get_map(_map_id: i64) -> &'static mesh_geo::Map {
    //match mapId {
    //    1 =>
    return &MAP_1;
}

fn _get_z(_map_id: i64, _x: i64, _y: i64) -> i64 {
    return 0;
}

fn _get_polygon(_map_id: i64, _x: i64, _y: i64) -> i64 {
    return 0;
}

fn _find_path(_map_id: i64, _x: i64, _y: i64, _x1: i64, _y1: i64) -> i64 {
    return 0;
}

//funnel

fn heur_dist((tox, toy): &(i64, i64), (nx, ny): &(i64, i64)) -> u64 {
    let dx = tox - nx;
    let dy = toy - ny;
    return (dx * dx + dy * dy) as u64;
}

fn get_tile(_map: &mesh_geo::Map, x: i64, y: i64) -> Option<&mesh_geo::PreTile> {
    let (bmin, _) = &MAP_1.bounds;

    let col = (x - bmin.x) / 1260;
    let row = (y - bmin.y) / 1260;

    let index: usize = (col + row * &MAP_1.tiles_cols).try_into().unwrap();

    let tile = &MAP_1.grid[index];

    return Some(tile);
}

fn main() -> io::Result<()> {
    // Visit all the files extracted by UE Viewer and print out any waypoint data found inside them.
    println!(
        "{:#?}",
        waypoints::visit_dirs(
            std::path::Path::new("/l2m/ex/data/waypoints/"),
            &waypoints::process_asset,
            &mut Vec::new()
        )
    );

    println!("{:?}", MAP_1.graph.edge_count());
    let source = (-306460, 223400);
    let dest: (i64, i64) = (-303880, 222860);

    let areas1: Vec<Vec<(i32, i32, f32)>> = get_tile(&MAP_1, -303660, 225540)
        .unwrap()
        .areas
        .iter()
        .map(|a| a.verts.clone())
        .collect();
    println!("{:?}", areas1);
    let areas2: Vec<Vec<(i32, i32, f32)>> = get_tile(&MAP_1, -303660, 225540)
        .unwrap()
        .areas
        .iter()
        .map(|a| a.verts.clone())
        .collect();
    println!("{:?}", areas2);

    let g1 = mesh_geo::find_closest_node(&MAP_1.graph, source.0, source.1);
    let g2 = mesh_geo::find_closest_node(&MAP_1.graph, dest.0, dest.1);

    thread_local!(static EXPANDED: Cell<u64> = Cell::new(0));

    let mut name_to_node = HashMap::<u64, &mesh_geo::PrePoly>::new();

    for tile in &MAP_1.tiles {
        for area in &tile.areas {
            name_to_node.insert(area.id, &area);
        }
    }

    let _now = Instant::now();

    let path = astar(
        &MAP_1.graph,
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

            let (_id, p) = &MAP_1.graph[e];
            return heur_dist(&dest, &(p.center.0 as i64, p.center.1 as i64));
        },
    );

    //println!("search elapsed: {}", now.elapsed().as_micros());

    let (_weight, p1) = path.or(Some((0, [].to_vec()))).unwrap();
    println!("path len: {:?}", p1.len());
    if p1.len() > 0 {
        let mut prevnode = p1[0];
        let mut edges = Vec::new();
        for node in p1 {
            if prevnode != node {
                for edge in MAP_1.graph.edges_connecting(prevnode, node) {
                    let (_d, l) = edge.weight();
                    edges.push(l);
                }
                //println!("e {:?}", edge.weight());
            }
            prevnode = node;
        }
        println!("e {:?}", edges);
    }

    EXPANDED.with(|thread_id| {
        let id = thread_id.get();
        println!("expanded nodes: {:?}", id);
    });

    println!("{:?}", g1);
    println!("{:?}", g2);

    println!("node {:?}", &MAP_1.graph[g1]);

    {
        let (node, _) = &MAP_1.graph[g1];

        for (n1, n2, l) in &MAP_1.links {
            if (*n1 == *node) | (*n2 == *node) {
                println!("{} {} {:?}", n1, n2, l);
            }
        }
    }

    println!("node {:?}", &MAP_1.graph[g2]);
    {
        let (node, _) = &MAP_1.graph[g2];

        for (n1, n2, l) in &MAP_1.links {
            if (*n1 == *node) | (*n2 == *node) {
                println!("{} {} {:?}", n1, n2, l);
            }
        }
    }

    {
        let (node, _) = &MAP_1.graph[g2];

        for edge in MAP_1.graph.edge_indices().into_iter() {
            let res = &MAP_1.graph.edge_endpoints(edge);
            match res {
                Some((a, b)) => {
                    let (n1, _) = &MAP_1.graph[*a];
                    let (n2, _) = &MAP_1.graph[*b];

                    if (*n1 == *node) | (*n2 == *node) {
                        println!("{} {}", n1, n2);
                    }
                    //  println!("{:?} {:?}", a, b),
                }
                None => (),
            }
        }
    }

    //use the grid to find the closest poly

    //let filename : &str = "../ex/data/mapdata/Elven_Ruins_NavTile.uexp.json";

    // let now = Instant::now();

    // let mut f = fs::File::open("mapdata/map1.bin".to_string()).unwrap();
    // let sm : mesh_geo::SavedMap = bincode::deserialize_from(f).unwrap();

    // println!("elapsed: {}", now.elapsed().as_millis());

    // println!("{:?}", (*MAP_92).graph.node_count());
    // println!("{:?}", (*MAP_92).graph.edge_count());
    // println!("{:?}", find_closest_node(&MAP_92.graph,-200000, 200000));
    // println!("{:?}", find_closest_node(&MAP_92.graph,-300000, 200000));

    Ok(())
}
