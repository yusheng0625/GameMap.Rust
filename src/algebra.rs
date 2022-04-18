use geo_types::{Coordinate, LineString, MultiPolygon, Polygon};

fn dist(a: (f64, f64, f32), b: (f64, f64, f32)) -> f64 {
    let v = (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1);
    return v.sqrt();
}

fn vector_cross(a: (f64, f64), b: (f64, f64)) -> f64 {
    let z = a.0 * b.1 - b.0 * a.1;
    return z;
}
fn vector_minus(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    return (a.0 - b.0, a.1 - b.1);
}
fn vector_is_less(a: (f64, f64), b: (f64, f64)) -> bool {
    return a.0 <= b.0 && a.1 <= b.1;
}
fn ccw0(a: (f64, f64), b: (f64, f64)) -> f64 {
    return vector_cross(a, b);
}
//p-origin  if p-b is reverse clock dir of p-a => positive
//if p-b is clock dir of p-a => negotive
//if p-b is same dir of p-a => 0
fn ccw(a: (f64, f64), b: (f64, f64), p: (f64, f64)) -> f64 {
    return ccw0(vector_minus(a, p), vector_minus(b, p));
}
pub fn segment_intersects(
    aa: (f64, f64),
    bb: (f64, f64),
    cc: (f64, f64),
    dd: (f64, f64),
) -> i32 {
    let ab = ccw(aa, bb, cc) * ccw(aa, bb, dd);
    let cd = ccw(cc, dd, aa) * ccw(cc, dd, bb);

    if ab == 0.0 && cd == 0.0 {
        let a;
        let b;
        let c;
        let d;

        if vector_is_less(bb, aa) {
            a = bb;
            b = aa;
        } else {
            a = aa;
            b = bb;
        }

        if vector_is_less(dd, cc) {
            c = dd;
            d = cc;
        } else {
            c = cc;
            d = dd;
        }
        if !(vector_is_less(b, c) || vector_is_less(d, a)) {
            return -1;
        }
        return 0;
    }
    if ab <= 0.0 && cd <= 0.0 {
        return 1;
    }
    return 0;
}


pub fn pt_inside_line(x: f64, y: f64, line: &LineString<f64>) -> i32 {

    let mut intersections = 0;
    let mut f_f: f64;
    let mut dy: f64;
    let mut dy2: f64;
    let points: Vec<Coordinate<f64>> = line.clone().into_iter().collect();
    dy2 = y - points[0].y;

    for ii in 0..(points.len()-1) {
        dy = dy2;
        dy2 = y - points[ii+1].y;

        if (dy * dy2) <= 0. && (x >= points[ii].x || x >= points[ii+1].x) {
            // non-horizontal line
            if dy < 0. || dy2 < 0.
            // => dy*dy2 != 0
            {
                f_f = dy * (points[ii+1].x - points[ii].x) / (dy - dy2) + points[ii].x;

                if x > f_f
                // line is left from the point - the ray moving towards
                {
                    // left, will intersect it
                    intersections += 1;
                } else if x == f_f
                // point on line
                {
                    return -1;
                }
            }
            // # point on upper peak (dy2=dx2=0) or horizontal line (dy=dy2=0 and
            // dx*dx2<=0)
            else if dy2 == 0.
                && (x == points[ii+1].x
                    || (dy == 0. && (x - points[ii].x) * (x - points[ii+1].x) <= 0.))
            {
                return -1;
            }
        }
    }
    return intersections & 1;
}

pub fn point_inside_poly(point: (f64, f64), polygon : &Vec<(f64, f64)>) -> i32{
    let length = 3;
    let mut intersections = 0;
    let mut f_f: f64;
    let mut dy: f64;
    let mut dy2: f64;
    dy2 = point.1 - polygon[0].1;

    let mut jj: usize = 1;
    for ii in 0..length {
        dy = dy2;
        if jj == length {
            jj = 0;
        }

        dy2 = point.1 - polygon[jj].1;
        if (dy * dy2) <= 0. && (point.0 >= polygon[ii].0 || point.0 >= polygon[jj].0) {
            // non-horizontal line
            if dy < 0. || dy2 < 0.
            // => dy*dy2 != 0
            {
                f_f = dy * (polygon[jj].0 - polygon[ii].0) / (dy - dy2) + polygon[ii].0;

                if point.0 > f_f
                // line is left from the point - the ray moving towards
                {
                    // left, will intersect it
                    intersections += 1;
                } else if point.0 == f_f
                // point on line
                {
                    return -1;
                }
            }
            // # point on upper peak (dy2=dx2=0) or horizontal line (dy=dy2=0 and
            // dx*dx2<=0)
            else if dy2 == 0.
                && (point.0 == polygon[jj].0
                    || (dy == 0. && (point.0 - polygon[ii].0) * (point.0 - polygon[jj].0) <= 0.))
            {
                return -1;
            }
        }
        jj += 1;
    }
    return intersections & 1;
}


