use std::collections::HashMap;
use std::fs::*;
use std::io::*;
use std::path::*;

use john_wick_parse::assets::*;

// A static representation of the dynamic data found inside a uasset file.

#[derive(Debug)]
pub struct InterpCurvePointVector {
    in_val: f32,
    out_val: FVector,
    arrive_tangent: FVector,
    leave_tangent: FVector,
    interp_mode: String,
}

#[derive(Debug)]
pub struct InterpCurveVector {
    points: Vec<InterpCurvePointVector>,
    b_is_looped: bool,
    loop_key_offset: f32,
}

#[derive(Debug)]
pub struct LMQuestMovePath {
    start_node: String,
    end_node: String,
    position: Vec<InterpCurveVector>,
    path_length: f32,
}

#[derive(Debug)]
pub struct QuestMoveNode {
    neighbor_distances: HashMap<String, i32>,
}

#[derive(Debug)]
pub struct QuestMoveSystem {
    file_path: String,
    way_graph: HashMap<String, QuestMoveNode>,
    pathes: Vec<LMQuestMovePath>,
    map_id: f32,
}

// Check if the given file is a umap/uasset file. If so, check to see if there
// any waypoint data inside it and add it to the output vector.

pub fn process_asset(path: &Path, qmss: &mut Vec<QuestMoveSystem>) -> std::option::Option<()> {
    let parent = path.parent()?;
    let file_stem = path.file_stem()?;
    let extension = path.extension()?;
    if extension == "umap" || extension == "uasset" {
        let path_wo_ext: PathBuf = [parent, Path::new(file_stem)].iter().collect();
        let package = Package::from_file(path_wo_ext.to_str()?).ok()?;
        for export in &package.get_exports() {
            if let Export::UObject(obj) = export {
                if obj.get_export_type() == "BP_QuestMoveSystem_C" {
                    let mut map_id = None;
                    let mut way_graph = HashMap::new();
                    let mut pathes = Vec::new();
                    for prop in obj.get_properties() {
                        match prop {
                            FPropertyTag {
                                ref name,
                                tag: Some(FPropertyTagType::FloatProperty(val)),
                                ..
                            } if name == "_map_id" => map_id = Some(*val),
                            FPropertyTag {
                                ref name,
                                tag: Some(FPropertyTagType::MapProperty(UScriptMap { map_data })),
                                ..
                            } if name == "_way_graph" => {
                                way_graph.clear();
                                for (map_data_key, map_data_val) in map_data {
                                    let mut neighbor_distances = HashMap::new();
                                    if let FPropertyTagType::StructProperty(UScriptStruct {
                                        struct_type:
                                            StructType::FStructFallback(FStructFallback { properties }),
                                        ..
                                    }) = map_data_val
                                    {
                                        neighbor_distances.clear();
                                        for prop in properties {
                                            match prop {
                                                FPropertyTag {
                                                    ref name,
                                                    tag:
                                                        Some(FPropertyTagType::MapProperty(
                                                            UScriptMap { map_data },
                                                        )),
                                                    ..
                                                } if name == "_neighbor_distances" => {
                                                    neighbor_distances.clear();
                                                    for (map_data_key, map_data_val) in map_data {
                                                        if let FPropertyTagType::NameProperty(
                                                            map_data_key,
                                                        ) = map_data_key
                                                        {
                                                            if let FPropertyTagType::IntProperty(
                                                                map_data_val,
                                                            ) = map_data_val
                                                            {
                                                                neighbor_distances.insert(
                                                                    map_data_key.to_string(),
                                                                    *map_data_val,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    let neighbor_distances_name =
                                        if let FPropertyTagType::NameProperty(
                                            neighbor_distances_name,
                                        ) = map_data_key
                                        {
                                            neighbor_distances_name.to_string()
                                        } else {
                                            "unk_neighbor_distances".to_owned()
                                        };
                                    way_graph.insert(
                                        neighbor_distances_name,
                                        QuestMoveNode { neighbor_distances },
                                    );
                                }
                            }
                            FPropertyTag {
                                ref name,
                                tag:
                                    Some(FPropertyTagType::ArrayProperty(UScriptArray { data, .. })),
                                ..
                            } if name == "_pathes" => {
                                for path in data {
                                    if let FPropertyTagType::StructProperty(UScriptStruct {
                                        struct_type:
                                            StructType::FStructFallback(FStructFallback { properties }),
                                        ..
                                    }) = path
                                    {
                                        let mut start_node_name = None;
                                        let mut end_node_name = None;
                                        let mut path_length = None;
                                        let mut position = Vec::new();
                                        for prop in properties {
                                            match prop {
                                                FPropertyTag {
                                                    ref name,
                                                    tag:
                                                        Some(FPropertyTagType::NameProperty(
                                                            start_node_name_tmp,
                                                        )),
                                                    ..
                                                } if name == "_start_node" => {
                                                    start_node_name = Some(start_node_name_tmp);
                                                }
                                                FPropertyTag {
                                                    ref name,
                                                    tag:
                                                        Some(FPropertyTagType::NameProperty(
                                                            end_node_name_tmp,
                                                        )),
                                                    ..
                                                } if name == "_end_node" => {
                                                    end_node_name = Some(end_node_name_tmp);
                                                }
                                                FPropertyTag {
                                                    ref name,
                                                    tag:
                                                        Some(FPropertyTagType::FloatProperty(
                                                            path_length_tmp,
                                                        )),
                                                    ..
                                                } if name == "_path_length" => {
                                                    path_length = Some(path_length_tmp);
                                                }
                                                FPropertyTag {
                                                    ref name,
                                                    tag:
                                                        Some(FPropertyTagType::StructProperty(
                                                            UScriptStruct {
                                                                struct_type:
                                                                    StructType::FStructFallback(
                                                                        FStructFallback {
                                                                            properties,
                                                                        },
                                                                    ),
                                                                ..
                                                            },
                                                        )),
                                                    ..
                                                } if name == "Position" => {
                                                    let mut b_is_looped = None;
                                                    let mut loop_key_offset = None;
                                                    let mut points = Vec::new();
                                                    for prop in properties {
                                                        match prop {
                                                          FPropertyTag { ref name, tag: Some(FPropertyTagType::BoolProperty(b_is_looped_tmp)), .. } if name == "bIsLooped" => {
                                                            b_is_looped = Some(b_is_looped_tmp);
                                                          }, FPropertyTag { ref name, tag: Some(FPropertyTagType::FloatProperty(loop_key_offset_tmp)), .. } if name == "LoopKeyOffset" => {
                                                            loop_key_offset = Some(loop_key_offset_tmp);
                                                          }, FPropertyTag { ref name, tag: Some(FPropertyTagType::ArrayProperty(UScriptArray { data, .. })), .. } if name == "Points" => {
                                                            points.clear();
                                                            for point in data {
                                                              if let FPropertyTagType::StructProperty(UScriptStruct { struct_type: StructType::FStructFallback(FStructFallback { properties }), .. }) = point {
                                                                let mut in_val = None;
                                                                let mut out_val = None;
                                                                let mut arrive_tangent = None;
                                                                let mut leave_tangent = None;
                                                                let mut interp_mode = None;
                                                                for prop in properties {
                                                                  match prop {
                                                                    FPropertyTag { ref name, tag: Some(FPropertyTagType::FloatProperty(in_val_tmp)), .. } if name == "InVal" => {
                                                                      in_val = Some(in_val_tmp);
                                                                    }, FPropertyTag { ref name, tag: Some(FPropertyTagType::StructProperty(UScriptStruct { struct_type: StructType::FVector(out_val_tmp), .. })), .. } if name == "OutVal" => {
                                                                      out_val = Some(out_val_tmp);
                                                                    }, FPropertyTag { ref name, tag: Some(FPropertyTagType::StructProperty(UScriptStruct { struct_type: StructType::FVector(arrive_tangent_tmp), .. })), .. } if name == "ArriveTangent" => {
                                                                      arrive_tangent = Some(arrive_tangent_tmp);
                                                                    }, FPropertyTag { ref name, tag: Some(FPropertyTagType::StructProperty(UScriptStruct { struct_type: StructType::FVector(leave_tangent_tmp), .. })), .. } if name == "LeaveTangent" => {
                                                                      leave_tangent = Some(leave_tangent_tmp);
                                                                    }, FPropertyTag { ref name, tag: Some(FPropertyTagType::NameProperty(interp_mode_tmp)), .. } if name == "InterpMode" => {
                                                                      interp_mode = Some(interp_mode_tmp);
                                                                    }, _ => {}
                                                                  }
                                                                }
                                                                points.push(InterpCurvePointVector {
                                                                  in_val: *in_val.unwrap(),
                                                                  out_val: *out_val.unwrap(),
                                                                  arrive_tangent: *arrive_tangent.unwrap(),
                                                                  leave_tangent: *leave_tangent.unwrap(),
                                                                  interp_mode: (*interp_mode.unwrap()).to_string(),
                                                                });
                                                              }
                                                            }
                                                          }, _ => {}
                                                        }
                                                    }
                                                    position.push(InterpCurveVector {
                                                        points: points,
                                                        b_is_looped: *b_is_looped.unwrap(),
                                                        loop_key_offset: *loop_key_offset.unwrap(),
                                                    });
                                                }
                                                _ => {}
                                            }
                                        }
                                        pathes.push(LMQuestMovePath {
                                            start_node: (*start_node_name.unwrap()).to_string(),
                                            end_node: (*end_node_name.unwrap()).to_string(),
                                            position: position,
                                            path_length: *path_length.unwrap(),
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    qmss.push(QuestMoveSystem {
                        file_path: path_wo_ext.to_str()?.to_string(),
                        map_id: map_id.unwrap_or(0.0),
                        way_graph: way_graph,
                        pathes: pathes,
                    });
                }
            }
        }
    }
    Some(())
}

// Recursively visit all the files inside the given directory and call the
// given visitor on each. Returns the context variable.

pub fn visit_dirs<'a, T>(
    path: &Path,
    f: &dyn Fn(&Path, &mut T) -> std::option::Option<()>,
    ctx: &'a mut T,
) -> Result<&'a mut T> {
    let dir_iter = read_dir(path)?;
    for dir_entry in dir_iter {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        if path.is_dir() {
            let _ = visit_dirs(&path, f, ctx);
        } else {
            f(&path, ctx);
        }
    }
    Ok(ctx)
}
