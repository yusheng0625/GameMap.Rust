// check section is cross

fn vector_cross(a: (f64, f64, f32), b: (f64, f64, f32)) -> f64{
    let z = a.0 * b.1 - b.0 * a.1;
    return z;
}

fn vector_minus(a: (f64, f64, f32), b: (f64, f64, f32)) -> (f64, f64, f32){
    return (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}


// fn vector_is_less(a: (f64, f64, f32), b: (f64, f64, f32)) -> bool{
//     return a.0 < b.0 && a.1 < b.1;
// }
fn vector_is_less(a: (f64, f64, f32), b: (f64, f64, f32)) -> bool{
    return a.0 <= b.0 && a.1 <= b.1;
}


fn ccw0(a: (f64, f64, f32), b: (f64, f64, f32))->f64{
    return vector_cross(a, b);
}
//p-origin  if p-b is reverse clock dir of p-a => positive
//if p-b is clock dir of p-a => negotive
//if p-b is same dir of p-a => 0
fn ccw(a: (f64, f64, f32), b: (f64, f64, f32), p: (f64, f64, f32))->f64{
    return ccw0(vector_minus(a, p), vector_minus(b, p));
}

fn dist(a: (f64, f64, f32), b: (f64, f64, f32)) -> f64
{
    let v = (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1);
    return v.sqrt();
}

pub fn segment_intersects(aa: (f64, f64, f32), bb: (f64, f64, f32), cc: (f64, f64, f32), dd: (f64, f64, f32))->bool{
    let ab = ccw(aa, bb, cc) * ccw(aa, bb, dd);
    let cd = ccw(cc, dd ,aa) * ccw(cc, dd, bb);

    if ab ==0.0 && cd == 0.0{
        let a;
        let b;
        let c;
        let d;

        if vector_is_less(bb, aa){
            a = bb;
            b = aa;
        }
        else
        {
            a = aa;
            b = bb;
        }

        if vector_is_less(dd, cc){
            c = dd;
            d = cc;
        }else {
            c = cc;
            d = dd;
        }
        return !(vector_is_less(b , c) || vector_is_less(d ,a));
    }
    return ab <=0.0 && cd <=0.0;
}






pub fn string_pull(s: (i64, i64, f32), e: (i64, i64, f32), edgs: Vec<((i64, i64, f32), (i64, i64, f32))>) -> Vec<(i64, i64, f32)>
{
    // convert to f64
    let mut edges: Vec<((f64, f64, f32), (f64, f64, f32))> = edgs.iter().map(|((x, y, z), (x1, y1, z1))|{

        //return ((*x as f64, *y as f64, *z), (*x1 as f64, *y1 as f64, *z1));
        let xx1 = *x as f64;
        let xx2 = *x1 as f64;
        let zz1 = *z as f64;
        let yy1 = *y as f64;
        let yy2 = *y1 as f64;
        let zz2 = *z1 as f64;

        let mut delta64: f64 = 50.0;

        let dst = dist((xx1, yy1, zz1 as f32), (xx2, yy2, zz2 as f32));
        if  dst < 60.0
        {
            return ((*x as f64, *y as f64, *z), (*x1 as f64, *y1 as f64, *z));
        }else if dst < 100.0
        {
            delta64 = 30.0;
        }

        let xxx1 = (delta64 * xx2 + (dst - delta64) * xx1)/dst;
        let yyy1 = (delta64 * yy2 + (dst - delta64) * yy1)/dst;
        let zzz1 = (delta64 * zz2 + (dst - delta64) * zz1)/dst;

        let xxx2 = ((dst - delta64) * xx2 + delta64 * xx1)/dst;
        let yyy2 = ((dst - delta64) * yy2 + delta64 * yy1)/dst;
        let zzz2 = ((dst - delta64) * zz2 + delta64 * zz1)/dst;
        return ((xxx1, yyy1, zzz1 as f32), (xxx2, yyy2, zzz2 as f32));

    }).collect();


    edges.insert(0, ((s.0 as f64, s.1 as f64, s.2), (s.0 as f64, s.1 as f64, s.2)));
    edges.insert(edges.len(), ((e.0 as f64, e.1 as f64, e.2), (e.0 as f64, e.1 as f64, e.2)));

    // println!("edges = {:?}", edges);

    let mut portal_apex = edges[0].0;
    let mut apex_index = 0;
    let mut results: Vec<(f64, f64, f32)> = vec![];
    let mut last_index;

    let mut b_all_crossed:bool;
    //results.insert(results.len(), portal_apex);

    while apex_index < edges.len()-2 {
        // Update right vertex.

        last_index = apex_index + 1;
        for ii in (apex_index+2)..edges.len(){
            b_all_crossed = true;
            let tmp_apex = (((edges[ii].0).0 + (edges[ii].1).0)/2.0,  ((edges[ii].0).1 + (edges[ii].1).1)/2.0, ((edges[ii].0).2 + (edges[ii].1).2)/2.0);
            for jj in (apex_index+1)..ii{
                if segment_intersects(portal_apex, tmp_apex, edges[jj].0, edges[jj].1) == false{
                    b_all_crossed = false;
                    break;
                }
            }
            if b_all_crossed==false{
                break;
            }
            last_index = ii;
        }

        portal_apex = (((edges[last_index].0).0 + (edges[last_index].1).0)/2.0,  ((edges[last_index].0).1 + (edges[last_index].1).1)/2.0, ((edges[last_index].0).2 + (edges[last_index].1).2)/2.0);
        apex_index = last_index;
        results.insert(results.len(), portal_apex);
    }

    results.insert(results.len(), edges[edges.len() - 1].0);
    results.dedup();

    let result: Vec<(i64, i64, f32)> = results.iter().map(|(x, y, z)|{
        return (*x as i64, *y as i64, *z);
    }).collect();

    return result;
}