pub fn polygon2tris(poly: &Polygon<f64>)->Vec<((f64, f64), (f64, f64), (f64, f64))>{
    let mut edges: Vec<((f64, f64), (f64, f64))> = vec![];
    let mut verts: Vec<(f64, f64)> = vec![];
    let mut tris: Vec<((f64, f64), (f64, f64), (f64, f64))> = vec![];
    let max_verts: usize = 500;
    let mut grid: Vec<bool> = vec![false; max_verts * max_verts];
    let mut hole_pts:Vec<Coordinate<f64>> = vec![];

    // collect all sections
    let exterior: Vec<Coordinate<f64>> = poly.exterior().clone().into_iter().collect();
    for i in 0..(exterior.len()-1){
        edges.push(((exterior[i].x, exterior[i].y), (exterior[i+1].x, exterior[i+1].y)));
        if i == (exterior.len()-2){
            let next_idx = verts.len() - i;
            grid[verts.len() * max_verts + next_idx] = true;
            grid[next_idx * max_verts + verts.len()] = true;
        }
        else
        {
            grid[verts.len() * max_verts + verts.len() + 1] = true;
            grid[(verts.len() + 1) * max_verts + verts.len()] = true;    
        }
        verts.push((exterior[i].x, exterior[i].y));
    }
    for line in poly.interiors(){
        let pointts: Vec<Coordinate<f64>> = line.clone().into_iter().collect();
        for i in 0..(pointts.len()-1){
            edges.push(((pointts[i].x, pointts[i].y), (pointts[i+1].x, pointts[i+1].y)));
            if i == (pointts.len()-2){
                let next_idx = verts.len() - i;
                grid[verts.len() * max_verts + next_idx] = true;
                grid[next_idx * max_verts + verts.len()] = true;
            }
            else
            {
                grid[verts.len() * max_verts + verts.len() + 1] = true;
                grid[(verts.len() + 1) * max_verts + verts.len()] = true;            
            }    
            verts.push((pointts[i].x, pointts[i].y));
            hole_pts.push(pointts[i].clone());
        }
    }


    // generate edges
    let len = verts.len();
    for i in 0..len{
        for j in 0..len{
            if i==j {continue;}

            let a = verts[i];
            let b = verts[j];
            let mut exist_edge = false;

            // check already exist
            for edge in &edges{
                if (a == edge.0 && b ==  edge.1)  || (b == edge.0 && a ==  edge.1) {
                    exist_edge = true;
                    break;
                }
            }
            if exist_edge{continue;}

            //check intersect with other edges            
            for edge in &edges{
                if segment_intersects(a, b, edge.0, edge.1) == 1{
                    exist_edge = true;
                    break;
                }
            }
            if exist_edge{continue;}

            // check if line is outside poly 
            let x = (a.0 + b.0)/2.0; 
            let y = (a.1 + b.1)/2.0; 
            if pt_inside_line(x, y, &poly.exterior()) == 0{continue;}

            // check if line is inside poly
            for line in poly.interiors(){
                if pt_inside_line(x, y, &line) == 1{
                    exist_edge = true;
                    continue;
                }
            }
            if exist_edge{continue;}

            edges.push(((a.0, a.1), (b.0, b.1)));
            grid[i* max_verts + j] = true;
            grid[j*max_verts  + i] = true;
        }
    }

    // println!("make tris={:?}, {:?}", len, edges.len());
    // make triangles
    for i in 0..len{
        for j in (i+1)..len{
            for k in (j+1)..len{
                if grid[i * max_verts + j] && grid[j * max_verts + k] && grid[i * max_verts + k]{
                    let polygon = vec![verts[i], verts[j], verts[k]];
                    let mut include_hole = false;
                    for hole in &hole_pts
                    {
                        if point_inside_poly((hole.x, hole.y), &polygon) == 1{
                            include_hole = true;
                            break;
                        }
                    }    
                    if include_hole ==false{
                        tris.push((verts[i], verts[j], verts[k]));
                    }                
                }
           }
        }
    }        
    // println!("res={:?}, {:?}", tris.len(), path_time_start.elapsed().as_micros());
    return tris;
}



