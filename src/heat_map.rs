use geo_types::{Coordinate, LineString, Polygon, MultiPolygon};
use geo_clipper::Clipper;
use std::collections::HashMap;

#[path = "algebra.rs"] mod algebra;


pub fn generate_heated_map(friends: &Vec<((i32, i32, f32), i32)>, foes: &Vec<((i32, i32, f32), i32)>) -> Vec<(Polygon<f64>, i32)>{

    let mut maps: Vec<(Polygon<f64>, i32)> = vec![];
    for f in friends{
        let x1 = ((f.0).0 - f.1) as f64;
        let x2 = ((f.0).0 + f.1) as f64;        
        let y1 = ((f.0).1 - f.1) as f64;
        let y2 = ((f.0).1 + f.1) as f64;

        let poly = Polygon::new(LineString(vec![Coordinate{x: x1, y: y1}, 
                                                Coordinate{x: x1, y: y2},
                                                Coordinate{x: x2, y: y2},
                                                Coordinate{x: x2, y: y1},
                                                Coordinate{x: x1, y: y1}]), vec![]);
        maps.push((poly, 1));
    }

    for f in foes{
        let x1 = ((f.0).0 - f.1) as f64;
        let x2 = ((f.0).0 + f.1) as f64;        
        let y1 = ((f.0).1 - f.1) as f64;
        let y2 = ((f.0).1 + f.1) as f64;

        let poly = Polygon::new(LineString(vec![Coordinate{x: x1, y: y1}, 
                                                Coordinate{x: x1, y: y2},
                                                Coordinate{x: x2, y: y2},
                                                Coordinate{x: x2, y: y1},
                                                Coordinate{x: x1, y: y1}]), vec![]);
        maps.push((poly, -1));
    }

    // merge all
    let mut processed = true;
    while processed==true{
        processed = false;

        for i_poly in 0..maps.len()-1{
            for j_poly in (i_poly+1)..maps.len(){

                // check if intersect
                let res = (maps[i_poly].0).intersection(&(maps[j_poly].0), 1.);                

                let mut intersections = 0;
                for r in res{
                    intersections = intersections + 1;
                    maps.push((r.clone(), maps[i_poly].1 + maps[j_poly].1));   

                    let res1 = (maps[i_poly].0).difference(&r, 1.);
                    for rr in res1{
                        maps.push((rr.clone(), maps[i_poly].1));
                    }

                    let res2 = (maps[j_poly].0).difference(&r, 1.);
                    for rr in res2{
                        maps.push((rr.clone(), maps[j_poly].1));
                    }
                }

                if intersections == 0{continue;}

                maps.remove(j_poly);
                maps.remove(i_poly);
                processed = true;
                break;
            }
            if processed==true{
                break;
            }
        }

    }

    maps.retain(|x| x.1!=0);
    return maps;
}

pub fn generate_heat_map_layout(
    layouts: &Vec<Polygon<f64>>, 
    friends: &Vec<((i32, i32, f32), i32)>, 
    foes: &Vec<((i32, i32, f32), i32)>
) -> Vec<(Polygon<f64>, i32)>{

    // generate heat map 
    let maps = generate_heated_map(friends, foes);
    let mut polys: Vec<(Polygon<f64>, i32)> = vec![];
    for p in layouts{
        polys.push((p.clone(), 0));        
    }

    let mut intersects_cnt = 0;
    for m in &maps{
        for i in (0..polys.len()).rev(){
            // check intersect
            let res = (polys[i].0).intersection(&m.0, 1.);
            for r in res{
                intersects_cnt = intersects_cnt + 1;
            }
            if intersects_cnt == 0 {continue;}

            let res1 = (polys[i].0).difference(&m.0, 1.);
            for r in res1{
                polys.push((r.clone(), polys[i].1));
            }            
            polys.remove(i);
        }
    }

    for m in &maps{
        polys.push((m.0.clone(), m.1));
    }
    return polys;
}
